//! O16/O17 commands. Thin wrappers — the engine lives in `translation::run`.

use tauri::AppHandle;

use crate::config::projects::Tone;
use crate::error::AppResult;
use crate::translation::run::{self, StartArgs};

// JS-side key casing: Tauri v2's #[tauri::command] macro converts every Rust
// snake_case parameter name to lowerCamelCase before matching against the
// JSON payload — see tauri-macros/src/command/wrapper.rs lines 505-508
// (`ArgumentCase::Camel => { key = key.to_lower_camel_case(); }`), with
// `ArgumentCase::Camel` as the hard-coded default (line 51).
// Therefore JS MUST send camelCase keys: `sourceLang`, `targetLang`, etc.
// (Override with `#[tauri::command(rename_all = "snake_case")]` if desired.)
#[tauri::command]
pub async fn start_translation(
    app: AppHandle,
    folder: String,
    files: Vec<String>,
    tone: Tone,
    source_lang: String,
    target_lang: String,
    now: i64,
) -> AppResult<()> {
    run::start(app, StartArgs { folder, files, tone, source_lang, target_lang, now }).await
}

#[tauri::command]
pub async fn cancel_translation(app: AppHandle) -> AppResult<()> {
    run::cancel(app).await
}
