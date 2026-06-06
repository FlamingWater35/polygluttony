//! Glossary run manager: one glossary op at a time (build / standalone
//! normalize / import), mutually exclusive with translation runs. Mirrors
//! `translation/run.rs` with a simpler forwarder (the slot is released by the
//! op itself, not by channel close).

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::config::store as config_store;
use crate::config::{AppConfig, Connection, Driver};
use crate::error::{AppError, AppResult};
use crate::events::{self, GlossaryEvent, RunEvent};
use crate::glossary::build::{build_glossary, BuildJob};
use crate::glossary::world_detector::WorldType;
use crate::llm::service::LlmService;
use crate::llm::LlmDriver;
use crate::models::language_pair::LanguagePair;

/// Managed Tauri state.
#[derive(Default)]
pub struct GlossaryRunState(pub Mutex<Option<GlossaryOpHandle>>);

pub struct GlossaryOpHandle {
    #[allow(dead_code)] // surfaced to the UI in a later step; cancel is the point
    pub kind: GlossaryOpKind,
    pub cancel: CancellationToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlossaryOpKind {
    Build,
    // claimed by the normalize/import commands (later step-4 tasks)
    #[allow(dead_code)]
    Normalize,
    #[allow(dead_code)]
    Import,
}

// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub struct StartArgs {
    pub folder: String,
    pub files: Vec<String>,
    pub world_type: WorldType,
    pub source_lang: String,
    pub target_lang: String,
    pub normalize: bool,
    pub personalize: bool,
    pub personalize_context: String,
}

/// Claim the single glossary-op slot. LOCK ORDERING: translation `RunState`
/// FIRST, then `GlossaryRunState` — `translation::run::start` uses the same
/// order; reversing it deadlocks.
pub async fn claim_slot(app: &AppHandle, kind: GlossaryOpKind) -> AppResult<CancellationToken> {
    let t_state = app.state::<crate::translation::run::RunState>();
    let t_guard = t_state.0.lock().await;
    if t_guard.is_some() {
        return Err(AppError::RunAlreadyActive);
    }
    let g_state = app.state::<GlossaryRunState>();
    let mut g_guard = g_state.0.lock().await;
    if g_guard.is_some() {
        return Err(AppError::RunAlreadyActive);
    }
    let cancel = CancellationToken::new();
    *g_guard = Some(GlossaryOpHandle { kind, cancel: cancel.clone() });
    Ok(cancel)
}

pub async fn release_slot(app: &AppHandle) {
    if let Some(state) = app.try_state::<GlossaryRunState>() {
        *state.0.lock().await = None;
    }
}

/// Forward engine events to the webview until the channel closes. Unlike the
/// translation forwarder this does NOT clear the run slot — the op that
/// claimed it releases it.
pub fn spawn_forwarder(app: AppHandle, mut rx: mpsc::Receiver<GlossaryEvent>) {
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = rx.recv().await {
            let _ = app.emit(events::GLOSSARY_EVENT, &ev);
        }
    });
}

/// `LlmService` emits its metering logs as `RunEvent::Log` (step-3 type). This
/// adapter bridges them onto the glossary channel without touching LlmService.
pub fn llm_log_channel(g_tx: mpsc::Sender<GlossaryEvent>) -> mpsc::Sender<RunEvent> {
    let (tx, mut rx) = mpsc::channel::<RunEvent>(64);
    tauri::async_runtime::spawn(async move {
        while let Some(ev) = rx.recv().await {
            if let RunEvent::Log { level, message, .. } = ev {
                let _ = g_tx.send(GlossaryEvent::Log { level, message }).await;
            }
        }
    });
    tx
}

/// Build an `LlmService` for a glossary op on `conn`.
// consumed by the normalize/import commands too (later step-4 tasks)
#[allow(dead_code)]
pub fn service_for(
    conn: &Connection,
    cancel: CancellationToken,
    g_tx: mpsc::Sender<GlossaryEvent>,
) -> LlmService {
    let concurrency = conn.concurrency.unwrap_or(2).max(1);
    let driver: Arc<dyn LlmDriver> = Arc::from(crate::llm::create_driver(conn.clone()));
    LlmService::new(driver, concurrency, cancel, llm_log_channel(g_tx))
}

