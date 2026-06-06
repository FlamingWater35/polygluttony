//! Glossary build pipeline (O10): the build orchestrator (`build_glossary`)
//! plus the `glossary_batches` slicer shared with the reference extractor.
//!
//! Port of `core/glossary_builder.py:build_from_files` (68-233) with the
//! UI-spec deviations: world type arrives pre-detected from the UI (the build
//! NEVER re-detects), reference terminology is loaded/extracted inline (O11),
//! and every incremental save merges with the existing glossary first — fixing
//! the Python data-loss window where `glossary_phase.py:104-105` wrote the
//! new-terms-only glossary over a prior on-disk one.
//!
//! ## Failure philosophy (hard requirement)
//! Batch failures NEVER abort the build. An auth error stops the *remaining*
//! batches (via the cancel token) but the build still finalizes: merge what
//! completed → save → `Done { aborted: true }`. User cancel takes the same
//! finalize path with `cancelled: true`. Partial glossary > no glossary. The
//! only `Error` event is a final-save IO failure.

use std::path::PathBuf;

use futures::stream::{FuturesOrdered, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::ass::{decode::decode_file, parse::parse_dialogues, tags::strip_for_text};
use crate::events::{GlossaryBuildSummary, GlossaryEvent, GlossaryPhase, LogLevel};
use crate::glossary::diff::GlossaryDiff;
use crate::glossary::io::{load_folder_glossary, save_folder_glossary};
use crate::glossary::model::Glossary;
use crate::glossary::{normalize, personalize, prompts, reference};
use crate::llm::error::LlmError;
use crate::llm::service::LlmService;
use crate::llm::LlmRequest;
use crate::models::language_pair::LanguagePair;
use crate::translation::parse_response;

/// Slice a cross-file line stream into batches of `limit × 0.7` lines
/// (`glossary_builder.py:136-138,235-241`; `reference_extractor.py:65-67,124-137`).
/// The 30% headroom leaves room for prompt overhead.
// also consumed by the reference extractor (reference.rs)
#[allow(dead_code)]
pub fn glossary_batches(lines: &[String], batch_limit: Option<u32>) -> Vec<String> {
    let limit = batch_limit.unwrap_or(crate::translation::batching::BATCH_LINE_LIMIT);
    let per = (((limit as f64) * 0.7) as usize).max(1);
    lines.chunks(per).map(|c| c.join("\n")).collect()
}

/// Everything O10 needs from the UI. The command layer (later task) assembles
/// this from the project state + Glossary view options.
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub struct BuildJob {
    pub folder: PathBuf,
    /// File NAMES relative to `folder` (the Project view's `selected_files`).
    pub files: Vec<String>,
    /// EFFECTIVE world (override ?? detected) — the build never re-detects.
    pub world_type: String,
    pub pair: LanguagePair,
    pub normalize: bool,
    pub personalize: bool,
    pub personalize_context: String,
    pub template_variant: Option<String>,
    pub batch_limit: Option<u32>,
    pub cancel: CancellationToken,
}

/// Existing terms always win; `new_terms` only fills gaps. Every save (the
/// incremental ones included) goes through this so a pre-existing glossary can
/// never be clobbered by a partial build.
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub(crate) fn merged_with_existing(existing: Option<&Glossary>, new_terms: &Glossary) -> Glossary {
    match existing {
        Some(e) => {
            let mut out = e.clone();
            out.merge_first_wins(new_terms);
            out
        }
        None => new_terms.clone(),
    }
}

/// After abort/cancel, batches still queued in the service fail with this
/// transport error — consequences of the stop, not causes worth recording.
fn is_cancel_noise(e: &LlmError) -> bool {
    matches!(e, LlmError::Transport(msg) if msg == "run cancelled")
}

async fn phase(tx: &mpsc::Sender<GlossaryEvent>, p: GlossaryPhase, detail: Option<String>) {
    let _ = tx.send(GlossaryEvent::Phase { phase: p, detail }).await;
}

async fn log(tx: &mpsc::Sender<GlossaryEvent>, level: LogLevel, message: String) {
    let _ = tx.send(GlossaryEvent::Log { level, message }).await;
}

/// O10 build orchestrator. Always emits `Done {summary}` when a result exists
/// (including aborted/cancelled/no-text); the only `Error` event is a
/// final-save IO failure (the last incremental save remains on disk then).
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub async fn build_glossary(
    job: BuildJob,
    svc: &LlmService,
    personalize_svc: Option<&LlmService>,
    tx: mpsc::Sender<GlossaryEvent>,
) {
    // Snapshot the pre-build glossary: merge target for every save + diff base.
    let existing = load_folder_glossary(&job.folder);

    // ── Loading: decode + parse + strip each selected file ─────────────────
    phase(&tx, GlossaryPhase::Loading, None).await;
    let mut all_lines: Vec<String> = Vec::new();
    let mut files_processed = 0u32;
    for name in &job.files {
        match decode_file(&job.folder.join(name)) {
            Ok(text) => {
                let dialogues = parse_dialogues(&text);
                if dialogues.is_empty() {
                    log(&tx, LogLevel::Warning, format!("no dialogue text in {name} — skipped"))
                        .await;
                    continue;
                }
                let n = dialogues.len();
                all_lines.extend(dialogues.iter().map(|d| strip_for_text(&d.text)));
                files_processed += 1;
                log(&tx, LogLevel::Info, format!("loaded {name} ({n} lines)")).await;
            }
            Err(e) => {
                log(&tx, LogLevel::Warning, format!("error loading {name}: {e} — skipped")).await;
            }
        }
    }

    if all_lines.is_empty() {
        // Nothing to extract from: report (never silently no-op) and write
        // nothing — `glossary_builder.py:121-126`.
        let result = merged_with_existing(existing.as_ref(), &Glossary::new(&job.world_type));
        let summary = GlossaryBuildSummary {
            world_type: job.world_type.clone(),
            files_processed,
            batches_processed: 0,
            batches_total: 0,
            terms_extracted: 0,
            terms_final: result.count() as u32,
            normalized: false,
            personalized: false,
            aborted: false,
            cancelled: false,
            errors: vec!["No text found in files".into()],
            diff: GlossaryDiff::compute(existing.as_ref(), &result),
        };
        let _ = tx.send(GlossaryEvent::Done { summary }).await;
        return;
    }

    // ── Reference: advisory English terminology (O11, logs for itself) ─────
    phase(&tx, GlossaryPhase::Reference, None).await;
    let reference_terms = reference::load_or_extract(&job.folder, svc, job.batch_limit, &tx).await;

    // ── Extracting: all batches through the LLM, one shared system prompt ──
    let batches = glossary_batches(&all_lines, job.batch_limit);
    let total = batches.len() as u32;
    phase(&tx, GlossaryPhase::Extracting, Some(format!("{total} batches"))).await;
    let _ = tx.send(GlossaryEvent::Progress { done: 0, total }).await;

    let system = prompts::extraction_prompt(
        &job.world_type,
        &job.pair,
        reference_terms.as_ref(),
        job.template_variant.as_deref(),
    );

    // FuturesOrdered, not tokio::spawn and not join_all: structured concurrency
    // (drop-cancellation, no 'static bound, no panic arm) with results yielded
    // IN BATCH ORDER as they complete — so the incremental merge+save keeps
    // flowing DURING the run instead of all landing at the end (crash safety).
    // The LlmService bounds the actual parallelism via its permits.
    let mut futs: FuturesOrdered<_> = batches
        .iter()
        .map(|batch| {
            let req = LlmRequest {
                system: system.clone(),
                user: prompts::extraction_user_prompt(batch),
            };
            svc.request(req)
        })
        .collect();

    let mut new_terms = Glossary::new(&job.world_type);
    let mut errors: Vec<String> = Vec::new();
    let mut terms_extracted = 0u32;
    let mut batches_processed = 0u32;
    let mut aborted = false;
    let mut done = 0u32;

    while let Some(result) = futs.next().await {
        done += 1;
        let n = done;
        match result {
            Ok(resp) => {
                // Unparseable 2xx = empty glossary but still "processed"
                // (Python parity: parse failure is logged, not an error).
                let batch_glossary = match parse_response::extract_object(&resp.text) {
                    Ok(v) => Glossary::from_terms_value(&v, &job.world_type),
                    Err(e) => {
                        log(
                            &tx,
                            LogLevel::Warning,
                            format!("batch {n}/{total}: unparseable response ({e})"),
                        )
                        .await;
                        Glossary::new(&job.world_type)
                    }
                };
                // Per-batch count BEFORE cross-batch dedupe (Python parity).
                let count = batch_glossary.count() as u32;
                terms_extracted += count;
                new_terms.merge_first_wins(&batch_glossary);
                batches_processed += 1;
                // Crash-safe incremental save: always merged-with-existing
                // (the Python bug wrote new-terms-only), never an empty file.
                let snapshot = merged_with_existing(existing.as_ref(), &new_terms);
                if !snapshot.is_empty() {
                    if let Err(e) = save_folder_glossary(&job.folder, &snapshot) {
                        log(&tx, LogLevel::Warning, format!("incremental save failed: {e}")).await;
                    }
                }
                log(&tx, LogLevel::Info, format!("batch {n}/{total}: {count} terms")).await;
            }
            Err(e) if e.is_auth() && !aborted => {
                // Retrying won't fix credentials: stop the REMAINING batches,
                // but keep consuming results — completed work is still merged
                // and saved below (hard requirement: partial > none).
                aborted = true;
                job.cancel.cancel();
                errors.push(format!(
                    "batch {n}/{total}: auth error, remaining batches stopped ({e})"
                ));
            }
            Err(e) => {
                let noise = (aborted || job.cancel.is_cancelled()) && is_cancel_noise(&e);
                if !noise {
                    errors.push(format!("batch {n}/{total} failed: {e}"));
                }
            }
        }
        let _ = tx.send(GlossaryEvent::Progress { done: n, total }).await;
    }

    let cancelled = job.cancel.is_cancelled() && !aborted;
    new_terms.deduplicate();

    // ── Normalizing: NEW terms only, before the merge (build step 6) ────────
    let mut normalized = false;
    if job.normalize && !aborted && !cancelled && !new_terms.is_empty() {
        phase(
            &tx,
            GlossaryPhase::Normalizing,
            Some(format!("{} new terms", new_terms.count())),
        )
        .await;
        new_terms = normalize::normalize_pass(svc, &new_terms, &tx).await;
        normalized = true;
    }

    // ── Merge: existing terms win (build step 7) ────────────────────────────
    let mut result = merged_with_existing(existing.as_ref(), &new_terms);

    // ── Personalizing: one call on the web-capable connection (step 8) ──────
    let mut personalized = false;
    if job.personalize && !aborted && !cancelled && !result.is_empty() {
        if let Some(p_svc) = personalize_svc {
            phase(&tx, GlossaryPhase::Personalizing, None).await;
            match personalize::personalize_pass(p_svc, &result, &job.personalize_context).await {
                Ok(g) => {
                    result = g;
                    personalized = true;
                }
                Err(reason) => errors.push(reason),
            }
        }
    }

    // ── Saving: final write; empty result writes nothing, ever ──────────────
    phase(&tx, GlossaryPhase::Saving, None).await;
    if !result.is_empty() {
        if let Err(e) = save_folder_glossary(&job.folder, &result) {
            // The ONLY Error event in the build. The last incremental save
            // remains on disk.
            let _ = tx
                .send(GlossaryEvent::Error { message: format!("could not save glossary: {e}") })
                .await;
            return;
        }
    }

    let summary = GlossaryBuildSummary {
        world_type: job.world_type,
        files_processed,
        batches_processed,
        batches_total: total,
        terms_extracted,
        terms_final: result.count() as u32,
        normalized,
        personalized,
        aborted,
        cancelled,
        errors,
        diff: GlossaryDiff::compute(existing.as_ref(), &result),
    };
    let _ = tx.send(GlossaryEvent::Done { summary }).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("line {i}")).collect()
    }

    #[test]
    fn batches_slice_at_seventy_percent() {
        // limit 10 → 7 lines per batch → 15 lines = 7 + 7 + 1.
        let b = glossary_batches(&lines(15), Some(10));
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].lines().count(), 7);
        assert_eq!(b[2], "line 14");
    }

    #[test]
    fn batches_floor_at_one_line_and_default_limit() {
        let b = glossary_batches(&lines(3), Some(1)); // 0.7 → floor 1
        assert_eq!(b.len(), 3);
        // Default = BATCH_LINE_LIMIT (260) → 182 per batch.
        let b = glossary_batches(&lines(183), None);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].lines().count(), 182);
    }

    #[test]
    fn empty_lines_give_no_batches() {
        assert!(glossary_batches(&[], Some(10)).is_empty());
    }

    // ── O10 orchestrator tests ──────────────────────────────────────────────

    use crate::events::GlossaryEvent;
    use crate::glossary::io::load_folder_glossary;
    use crate::glossary::model::Glossary;
    use crate::llm::error::LlmError;
    use crate::llm::service::LlmService;
    use crate::llm::test_support::ScriptedDriver;
    use crate::models::language_pair::LanguagePair;
    use std::path::Path;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    fn svc1(driver: Arc<ScriptedDriver>, cancel: CancellationToken) -> LlmService {
        let (tx, _rx) = tokio::sync::mpsc::channel(64);
        // cap 1 → batches hit the driver in batch order (deterministic scripts).
        LlmService::new(driver, 1, cancel, tx)
    }

    fn write_ass(dir: &Path, name: &str, lines: &[&str]) {
        let mut content = String::from(
            "[Script Info]\nTitle: t\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );
        for (i, l) in lines.iter().enumerate() {
            content.push_str(&format!(
                "Dialogue: 0,0:00:0{i}.00,0:00:0{}.00,Default,,0,0,0,,{l}\n",
                i + 1
            ));
        }
        std::fs::write(dir.join(name), content).unwrap();
    }

    fn job(dir: &Path, files: Vec<String>, cancel: CancellationToken) -> BuildJob {
        BuildJob {
            folder: dir.to_path_buf(),
            files,
            world_type: "xianxia".into(),
            pair: LanguagePair::from_codes("zh", "en").unwrap(),
            normalize: false,
            personalize: false,
            personalize_context: String::new(),
            template_variant: None,
            batch_limit: Some(2), // ×0.7 → 1 line per batch
            cancel,
        }
    }

    /// Drain the channel after build returns; the Done summary must be last.
    async fn run_and_collect(
        job: BuildJob,
        svc: &LlmService,
        p_svc: Option<&LlmService>,
    ) -> (Vec<GlossaryEvent>, crate::events::GlossaryBuildSummary) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        build_glossary(job, svc, p_svc, tx).await;
        let mut events = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            events.push(ev);
        }
        let summary = events
            .iter()
            .rev()
            .find_map(|e| match e {
                GlossaryEvent::Done { summary } => Some(summary.clone()),
                _ => None,
            })
            .expect("build must emit Done");
        (events, summary)
    }

    #[tokio::test(start_paused = true)]
    async fn happy_path_merges_first_wins_in_index_order_and_saves() {
        let dir = tempfile::tempdir().unwrap();
        write_ass(dir.path(), "e1.ass", &["修仙第一句", "修仙第二句"]); // 2 lines → 2 batches
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![
            Ok(r#"{"terms":{"characters":{"林动":"Lin Dong"}}}"#.into()),
            // Batch 2 re-extracts 林动 with a different value — batch 1 wins.
            Ok(r#"{"characters":{"林动":"DUPE"},"locations":{"青阳镇":"Qingyang Town"}}"#.into()),
        ]);
        let svc = svc1(d, cancel.clone());
        let (_events, s) =
            run_and_collect(job(dir.path(), vec!["e1.ass".into()], cancel), &svc, None).await;

        assert_eq!(s.batches_total, 2);
        assert_eq!(s.batches_processed, 2);
        assert_eq!(s.files_processed, 1);
        assert_eq!(s.terms_extracted, 3); // 1 + 2, pre cross-batch dedupe
        assert_eq!(s.terms_final, 2);
        assert!(!s.aborted && !s.cancelled && s.errors.is_empty());
        assert_eq!(s.world_type, "xianxia");
        assert_eq!(s.diff.total_added, 2);

        let saved = load_folder_glossary(dir.path()).unwrap();
        assert_eq!(saved.characters.get("林动").unwrap(), "Lin Dong"); // first wins
        assert_eq!(saved.locations.get("青阳镇").unwrap(), "Qingyang Town");
        assert_eq!(saved.world_type, "xianxia");
    }

    #[tokio::test(start_paused = true)]
    async fn batch_failure_is_partial_never_fatal() {
        let dir = tempfile::tempdir().unwrap();
        write_ass(dir.path(), "e1.ass", &["一", "二"]);
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![
            Err(LlmError::Http { status: 400, body: "bad request".into() }), // batch 1: non-retryable, one call
            Ok(r#"{"characters":{"林动":"Lin Dong"}}"#.into()),              // batch 2 ok
        ]);
        let svc = svc1(d, cancel.clone());
        let (_events, s) =
            run_and_collect(job(dir.path(), vec!["e1.ass".into()], cancel), &svc, None).await;

        assert!(!s.aborted, "non-auth failures must not abort");
        assert_eq!(s.batches_processed, 1);
        assert_eq!(s.errors.len(), 1);
        assert!(s.errors[0].contains("batch 1"));
        assert_eq!(s.terms_final, 1); // half a glossary > no glossary
        assert!(load_folder_glossary(dir.path()).is_some());
    }

    #[tokio::test(start_paused = true)]
    async fn auth_error_aborts_remaining_but_keeps_partial_and_existing() {
        let dir = tempfile::tempdir().unwrap();
        // Existing glossary on disk — must survive verbatim (crash-safety fix).
        let mut existing = Glossary::new("xianxia");
        existing.characters.insert("应欢欢".into(), "Ying Huanhuan".into());
        crate::glossary::io::save_folder_glossary(dir.path(), &existing).unwrap();

        write_ass(dir.path(), "e1.ass", &["一", "二"]);
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![
            Ok(r#"{"characters":{"林动":"Lin Dong"}}"#.into()), // batch 1 ok
            Err(LlmError::Http { status: 401, body: "bad key".into() }), // batch 2: auth, no retry
        ]);
        let svc = svc1(d, cancel.clone());
        let (_events, s) =
            run_and_collect(job(dir.path(), vec!["e1.ass".into()], cancel), &svc, None).await;

        assert!(s.aborted);
        assert!(!s.cancelled);
        assert_eq!(s.errors.len(), 1, "only the auth error, no cancel noise: {:?}", s.errors);
        assert_eq!(s.batches_processed, 1);
        let saved = load_folder_glossary(dir.path()).unwrap();
        assert_eq!(saved.characters.get("应欢欢").unwrap(), "Ying Huanhuan"); // existing preserved
        assert_eq!(saved.characters.get("林动").unwrap(), "Lin Dong"); // partial kept
        assert_eq!(s.diff.total_added, 1); // diff vs pre-build state
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_finalizes_with_partial_counts() {
        let dir = tempfile::tempdir().unwrap();
        write_ass(dir.path(), "e1.ass", &["一", "二"]);
        let cancel = CancellationToken::new();
        cancel.cancel(); // user cancelled immediately
        let d = ScriptedDriver::new(vec![]); // service errors before reaching the driver
        let svc = svc1(d, cancel.clone());
        let (_events, s) =
            run_and_collect(job(dir.path(), vec!["e1.ass".into()], cancel), &svc, None).await;
        assert!(s.cancelled && !s.aborted);
        assert_eq!(s.batches_processed, 0);
        assert!(s.errors.is_empty(), "cancel noise suppressed: {:?}", s.errors);
        assert!(load_folder_glossary(dir.path()).is_none(), "nothing written");
    }

    #[tokio::test(start_paused = true)]
    async fn no_text_emits_done_with_error_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.ass"), "not an ass file").unwrap();
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![]);
        let svc = svc1(d, cancel.clone());
        let (_events, s) = run_and_collect(
            job(dir.path(), vec!["bad.ass".into(), "missing.ass".into()], cancel),
            &svc,
            None,
        )
        .await;
        assert_eq!(s.files_processed, 0);
        assert_eq!(s.errors, vec!["No text found in files".to_string()]);
        assert!(load_folder_glossary(dir.path()).is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn normalize_runs_on_new_terms_and_merge_keeps_existing_wins() {
        let dir = tempfile::tempdir().unwrap();
        let mut existing = Glossary::new("xianxia");
        existing.characters.insert("林动".into(), "EXISTING WINS".into());
        crate::glossary::io::save_folder_glossary(dir.path(), &existing).unwrap();

        write_ass(dir.path(), "e1.ass", &["一"]); // 1 batch
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![
            // extraction: two new chars (one colliding with existing)
            Ok(r#"{"characters":{"林动":"ignored","应欢欢":"ying huanhuan"}}"#.into()),
            // normalize characters (the only non-empty category of NEW terms)
            Ok(r#"{"林动":"still ignored","应欢欢":"Ying Huanhuan"}"#.into()),
        ]);
        let svc = svc1(d, cancel.clone());
        let mut j = job(dir.path(), vec!["e1.ass".into()], cancel);
        j.normalize = true;
        let (_events, s) = run_and_collect(j, &svc, None).await;
        assert!(s.normalized);
        let saved = load_folder_glossary(dir.path()).unwrap();
        assert_eq!(saved.characters.get("林动").unwrap(), "EXISTING WINS");
        assert_eq!(saved.characters.get("应欢欢").unwrap(), "Ying Huanhuan");
    }

    #[tokio::test(start_paused = true)]
    async fn personalize_failure_keeps_glossary_and_records_error() {
        let dir = tempfile::tempdir().unwrap();
        write_ass(dir.path(), "e1.ass", &["一"]);
        let cancel = CancellationToken::new();
        let d = ScriptedDriver::new(vec![Ok(r#"{"characters":{"林动":"Lin Dong"}}"#.into())]);
        let pd = ScriptedDriver::new(vec![Ok("no json here".into())]);
        let svc = svc1(d, cancel.clone());
        let p_svc = svc1(pd, CancellationToken::new());
        let mut j = job(dir.path(), vec!["e1.ass".into()], cancel);
        j.personalize = true;
        let (_events, s) = run_and_collect(j, &svc, Some(&p_svc)).await;
        assert!(!s.personalized);
        assert_eq!(s.errors.len(), 1);
        assert!(s.errors[0].contains("personalize"));
        assert_eq!(s.terms_final, 1); // glossary intact
    }
}
