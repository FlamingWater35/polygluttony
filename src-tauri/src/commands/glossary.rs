//! Glossary commands (O9–O15). Thin wrappers — the engine lives in `glossary::*`.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::config::store as config_store;
use crate::error::{AppError, AppResult};
use crate::glossary::diff::GlossaryDiff;
use crate::glossary::io::{load_folder_glossary, save_folder_glossary};
use crate::glossary::model::GlossaryDoc;
use crate::glossary::normalize::{normalize_pass, NormalizeReview};
use crate::glossary::reference::{self, ReferenceStatus, ReferenceSummary};
use crate::glossary::run::{self, GlossaryOpKind, StartArgs};
use crate::glossary::watch::{self, GlossaryWatchState};
use crate::glossary::world_detector::WorldType;

/// O9 — None when no glossary.json exists (or it's unreadable: lenient).
#[tauri::command]
pub fn load_glossary(folder: String) -> Option<GlossaryDoc> {
    load_folder_glossary(&PathBuf::from(folder)).map(|g| GlossaryDoc::from(&g))
}

/// O14 — atomic write; the UI auto-saves every term edit.
#[tauri::command]
pub fn save_glossary(folder: String, doc: GlossaryDoc) -> AppResult<()> {
    save_folder_glossary(&PathBuf::from(folder), &doc.into_glossary())
}

// JS sends camelCase keys (worldType, sourceLang, …) — the macro converts; see
// commands/translate.rs:9-15.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn start_glossary_build(
    app: AppHandle,
    folder: String,
    files: Vec<String>,
    world_type: WorldType,
    source_lang: String,
    target_lang: String,
    normalize: bool,
    personalize: bool,
    personalize_context: String,
) -> AppResult<()> {
    run::start(
        app,
        StartArgs {
            folder,
            files,
            world_type,
            source_lang,
            target_lang,
            normalize,
            personalize,
            personalize_context,
        },
    )
    .await
}

/// Cancels the active glossary op (build / normalize / import).
#[tauri::command]
pub async fn cancel_glossary_build(app: AppHandle) -> AppResult<()> {
    run::cancel(app).await
}

/// O12 — returns the review WITHOUT saving; the UI saves on accept via
/// `save_glossary`. Claims the op slot (exclusive with build/translation).
#[tauri::command]
pub async fn normalize_glossary(app: AppHandle, folder: String) -> AppResult<NormalizeReview> {
    let dir = PathBuf::from(&folder);
    let original = load_folder_glossary(&dir)
        .ok_or_else(|| AppError::Other("no glossary to normalize".into()))?;
    if original.is_empty() {
        return Err(AppError::Other("glossary is empty".into()));
    }
    let cfg = config_store::load(&app)?;
    let conn =
        crate::translation::run::usable_connection(&cfg).ok_or(AppError::NoActiveConnection)?;
    let cancel = run::claim_slot(&app, GlossaryOpKind::Normalize).await?;
    // Slot released on every path below (the async block catches early returns).
    let result = async {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        run::spawn_forwarder(app.clone(), rx);
        let svc = run::service_for(&conn, cancel.clone(), tx.clone());
        let normalized = normalize_pass(&svc, &original, &tx).await;
        Ok(NormalizeReview {
            diff: GlossaryDiff::compute(Some(&original), &normalized),
            normalized: GlossaryDoc::from(&normalized),
        })
    }
    .await;
    run::release_slot(&app).await;
    result
}

/// O11 — extract reference terms from user-picked translated `.ass` files and
/// cache them; subsequent builds pick the cache up automatically.
#[tauri::command]
pub async fn import_reference_files(
    app: AppHandle,
    folder: String,
    paths: Vec<String>,
) -> AppResult<ReferenceSummary> {
    if paths.is_empty() {
        return Err(AppError::Other("no files chosen".into()));
    }
    let dir = PathBuf::from(&folder);
    let cfg = config_store::load(&app)?;
    let conn =
        crate::translation::run::usable_connection(&cfg).ok_or(AppError::NoActiveConnection)?;
    let cancel = run::claim_slot(&app, GlossaryOpKind::Import).await?;
    let result = async {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        run::spawn_forwarder(app.clone(), rx);
        let svc = run::service_for(&conn, cancel.clone(), tx.clone());
        let files: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let (terms, files_processed, errors) =
            reference::extract_from_files(&svc, &files, conn.batch_dialogue_limit, &tx).await;
        let count = terms.count() as u32;
        if count > 0 {
            reference::save_cache(&dir, &terms)?;
        }
        Ok(ReferenceSummary { count, files_processed, errors })
    }
    .await;
    run::release_slot(&app).await;
    result
}

#[tauri::command]
pub fn reference_status(folder: String) -> ReferenceStatus {
    reference::reference_status(&PathBuf::from(folder))
}

#[tauri::command]
pub fn clear_reference(folder: String) -> AppResult<()> {
    reference::clear_cache(&PathBuf::from(folder))
}

/// Plain file copy; the UI supplies `dest` from a save dialog.
#[tauri::command]
pub fn export_glossary(folder: String, dest: String) -> AppResult<()> {
    let src = PathBuf::from(folder).join("glossary.json");
    if !src.is_file() {
        return Err(AppError::Other("no glossary.json to export".into()));
    }
    std::fs::copy(src, dest)?;
    Ok(())
}

/// O15 — open glossary.json with the OS default editor.
#[tauri::command]
pub fn open_glossary_editor(app: AppHandle, folder: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let path = PathBuf::from(folder).join("glossary.json");
    if !path.is_file() {
        return Err(AppError::Other("no glossary.json yet".into()));
    }
    app.opener()
        .open_path(path.display().to_string(), None::<String>)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn watch_glossary(app: AppHandle, folder: String) -> AppResult<()> {
    let state = app.state::<GlossaryWatchState>();
    watch::watch(app.clone(), &state, &PathBuf::from(folder))
}

#[tauri::command]
pub fn unwatch_glossary(app: AppHandle) -> AppResult<()> {
    watch::unwatch(&app.state::<GlossaryWatchState>());
    Ok(())
}

/// Name of a usable web-capable personalization connection, else None —
/// powers the "Look up established names online" checkbox gating.
#[tauri::command]
pub fn personalization_status(app: AppHandle) -> AppResult<Option<String>> {
    let cfg = config_store::load(&app)?;
    Ok(run::web_capable_personalization(&cfg).map(|(name, _)| name))
}