/// The personalization connection, iff usable AND web-capable
/// (openai-responses driver with web search on).
pub fn web_capable_personalization(cfg: &AppConfig) -> Option<(String, Connection)> {
    let name = cfg.personalization_model.as_ref()?;
    let conn = cfg.connections.get(name)?;
    if !config_store::connection_is_usable(conn) {
        return None;
    }
    if conn.driver == Driver::OpenaiResponses && conn.web_search == Some(true) {
        Some((name.clone(), conn.clone()))
    } else {
        None
    }
}

// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub async fn start(app: AppHandle, args: StartArgs) -> AppResult<()> {
    if args.files.is_empty() {
        return Err(AppError::Other("no files selected".into()));
    }
    let pair = LanguagePair::from_codes(&args.source_lang, &args.target_lang)?;
    if !pair.supports_glossary {
        return Err(AppError::Other(format!(
            "glossary isn't available for {}",
            pair.source_name
        )));
    }
    let cfg = config_store::load(&app)?;
    let conn =
        crate::translation::run::usable_connection(&cfg).ok_or(AppError::NoActiveConnection)?;

    let cancel = claim_slot(&app, GlossaryOpKind::Build).await?;
    let (tx, rx) = mpsc::channel::<GlossaryEvent>(512);
    spawn_forwarder(app.clone(), rx);

    let svc = service_for(&conn, cancel.clone(), tx.clone());
    // Personalization service only when requested AND a web-capable
    // personalization connection exists (cap 1 — single call). MUST share the
    // build's cancel token so cancel can interrupt a long web-search call.
    let personalize_svc = if args.personalize {
        web_capable_personalization(&cfg).map(|(_, p_conn)| {
            let d: Arc<dyn LlmDriver> = Arc::from(crate::llm::create_driver(p_conn));
            LlmService::new(d, 1, cancel.clone(), llm_log_channel(tx.clone()))
        })
    } else {
        None
    };

    let job = BuildJob {
        folder: PathBuf::from(&args.folder),
        files: args.files,
        world_type: args.world_type.as_str().to_string(),
        pair,
        normalize: args.normalize,
        personalize: args.personalize,
        personalize_context: args.personalize_context,
        template_variant: conn.prompt_template.clone(),
        batch_limit: conn.batch_dialogue_limit,
        cancel: cancel.clone(),
    };
    let app_for_release = app.clone();
    tauri::async_runtime::spawn(async move {
        build_glossary(job, &svc, personalize_svc.as_ref(), tx).await;
        // tx dropped by build_glossary's end of scope → forwarder drains & exits.
        release_slot(&app_for_release).await;
    });
    Ok(())
}

/// Cancels whichever glossary op holds the slot (build, normalize, import).
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub async fn cancel(app: AppHandle) -> AppResult<()> {
    let state = app.state::<GlossaryRunState>();
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

    #[test]
    fn web_capable_personalization_gating() {
        let mut cfg = default_config();
        // Point personalization at a connection we fully control.
        cfg.personalization_model = Some("p".into());
        let mut conn = cfg.connections.values().next().unwrap().clone();
        conn.driver = Driver::OpenaiResponses;
        conn.web_search = Some(true);
        conn.api_key = "sk-x".into();
        conn.base_url = "https://api.example.com".into();
        cfg.connections.insert("p".into(), conn);
        assert_eq!(web_capable_personalization(&cfg).unwrap().0, "p");

        // Wrong driver → None.
        cfg.connections.get_mut("p").unwrap().driver = Driver::Openai;
        assert!(web_capable_personalization(&cfg).is_none());
        cfg.connections.get_mut("p").unwrap().driver = Driver::OpenaiResponses;

        // Web search off → None.
        cfg.connections.get_mut("p").unwrap().web_search = Some(false);
        assert!(web_capable_personalization(&cfg).is_none());
        cfg.connections.get_mut("p").unwrap().web_search = Some(true);

        // Unusable (no key, not localhost) → None.
        cfg.connections.get_mut("p").unwrap().api_key.clear();
        assert!(web_capable_personalization(&cfg).is_none());

        // No personalization set → None.
        cfg.personalization_model = None;
        assert!(web_capable_personalization(&cfg).is_none());
    }
}
