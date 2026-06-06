//! Events emitted from the Rust core to the webview during long-running
//! pipeline operations. The frontend subscribes via `onBackendEvent` (see
//! `src/lib/ipc.ts`).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Event channel names. Keep in sync with the frontend listeners.
pub mod names {
    // Step 4 (Glossary) and Step 5 (Verify/Translate views) will consume these.
    #[allow(dead_code)]
    pub const TRANSLATION_PROGRESS: &str = "translation://progress";
    // Superseded by GLOSSARY_EVENT (step 4 uses the tagged-union channel
    // instead); kept here in case external tooling or future steps need it.
    #[allow(dead_code)]
    pub const GLOSSARY_PROGRESS: &str = "glossary://progress";
    #[allow(dead_code)]
    pub const VERIFICATION_PROGRESS: &str = "verification://progress";
    #[allow(dead_code)]
    pub const LOG: &str = "core://log";
}

/// Generic progress payload for a single unit of work (e.g. a file).
// Step 4 (Glossary) will emit these on the progress channels.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ProgressEvent {
    /// Identifier of the unit of work, typically the file name.
    pub id: String,
    /// Completed steps so far.
    pub completed: u32,
    /// Total steps, when known.
    pub total: Option<u32>,
    /// Optional human-readable status message.
    pub message: Option<String>,
}

/// Channel for all step-3 run events.
pub const TRANSLATION_EVENT: &str = "translation://event";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum FileStateKind {
    Pending,
    Translating,
    Retranslating,
    Cleanup,
    Verifying,
    Done,
    Warning,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum LogPhase {
    Parse,
    Batch,
    Cleanup,
    Verify,
    Llm,
    Error,
    Retranslate,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct VerifyIssue {
    pub line_id: u32,
    pub source: String,
    pub translation: String,
    pub issue_type: String,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct FileResult {
    /// File *name* relative to the run folder, never an absolute path.
    pub file: String,
    pub success: bool,
    pub total_lines: u32,
    pub translated_lines: u32,
    pub has_warnings: bool,
    pub issues: Vec<VerifyIssue>,
    pub output_path: Option<String>,
}

/// Everything the UI hears during a run, on `TRANSLATION_EVENT`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum RunEvent {
    State { file: String, state: FileStateKind, detail: Option<String> },
    Progress { file: String, translated: u32, total: u32, batch: u32, total_batches: u32, retries: u32 },
    Log { file: Option<String>, level: LogLevel, phase: LogPhase, message: String },
    FileDone { file: String, has_warnings: bool },
    Error { file: String, message: String },
    RunFinished { results: Vec<FileResult> },
}

/// Channel for all step-4 glossary events.
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub const GLOSSARY_EVENT: &str = "glossary://event";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum GlossaryPhase {
    Loading,
    Reference,
    Extracting,
    Normalizing,
    Personalizing,
    Saving,
}

/// Terminal payload of a glossary build. `aborted` = mid-build fatal (auth)
/// — partial results were still merged and saved; `cancelled` = user cancel,
/// same partial-save semantics. Partial > none (hard requirement).
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GlossaryBuildSummary {
    pub world_type: String,
    pub files_processed: u32,
    pub batches_processed: u32,
    pub batches_total: u32,
    pub terms_extracted: u32,
    pub terms_final: u32,
    pub normalized: bool,
    pub personalized: bool,
    pub aborted: bool,
    pub cancelled: bool,
    pub errors: Vec<String>,
    /// Diff vs the glossary as it stood before the build.
    pub diff: crate::glossary::diff::GlossaryDiff,
}

/// Everything the UI hears on `GLOSSARY_EVENT`. `Error` is terminal-without-
/// result (e.g. final save IO failure); pre-flight failures surface as command
/// errors, not events. `FileChanged` comes from the O15 watcher.
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum GlossaryEvent {
    Phase { phase: GlossaryPhase, detail: Option<String> },
    Progress { done: u32, total: u32 },
    Log { level: LogLevel, message: String },
    Done { summary: GlossaryBuildSummary },
    Error { message: String },
    FileChanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glossary_event_serializes_with_kind_tag() {
        let ev = GlossaryEvent::Phase { phase: GlossaryPhase::Extracting, detail: None };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["kind"], "phase");
        assert_eq!(v["phase"], "extracting");

        let done = GlossaryEvent::Done {
            summary: GlossaryBuildSummary {
                world_type: "xianxia".into(),
                files_processed: 1,
                batches_processed: 2,
                batches_total: 2,
                terms_extracted: 10,
                terms_final: 8,
                normalized: true,
                personalized: false,
                aborted: false,
                cancelled: false,
                errors: vec![],
                diff: crate::glossary::diff::GlossaryDiff::compute(
                    None,
                    &crate::glossary::model::Glossary::new("xianxia"),
                ),
            },
        };
        let v = serde_json::to_value(&done).unwrap();
        assert_eq!(v["kind"], "done");
        assert_eq!(v["summary"]["terms_final"], 8);
    }
}
