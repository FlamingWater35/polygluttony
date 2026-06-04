//! Config persistence. Pure `AppConfig` helpers (unit-tested) + a thin Tauri
//! store adapter (`load`/`save`) that seeds defaults on first run.

use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::config::{presets::default_config, AppConfig, Connection};
use crate::error::{AppError, AppResult};

const STORE_FILE: &str = "config.json";
const STORE_KEY: &str = "app";

// ---- pure helpers over AppConfig -------------------------------------------

pub fn upsert_connection(cfg: &mut AppConfig, name: &str, conn: Connection) {
    cfg.connections.insert(name.to_string(), conn);
}

pub fn set_active(cfg: &mut AppConfig, name: &str) -> AppResult<()> {
    if !cfg.connections.contains_key(name) {
        return Err(AppError::Other(format!("unknown connection: {name}")));
    }
    cfg.active_connection = name.to_string();
    Ok(())
}

pub fn set_personalization(cfg: &mut AppConfig, name: &str) -> AppResult<()> {
    if !cfg.connections.contains_key(name) {
        return Err(AppError::Other(format!("unknown connection: {name}")));
    }
    cfg.personalization_model = Some(name.to_string());
    Ok(())
}

pub fn remove_connection(cfg: &mut AppConfig, name: &str) -> AppResult<()> {
    if cfg.active_connection == name {
        return Err(AppError::Other(
            "reassign the active connection before removing it".into(),
        ));
    }
    cfg.connections.remove(name);
    Ok(())
}

/// First-run check (O21): any connection carrying a non-empty api_key.
pub fn has_usable_connection(cfg: &AppConfig) -> bool {
    cfg.connections.values().any(|c| !c.api_key.trim().is_empty())
}

// ---- Tauri store adapter (thin; not unit-tested) ---------------------------

/// Load the config from the store, seeding + persisting defaults on first run.
pub fn load<R: Runtime>(app: &AppHandle<R>) -> AppResult<AppConfig> {
    let store = app.store(STORE_FILE).map_err(|e| AppError::Other(e.to_string()))?;
    match store.get(STORE_KEY) {
        Some(value) => serde_json::from_value(value).map_err(AppError::from),
        None => {
            let cfg = default_config();
            store.set(STORE_KEY, serde_json::to_value(&cfg)?);
            store.save().map_err(|e| AppError::Other(e.to_string()))?;
            Ok(cfg)
        }
    }
}

/// Persist the whole config.
pub fn save<R: Runtime>(app: &AppHandle<R>, cfg: &AppConfig) -> AppResult<()> {
    let store = app.store(STORE_FILE).map_err(|e| AppError::Other(e.to_string()))?;
    store.set(STORE_KEY, serde_json::to_value(cfg)?);
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{presets::default_config, Connection, Driver};

    fn sample() -> Connection {
        Connection {
            driver: Driver::Openai, base_url: "u".into(), api_key: "k".into(),
            model: "m".into(), max_tokens: None, batch_dialogue_limit: None,
            timeout: None, connect_timeout: None, concurrency: None,
            thinking_enabled: None, thinking_budget: None, web_search: None,
            prompt_template: None, thinking_glossary_norm_budget: None,
        }
    }

    #[test]
    fn upsert_then_read_back() {
        let mut cfg = default_config();
        upsert_connection(&mut cfg, "mine", sample());
        assert_eq!(cfg.connections["mine"].api_key, "k");
    }

    #[test]
    fn set_active_requires_existing() {
        let mut cfg = default_config();
        assert!(set_active(&mut cfg, "anthropic").is_ok());
        assert_eq!(cfg.active_connection, "anthropic");
        assert!(set_active(&mut cfg, "nope").is_err());
    }

    #[test]
    fn delete_blocks_removing_active() {
        let mut cfg = default_config();
        set_active(&mut cfg, "anthropic").unwrap();
        // Removing the active connection is refused.
        assert!(remove_connection(&mut cfg, "anthropic").is_err());
        // A non-active one is removable.
        assert!(remove_connection(&mut cfg, "google").is_ok());
        assert!(!cfg.connections.contains_key("google"));
    }

    #[test]
    fn first_run_is_true_when_no_key() {
        let cfg = default_config(); // only ollama has a placeholder key
        // ollama's placeholder counts as a "usable" key, so default is NOT first-run.
        assert!(has_usable_connection(&cfg));
        let mut empty = default_config();
        for c in empty.connections.values_mut() { c.api_key.clear(); }
        assert!(!has_usable_connection(&empty));
    }
}
