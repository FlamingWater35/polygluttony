//! Provider presets, curated model lists, and the seeded default config.
//! Ports `config/settings.py:get_default_config` adapted to the 5-provider list.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{AppConfig, Connection, Driver};

/// A provider preset shown in the Connections "Provider" dropdown.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Preset {
    pub key: String,
    pub label: String,
    /// `None` = auto-detect on Test (Custom).
    pub driver: Option<Driver>,
    pub base_url: String,
    pub model: String,
    /// Curated model suggestions (live `/models` fetch supplements these).
    pub models: Vec<String>,
}

fn conn(driver: Driver, base_url: &str, model: &str, api_key: &str) -> Connection {
    Connection {
        driver,
        base_url: base_url.to_string(),
        api_key: api_key.to_string(),
        model: model.to_string(),
        max_tokens: Some(16000),
        batch_dialogue_limit: Some(100),
        timeout: Some(120),
        connect_timeout: Some(10),
        concurrency: Some(5),
        thinking_enabled: None,
        thinking_budget: None,
        web_search: None,
        thinking_glossary_norm_budget: None,
    }
}

const ANTHROPIC_MODELS: &[&str] =
    &["claude-opus-4-5", "claude-sonnet-4-5", "claude-haiku-4-5"];
const OPENAI_MODELS: &[&str] = &["gpt-5.2", "gpt-5.1", "gpt-5", "gpt-4.1", "o4-mini"];
const GOOGLE_MODELS: &[&str] = &["gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.0-flash"];
const OLLAMA_MODELS: &[&str] = &["llama3.1", "qwen2.5"];

fn models(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

const ANTHROPIC_BASE: &str = "https://api.anthropic.com";
const GOOGLE_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/openai/";
const OPENAI_BASE: &str = "https://api.openai.com/v1";
const OLLAMA_BASE: &str = "http://localhost:11434/v1";

/// The provider preset table for the UI dropdown.
pub fn presets() -> Vec<Preset> {
    vec![
        Preset { key: "anthropic".into(), label: "Anthropic".into(),
            driver: Some(Driver::Anthropic), base_url: ANTHROPIC_BASE.into(),
            model: "claude-opus-4-5".into(), models: models(ANTHROPIC_MODELS) },
        Preset { key: "google".into(), label: "Google (Gemini)".into(),
            driver: Some(Driver::Openai), base_url: GOOGLE_BASE.into(),
            model: "gemini-2.5-pro".into(), models: models(GOOGLE_MODELS) },
        Preset { key: "openai".into(), label: "OpenAI".into(),
            driver: Some(Driver::OpenaiResponses), base_url: OPENAI_BASE.into(),
            model: "gpt-5.2".into(), models: models(OPENAI_MODELS) },
        Preset { key: "ollama".into(), label: "Ollama (local)".into(),
            driver: Some(Driver::Openai), base_url: OLLAMA_BASE.into(),
            model: "llama3.1".into(), models: models(OLLAMA_MODELS) },
        Preset { key: "custom".into(), label: "Custom".into(),
            driver: None, base_url: String::new(),
            model: String::new(), models: vec![] },
    ]
}

/// The default `AppConfig` seeded on first run.
pub fn default_config() -> AppConfig {
    let mut connections: BTreeMap<String, Connection> = BTreeMap::new();
    connections.insert("anthropic".into(),
        conn(Driver::Anthropic, ANTHROPIC_BASE, "claude-opus-4-5", ""));
    connections.insert("google".into(),
        conn(Driver::Openai, GOOGLE_BASE, "gemini-2.5-pro", ""));
    connections.insert("openai".into(),
        conn(Driver::OpenaiResponses, OPENAI_BASE, "gpt-5.2", ""));
    connections.insert("ollama".into(),
        conn(Driver::Openai, OLLAMA_BASE, "llama3.1", "ollama"));

    AppConfig {
        default_source: "zh".into(),
        default_target: "en".into(),
        active_connection: "anthropic".into(),
        personalization_model: Some("openai".into()),
        default_workdir: None,
        connections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Driver;

    #[test]
    fn default_config_has_seeded_connections() {
        let cfg = default_config();
        assert_eq!(cfg.default_source, "zh");
        assert_eq!(cfg.default_target, "en");
        assert_eq!(cfg.active_connection, "anthropic");
        assert_eq!(cfg.personalization_model.as_deref(), Some("openai"));
        for name in ["anthropic", "google", "openai", "ollama"] {
            assert!(cfg.connections.contains_key(name), "missing seed {name}");
        }
        // Cloud seeds ship without a key; ollama carries a placeholder.
        assert_eq!(cfg.connections["anthropic"].api_key, "");
        assert_eq!(cfg.connections["ollama"].api_key, "ollama");
    }

    #[test]
    fn presets_cover_five_providers_with_drivers() {
        let presets = presets();
        let keys: Vec<&str> = presets.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, ["anthropic", "google", "openai", "ollama", "custom"]);
        assert_eq!(presets[0].driver, Some(Driver::Anthropic));   // anthropic
        assert_eq!(presets[1].driver, Some(Driver::Openai));      // google
        assert_eq!(presets[2].driver, Some(Driver::OpenaiResponses)); // openai
        assert_eq!(presets[3].driver, Some(Driver::Openai));      // ollama
        assert_eq!(presets[4].driver, None);                      // custom = auto-detect
        assert!(presets[0].models.iter().any(|m| m.starts_with("claude")));
    }
}
