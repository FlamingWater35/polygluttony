//! Run manager: one active run; spawns ≤concurrency file pipelines; forwards
//! engine events to the webview; persists verify results.
//!
//! # tx lifecycle — why the forwarder terminates
//!
//! There are three `mpsc::Sender<RunEvent>` clones at peak:
//!
//! 1. **`tx`** — held by the spawner task until it drops at the end of its
//!    body (after sending `RunFinished`).
//! 2. **`job_tx`** — a per-file clone passed into each `translate_file` call;
//!    dropped when the file task returns.
//! 3. **`svc`'s internal clone** — `LlmService::new(driver, cap, cancel, tx)` stores
//!    one clone to emit `Log` events. `svc` is wrapped in `Arc` and cloned once per
//!    file task (`job_svc`). The spawner's own `svc` binding and all `job_svc` arcs
//!    are dropped when the spawner task and each file task finish. No other code path
//!    holds an `Arc<LlmService>`, so the service and its inner tx clone are freed once
//!    the last file task exits and the spawner drops its `svc`.
//!
//! Once (1), (2) for every file, and (3) have all dropped, `rx.recv()` returns `None`
//! and the forwarder loop exits — triggering persistence and state clear.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

use crate::config::projects::Tone;
use crate::config::store as config_store;
use crate::config::AppConfig;
use crate::config::Connection;
use crate::error::{AppError, AppResult};
use crate::events::{self, FileResult, RunEvent};
use crate::glossary::io::load_folder_glossary;
use crate::glossary::model::Glossary;
use crate::llm::service::LlmService;
use crate::models::language_pair::LanguagePair;
use crate::translation::pipeline::{translate_file, FileJob};

/// Managed Tauri state.
#[derive(Default)]
pub struct RunState(pub Mutex<Option<RunHandle>>);

pub struct RunHandle {
    pub cancel: CancellationToken,
}

pub struct StartArgs {
    pub folder: String,
    pub files: Vec<String>,
    pub tone: Tone,
    pub source_lang: String,
    pub target_lang: String,
    pub now: i64, // webview-supplied timestamp (same convention as open_folder)
}

/// Return the active connection if it is usable (non-empty API key, or a
/// localhost/127.0.0.1 base URL which doesn't need one). Pure helper so it can
/// be unit-tested without an AppHandle.
pub fn usable_connection(cfg: &AppConfig) -> Option<Connection> {
    let conn = cfg.connections.get(&cfg.active_connection)?;
    let key_ok = !conn.api_key.trim().is_empty();
    let local = conn.base_url.contains("localhost") || conn.base_url.contains("127.0.0.1");
    if key_ok || local {
        Some(conn.clone())
    } else {
        None
    }
}

