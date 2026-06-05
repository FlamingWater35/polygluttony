//! polygluttony — Tauri shell.
//!
//! This crate hosts both the desktop shell (Tauri commands + event emission)
//! and the translation engine. The engine modules below are pure Rust and exist
//! solely to serve the webview through the commands in [`commands`].

// Scaffolding: engine types/events/config are defined ahead of their use sites.
// Remove once the engine modules are implemented.
#![allow(dead_code)]

mod commands;
mod config;
mod error;
mod events;
mod models;

// Translation engine. Ported from the original Python `subs_translator` package;
// implementation lands here incrementally.
mod ass;
mod utils;
mod glossary;
mod llm;
mod translation;
mod validation;

/// Entry point invoked from `main.rs` (and the mobile entry point).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_connections,
            commands::read_connection,
            commands::save_connection,
            commands::delete_connection,
            commands::rename_connection,
            commands::set_active_connection,
            commands::set_personalization_connection,
            commands::first_run_status,
            commands::list_presets,
            commands::test_connection,
            commands::list_models,
            commands::list_languages,
            commands::list_recents,
            commands::remove_recent,
            commands::clear_recents,
            commands::save_folder_prefs,
            commands::set_default_languages,
            commands::open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
