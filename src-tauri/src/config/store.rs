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
    // Don't leave a dangling personalization reference to a deleted connection.
    if cfg.personalization_model.as_deref() == Some(name) {
        cfg.personalization_model = None;
    }
    Ok(())
}

/// Rename a connection, preserving its settings and any `active` /
/// `personalization` references that pointed at the old name.
pub fn rename_connection(cfg: &mut AppConfig, old: &str, new: &str) -> AppResult<()> {
    let new = new.trim();
    if new.is_empty() {
        return Err(AppError::Other("connection name cannot be empty".into()));
    }
    if old == new {
        return Ok(());
    }
    if !cfg.connections.contains_key(old) {
        return Err(AppError::Other(format!("unknown connection: {old}")));
    }
    if cfg.connections.contains_key(new) {
        return Err(AppError::Other(format!(
            "a connection named '{new}' already exists"
        )));
    }
    if let Some(conn) = cfg.connections.remove(old) {
        cfg.connections.insert(new.to_string(), conn);
    }
    if cfg.active_connection == old {
        cfg.active_connection = new.to_string();
    }
    if cfg.personalization_model.as_deref() == Some(old) {
        cfg.personalization_model = Some(new.to_string());
    }
    Ok(())
}

/// A single connection is usable when it has a non-empty API key **or** its
/// base URL points to localhost / 127.0.0.1 (e.g. an Ollama instance that
/// needs no key).
pub fn connection_is_usable(conn: &Connection) -> bool {
    !conn.api_key.trim().is_empty()
        || conn.base_url.contains("localhost")
        || conn.base_url.contains("127.0.0.1")
}

/// First-run check (O21): any connection that passes `connection_is_usable`.
///
/// Behaviour change from the old key-only check: a keyless localhost
/// connection (e.g. bare Ollama) now counts as usable, matching the
/// run-gate semantics in `run.rs::usable_connection`.
pub fn has_usable_connection(cfg: &AppConfig) -> bool {
    cfg.connections.values().any(connection_is_usable)
}

/// Update the global default source/target languages. These seed a newly opened
/// folder that has no saved per-folder preferences (and persist across sessions).
pub fn set_default_languages(cfg: &mut AppConfig, source: &str, target: &str) {
    cfg.default_source = source.to_string();
    cfg.default_target = target.to_string();
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
            thinking_enabled: None, thinking_budget: None, thinking_glossary_budget: None,
            thinking_glossary_norm_budget: None, web_search: None,
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
    fn rename_moves_entry_and_updates_references() {
        let mut cfg = default_config(); // active=anthropic, personalization=openai
        // Rename the active connection: the active reference follows.
        rename_connection(&mut cfg, "anthropic", "claude").unwrap();
        assert!(!cfg.connections.contains_key("anthropic"));
        assert!(cfg.connections.contains_key("claude"));
        assert_eq!(cfg.active_connection, "claude");
        // Rename the personalization connection: that reference follows too.
        rename_connection(&mut cfg, "openai", "gpt").unwrap();
        assert_eq!(cfg.personalization_model.as_deref(), Some("gpt"));
        // Collisions and empty names are rejected; same-name is a no-op.
        assert!(rename_connection(&mut cfg, "google", "claude").is_err());
        assert!(rename_connection(&mut cfg, "google", "  ").is_err());
        assert!(rename_connection(&mut cfg, "google", "google").is_ok());
    }

    #[test]
    fn removing_personalization_connection_clears_the_reference() {
        let mut cfg = default_config(); // personalization = "openai", active = "anthropic"
        assert_eq!(cfg.personalization_model.as_deref(), Some("openai"));
        remove_connection(&mut cfg, "openai").unwrap();
        assert_eq!(cfg.personalization_model, None);
    }

    #[test]
    fn set_default_languages_updates_config() {
        let mut cfg = default_config();
        set_default_languages(&mut cfg, "ja", "en");
        assert_eq!(cfg.default_source, "ja");
        assert_eq!(cfg.default_target, "en");
    }

    #[test]
    fn first_run_detection() {
        // Default config: ollama has both a placeholder key AND a localhost base_url.
        // Either alone makes it usable, so the default config is never "first run".
        let cfg = default_config();
        assert!(has_usable_connection(&cfg));

        // Clear all keys: ollama still passes because its base_url is localhost.
        let mut keys_cleared = default_config();
        for c in keys_cleared.connections.values_mut() {
            c.api_key.clear();
        }
        assert!(has_usable_connection(&keys_cleared), "localhost connection is usable without a key");

        // Clear keys AND change ollama's base_url to a non-local one → genuinely first-run.
        let mut truly_empty = default_config();
        for c in truly_empty.connections.values_mut() {
            c.api_key.clear();
            c.base_url = "https://api.example.com".into();
        }
        assert!(!has_usable_connection(&truly_empty));
    }

    #[test]
    fn connection_is_usable_rules() {
        use crate::config::Driver;
        let mut c = Connection {
            driver: Driver::Openai, base_url: "https://api.example.com".into(),
            api_key: String::new(), model: "m".into(), max_tokens: None,
            batch_dialogue_limit: None, timeout: None, connect_timeout: None,
            concurrency: None, thinking_enabled: None, thinking_budget: None,
            thinking_glossary_budget: None, thinking_glossary_norm_budget: None,
            web_search: None,
        };
        // No key, non-local URL → not usable.
        assert!(!connection_is_usable(&c));
        // Non-empty key → usable.
        c.api_key = "sk-test".into();
        assert!(connection_is_usable(&c));
        // Key cleared, localhost URL → usable.
        c.api_key.clear();
        c.base_url = "http://localhost:11434".into();
        assert!(connection_is_usable(&c));
        // 127.0.0.1 → usable.
        c.base_url = "http://127.0.0.1:11434".into();
        assert!(connection_is_usable(&c));
    }
}
