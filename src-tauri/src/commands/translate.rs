//! O16/O17 commands. Thin wrappers — the engine lives in `translation::run`.

use tauri::AppHandle;

use crate::config::projects::Tone;
use crate::error::AppResult;
use crate::translation::run::{self, StartArgs};

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
