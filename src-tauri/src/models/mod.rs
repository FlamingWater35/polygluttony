//! Shared data types surfaced to the webview. TypeScript bindings are generated
//! by `ts-rs` into `src/types/generated/` (run `cargo test` to regenerate).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::projects::FolderPrefs;
use crate::config::Driver;
use crate::glossary::world_detector::WorldType;

pub mod language_pair;

/// Basic application/core metadata. Doubles as an IPC health check.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

/// One row in the Connections list (no secret material).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ConnectionSummary {
    pub name: String,
    pub driver: Driver,
    pub has_key: bool,
}

/// O1 — the Connections list view-model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ConnectionsView {
    pub connections: Vec<ConnectionSummary>,
    pub active: String,
    pub personalization: Option<String>,
}

/// O5 — result of a Test.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct TestResult {
    pub ok: bool,
    pub model: String,
    pub detected_driver: Option<Driver>,
    pub message: String,
}

/// O21 — first-run check.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct FirstRunStatus {
    pub has_usable_connection: bool,
}

/// One discovered source file.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct SourceFile {
    pub path: String,
    pub name: String,
    pub dialogue_count: u32,
    pub has_translation: bool,
}

/// Result of opening a folder (O6/O7/O8 bundled).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ProjectView {
    pub folder: String,
    pub files: Vec<SourceFile>,
    pub total_dialogue_lines: u32,
    pub detected_source_lang: Option<String>,
    pub detected_world: WorldType,
    pub prefs: FolderPrefs,
    pub supports_glossary: bool,
    /// Number of glossary terms in `glossary.json`, or `None` if no glossary exists.
    pub glossary_terms: Option<u32>,
}
