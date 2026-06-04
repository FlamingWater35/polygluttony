//! Tauri command handlers — the entire surface the webview can call. Each
//! command is a thin wrapper that delegates to the engine modules.

pub mod connections;
pub use connections::*;

use crate::models::AppInfo;

/// Returns app/core metadata. Used by the webview as a startup health check for
/// the IPC bridge.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
