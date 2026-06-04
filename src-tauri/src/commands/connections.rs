//! Connection management commands (O1–O5, O21) + presets/model listing.

use tauri::AppHandle;

use crate::config::presets::{presets, Preset};
use crate::config::store;
use crate::config::{AppConfig, Connection, Driver};
use crate::error::{AppError, AppResult};
use crate::llm::detect::detect_format;
use crate::llm::create_driver;
use crate::models::{ConnectionSummary, ConnectionsView, FirstRunStatus, TestResult};

/// Pure: AppConfig -> list view-model (no keys leaked).
pub(crate) fn build_connections_view(cfg: &AppConfig) -> ConnectionsView {
    let connections = cfg
        .connections
        .iter()
        .map(|(name, c)| ConnectionSummary {
            name: name.clone(),
            driver: c.driver,
            has_key: !c.api_key.trim().is_empty(),
        })
        .collect();
    ConnectionsView {
        connections,
        active: cfg.active_connection.clone(),
        personalization: cfg.personalization_model.clone(),
    }
}

#[tauri::command]
pub fn list_connections(app: AppHandle) -> AppResult<ConnectionsView> {
    Ok(build_connections_view(&store::load(&app)?))
}

#[tauri::command]
pub fn read_connection(app: AppHandle, name: String) -> AppResult<Connection> {
    let cfg = store::load(&app)?;
    cfg.connections
        .get(&name)
        .cloned()
        .ok_or_else(|| AppError::Other(format!("unknown connection: {name}")))
}

#[tauri::command]
pub fn save_connection(app: AppHandle, name: String, connection: Connection) -> AppResult<()> {
    let mut cfg = store::load(&app)?;
    store::upsert_connection(&mut cfg, &name, connection);
    store::save(&app, &cfg)
}

#[tauri::command]
pub fn delete_connection(app: AppHandle, name: String) -> AppResult<()> {
    let mut cfg = store::load(&app)?;
    store::remove_connection(&mut cfg, &name)?;
    store::save(&app, &cfg)
}

#[tauri::command]
pub fn set_active_connection(app: AppHandle, name: String) -> AppResult<()> {
    let mut cfg = store::load(&app)?;
    store::set_active(&mut cfg, &name)?;
    store::save(&app, &cfg)
}

#[tauri::command]
pub fn set_personalization_connection(app: AppHandle, name: String) -> AppResult<()> {
    let mut cfg = store::load(&app)?;
    store::set_personalization(&mut cfg, &name)?;
    store::save(&app, &cfg)
}

#[tauri::command]
pub fn first_run_status(app: AppHandle) -> AppResult<FirstRunStatus> {
    Ok(FirstRunStatus {
        has_usable_connection: store::has_usable_connection(&store::load(&app)?),
    })
}

#[tauri::command]
pub fn list_presets() -> Vec<Preset> {
    presets()
}

/// Build a minimal Connection for Test/model-list (thinking + web_search off,
/// tiny max_tokens) so the call is cheap and valid.
fn probe_connection(mut c: Connection, driver: Driver) -> Connection {
    c.driver = driver;
    c.thinking_enabled = Some(false);
    c.web_search = Some(false);
    c.max_tokens = Some(16);
    c
}

#[tauri::command]
pub async fn test_connection(connection: Connection) -> AppResult<TestResult> {
    // Resolve the driver: trust an explicit driver, but for a Custom connection
    // the UI sends `prompt_template == "__detect__"` as the detection sentinel.
    // Read base_url/api_key BEFORE moving connection into probe_connection.
    let detect = connection.prompt_template.as_deref() == Some("__detect__");
    let (driver, detected) = if detect {
        let base_url = connection.base_url.clone();
        let api_key = connection.api_key.clone();
        match detect_format(&base_url, &api_key).await {
            Ok(d) => (d, Some(d)),
            // Undetermined format (both routes 404 / host unreachable): surface a
            // graceful TestResult rather than an IPC-level error (spec §5.7).
            Err(e) => {
                return Ok(TestResult {
                    ok: false,
                    model: connection.model.clone(),
                    detected_driver: None,
                    message: e.to_string(),
                })
            }
        }
    } else {
        (connection.driver, None)
    };

    let mut probe = probe_connection(connection, driver);
    probe.prompt_template = None; // strip the sentinel
    let model = probe.model.clone();
    let client = create_driver(probe);
    match client.complete("Reply with OK.", "ping").await {
        Ok(_) => {
            let msg = match detected {
                Some(d) => format!("✓ Detected {} — responded as {model}", driver_label(d)),
                None => format!("✓ Connection works — responded as {model}"),
            };
            Ok(TestResult {
                ok: true,
                model,
                detected_driver: detected,
                message: msg,
            })
        }
        Err(e) => Ok(TestResult {
            ok: false,
            model,
            detected_driver: detected,
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn list_models(connection: Connection) -> AppResult<Vec<String>> {
    let driver = if connection.prompt_template.as_deref() == Some("__detect__") {
        let base_url = connection.base_url.clone();
        let api_key = connection.api_key.clone();
        detect_format(&base_url, &api_key).await?
    } else {
        connection.driver
    };
    let mut c = connection;
    c.prompt_template = None;
    c.driver = driver;
    Ok(create_driver(c).list_models().await.unwrap_or_default())
}

fn driver_label(d: Driver) -> &'static str {
    match d {
        Driver::Anthropic => "Anthropic-compatible",
        Driver::Openai | Driver::OpenaiResponses => "OpenAI-compatible",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::presets::default_config;

    #[test]
    fn view_lists_summaries_without_keys() {
        let cfg = default_config();
        let view = build_connections_view(&cfg);
        assert_eq!(view.active, "anthropic");
        assert_eq!(view.personalization.as_deref(), Some("openai"));
        let anthropic = view.connections.iter().find(|c| c.name == "anthropic").unwrap();
        assert!(!anthropic.has_key); // seeded empty
        let ollama = view.connections.iter().find(|c| c.name == "ollama").unwrap();
        assert!(ollama.has_key); // placeholder key
    }
}