pub async fn start(app: AppHandle, args: StartArgs) -> AppResult<()> {
    let state = app.state::<RunState>();
    let mut guard = state.0.lock().await;
    if guard.is_some() {
        return Err(AppError::RunAlreadyActive);
    }

    let cfg = config_store::load(&app)?;
    let conn = usable_connection(&cfg).ok_or(AppError::NoActiveConnection)?;

    if args.files.is_empty() {
        return Err(AppError::Other("no files selected".into()));
    }

    let pair = LanguagePair::from_codes(&args.source_lang, &args.target_lang)?;
    let folder = PathBuf::from(&args.folder);
    let glossary: Arc<Glossary> = Arc::new(load_folder_glossary(&folder).unwrap_or_default());

    let cancel = CancellationToken::new();
    let (tx, mut rx) = mpsc::channel::<RunEvent>(512);
    let concurrency = conn.concurrency.unwrap_or(2).max(1);
    let driver: Arc<dyn crate::llm::LlmDriver> = Arc::from(crate::llm::create_driver(conn.clone()));
    let svc = Arc::new(LlmService::new(driver, concurrency, cancel.clone(), tx.clone()));

    *guard = Some(RunHandle { cancel: cancel.clone() });
    drop(guard);

    // Forwarder: engine events → webview; collects results; persists; clears state.
    //
    // `rx.recv()` returns None once all Sender clones are dropped. The spawner
    // holds `tx` (dropped at end of spawner body) and `svc` (which holds one
    // inner tx clone, dropped when the Arc refcount hits zero after every file
    // task and the spawner itself drop their `Arc<LlmService>`). Each file task
    // also holds `job_tx` (dropped when the task returns). So the forwarder
    // exits naturally after RunFinished is sent and all tasks have finished.
    let app_fwd = app.clone();
    let folder_key = args.folder.clone();
    let now = args.now;
    tauri::async_runtime::spawn(async move {
        let mut results: Vec<FileResult> = Vec::new();
        while let Some(ev) = rx.recv().await {
            if let RunEvent::RunFinished { results: r } = &ev {
                results = r.clone();
            }
            let _ = app_fwd.emit(events::TRANSLATION_EVENT, &ev);
        }
        if !results.is_empty() {
            let _ = crate::config::verification::save_folder(&app_fwd, &folder_key, &results, now);
        }
        if let Some(state) = app_fwd.try_state::<RunState>() {
            *state.0.lock().await = None;
        }
    });

    // Worker spawner.
    let sem = Arc::new(Semaphore::new(concurrency as usize));
    let batch_limit = conn.batch_dialogue_limit;
    let template_variant = conn.prompt_template.clone();
    let tone = args.tone;
    tauri::async_runtime::spawn(async move {
        // Zip file names with task handles so a panicking task can emit a named error.
        let mut handles: Vec<(String, tauri::async_runtime::JoinHandle<FileResult>)> = Vec::new();
        for name in args.files {
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            if cancel.is_cancelled() {
                drop(permit);
                break;
            }
            let job_tx = tx.clone();
            let job_cancel = cancel.clone();
            let job_svc = svc.clone();
            let job_glossary = glossary.clone();
            let job_pair = pair.clone();
            let input = folder.join(&name);
            let job_variant = template_variant.clone();
            let task_name = name.clone();
            let handle = tauri::async_runtime::spawn(async move {
                let _permit = permit;
                translate_file(FileJob {
                    input,
                    file_name: task_name,
                    svc: &job_svc,
                    glossary: &job_glossary,
                    pair: job_pair,
                    tone,
                    template_variant: job_variant,
                    batch_limit,
                    cancel: job_cancel,
                    tx: job_tx,
                })
                .await
            });
            handles.push((name, handle));
        }

        // Drop svc here so the service's inner tx clone is released before we
        // wait on handles. File tasks each have their own Arc<LlmService> clone;
        // those drop when their task finishes. The spawner's `svc` binding is
        // the only one left at this point, and must be dropped before the join
        // loop or the forwarder could otherwise outlive the RunFinished send.
        drop(svc);

        let mut results = Vec::new();
        for (name, handle) in handles {
            match handle.await {
                Ok(r) => results.push(r),
                Err(_panic) => {
                    // Task panicked: emit a named error event and push a failure
                    // result so the run doesn't silently drop the file.
                    let _ = tx
                        .send(RunEvent::Error {
                            file: name.clone(),
                            message: "file task panicked".into(),
                        })
                        .await;
                    results.push(FileResult {
                        file: name,
                        success: false,
                        total_lines: 0,
                        translated_lines: 0,
                        has_warnings: false,
                        issues: Vec::new(),
                        output_path: None,
                    });
                }
            }
        }
        // Send RunFinished then drop tx — forwarder loop ends once this and all
        // job_tx clones (already dropped by finished file tasks) are released.
        let _ = tx.send(RunEvent::RunFinished { results }).await;
        // tx dropped here
    });

    Ok(())
}

pub async fn cancel(app: AppHandle) -> AppResult<()> {
    let state = app.state::<RunState>();
    let guard = state.0.lock().await;
    match guard.as_ref() {
        Some(h) => {
            h.cancel.cancel();
            Ok(())
        }
        None => Err(AppError::NoActiveRun),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::presets::default_config;
    use crate::config::Driver;

    fn conn_with_key(key: &str, base_url: &str) -> Connection {
        Connection {
            driver: Driver::Openai,
            base_url: base_url.into(),
            api_key: key.into(),
            model: "m".into(),
            max_tokens: None,
            batch_dialogue_limit: None,
            timeout: None,
            connect_timeout: None,
            concurrency: None,
            thinking_enabled: None,
            thinking_budget: None,
            web_search: None,
            prompt_template: None,
            thinking_glossary_norm_budget: None,
        }
    }

    #[test]
    fn usable_connection_requires_key_or_localhost() {
        let mut cfg = default_config();
        // Clear all keys to put us in a clean state.
        for c in cfg.connections.values_mut() {
            c.api_key.clear();
            c.base_url = "https://api.example.com".into();
        }
        // No key, no localhost → None.
        assert!(usable_connection(&cfg).is_none());

        // Non-empty key → Some.
        cfg.connections.get_mut(&cfg.active_connection).unwrap().api_key = "sk-test".into();
        assert!(usable_connection(&cfg).is_some());

        // Localhost without key → Some.
        {
            let conn = cfg.connections.get_mut(&cfg.active_connection).unwrap();
            conn.api_key.clear();
            conn.base_url = "http://localhost:11434".into();
        }
        assert!(usable_connection(&cfg).is_some());

        // 127.0.0.1 without key → Some.
        cfg.connections.get_mut(&cfg.active_connection).unwrap().base_url =
            "http://127.0.0.1:11434".into();
        assert!(usable_connection(&cfg).is_some());
    }

    #[test]
    fn missing_active_connection_returns_none() {
        let mut cfg = default_config();
        cfg.active_connection = "nonexistent".into();
        assert!(usable_connection(&cfg).is_none());
    }
}
