//! Persisted verification results: the future Verify view reads these without
//! re-running anything. Stored in `verification.json` app-data.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::error::{AppError, AppResult};
use crate::events::FileResult;

const STORE_FILE: &str = "verification.json";
const STORE_KEY: &str = "folders";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderVerification {
    pub timestamp: i64,
    pub files: BTreeMap<String, FileResult>,
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> AppResult<BTreeMap<String, FolderVerification>> {
    let store = app.store(STORE_FILE).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(store
        .get(STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

pub fn save_folder<R: Runtime>(
    app: &AppHandle<R>,
    folder: &str,
    results: &[FileResult],
    timestamp: i64,
) -> AppResult<()> {
    let store = app.store(STORE_FILE).map_err(|e| AppError::Other(e.to_string()))?;
    let mut all = load(app)?;
    let entry = all.entry(folder.to_string()).or_default();
    entry.timestamp = timestamp;
    for r in results {
        entry.files.insert(r.file.clone(), r.clone());
    }
    store.set(STORE_KEY, serde_json::to_value(&all)?);
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}
