//! Reference terminology (O11): English terms lifted from already-translated
//! `.ass` files, injected into the extraction prompt for consistency. Ports
//! `glossary/reference_terminology.py` + `reference_loader.py` + the async LLM
//! extractor (`reference_extractor.py`).
//!
//! ## Cache-placement deviation from Python
//! Python (`reference_loader.py:63`) stores the cache at `ref_dir.parent()` —
//! which may be *above* the work folder when ref/ is at the parent or grandparent
//! level. We always place the cache at `{folder}/glossary-reference.json` (the
//! spec-blessed location). Consequence: Python-era caches that sit next to a
//! parent-level `ref/` are silently ignored, and sibling season folders no longer
//! share a single cache file.

use std::path::{Path, PathBuf};

use futures::future::join_all;
use tokio::sync::mpsc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ass::{decode::decode_file, parse::parse_dialogues, tags::strip_for_text};
use crate::error::AppResult;
use crate::events::{GlossaryEvent, LogLevel};
use crate::llm::service::LlmService;
use crate::llm::LlmRequest;
use crate::translation::parse_response;

// consumed by the async LLM extractor and O11 command (later step-4 tasks)
#[allow(dead_code)]
pub const CACHE_FILENAME: &str = "glossary-reference.json";

const CATEGORY_LABELS: [(&str, &str); 6] = [
    ("characters", "CHARACTER NAMES"),
    ("cultivation", "CULTIVATION LEVELS"),
    ("skills", "SKILLS"),
    ("locations", "LOCATIONS"),
    ("items", "ITEMS"),
    ("organizations", "ORGANIZATIONS"),
];

/// Six list-categories of English terms (no source mapping — guidance only).
// consumed by the async LLM extractor (later step-4 task)
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReferenceTerminology {
    #[serde(default)]
    pub characters: Vec<String>,
    #[serde(default)]
    pub cultivation: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub organizations: Vec<String>,
}

// all methods consumed by the async LLM extractor (later step-4 task)
#[allow(dead_code)]
impl ReferenceTerminology {
    fn category(&self, name: &str) -> &Vec<String> {
        match name {
            "characters" => &self.characters,
            "cultivation" => &self.cultivation,
            "skills" => &self.skills,
            "locations" => &self.locations,
            "items" => &self.items,
            "organizations" => &self.organizations,
            _ => unreachable!("unknown reference category: {name}"),
        }
    }

    fn category_mut(&mut self, name: &str) -> &mut Vec<String> {
        match name {
            "characters" => &mut self.characters,
            "cultivation" => &mut self.cultivation,
            "skills" => &mut self.skills,
            "locations" => &mut self.locations,
            "items" => &mut self.items,
            "organizations" => &mut self.organizations,
            _ => unreachable!("unknown reference category: {name}"),
        }
    }

    pub fn count(&self) -> usize {
        CATEGORY_LABELS.iter().map(|(c, _)| self.category(c).len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Case-insensitive append-merge (`reference_terminology.py:50-71`).
    pub fn merge(&mut self, other: &ReferenceTerminology) {
        for (c, _) in CATEGORY_LABELS {
            let target = self.category_mut(c);
            let mut seen: std::collections::HashSet<String> =
                target.iter().map(|t| t.to_lowercase()).collect();
            for term in other.category(c) {
                if seen.insert(term.to_lowercase()) {
                    target.push(term.clone());
                }
            }
        }
    }

    /// Order-preserving, case-insensitive in-category dedupe
    /// (`reference_terminology.py:73-91`).
    pub fn deduplicate(&mut self) {
        for (c, _) in CATEGORY_LABELS {
            let mut seen = std::collections::HashSet::new();
            self.category_mut(c).retain(|t| seen.insert(t.to_lowercase()));
        }
    }

    /// `CHARACTER NAMES: a, b` lines for the `{reference_terminology}`
    /// placeholder (`reference_terminology.py:26-48`).
    pub fn to_prompt_string(&self) -> String {
        CATEGORY_LABELS
            .iter()
            .filter(|(c, _)| !self.category(c).is_empty())
            .map(|(c, label)| format!("{label}: {}", self.category(c).join(", ")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Lenient parse of an extraction response `{category: [terms]}` — non-
    /// string entries and unknown keys dropped.
    pub fn from_value(v: &serde_json::Value) -> ReferenceTerminology {
        let mut t = ReferenceTerminology::default();
        if let Some(obj) = v.as_object() {
            for (c, _) in CATEGORY_LABELS {
                if let Some(arr) = obj.get(c).and_then(|x| x.as_array()) {
                    *t.category_mut(c) = arr
                        .iter()
                        .filter_map(|e| e.as_str().map(String::from))
                        .collect();
                }
            }
        }
        t
    }
}

/// Cache load: missing or corrupt ⇒ None. We deliberately do NOT delete a
/// corrupt cache (Python does) — the user may want to fix it by hand.
// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
pub fn load_cache(folder: &Path) -> Option<ReferenceTerminology> {
    let text = std::fs::read_to_string(folder.join(CACHE_FILENAME)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_cache(folder: &Path, t: &ReferenceTerminology) -> AppResult<()> {
    let json = serde_json::to_string_pretty(t)?;
    std::fs::write(folder.join(CACHE_FILENAME), json)?;
    Ok(())
}

/// Idempotent delete (missing file is fine).
pub fn clear_cache(folder: &Path) -> AppResult<()> {
    match std::fs::remove_file(folder.join(CACHE_FILENAME)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// `folder/ref` → `folder/../ref` → `folder/../../ref`
/// (`reference_loader.py:103-128`).
// consumed by the async LLM extractor (later step-4 task)
#[allow(dead_code)]
pub fn find_ref_dir(folder: &Path) -> Option<PathBuf> {
    let mut candidates = vec![folder.join("ref")];
    if let Some(p) = folder.parent() {
        candidates.push(p.join("ref"));
        if let Some(pp) = p.parent() {
            candidates.push(pp.join("ref"));
        }
    }
    candidates.into_iter().find(|c| c.is_dir())
}

/// Sorted `*.ass` files in a reference dir (`reference_loader.py:31-40`).
///
/// Deliberate improvement over Python: the extension check is
/// case-insensitive (`.ASS` matches), whereas Python's `glob("*.ass")` was
/// case-sensitive on Linux/macOS.
// consumed by the async LLM extractor (later step-4 task)
#[allow(dead_code)]
pub fn ref_ass_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("ass")))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum ReferenceSource {
    Cached,
    RefDir,
    None,
}

/// What the Import card chip shows. `count` = terms when cached, `.ass` file
/// count when only a ref/ dir exists.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ReferenceStatus {
    pub source: ReferenceSource,
    pub count: u32,
}

pub fn reference_status(folder: &Path) -> ReferenceStatus {
    if let Some(t) = load_cache(folder) {
        return ReferenceStatus { source: ReferenceSource::Cached, count: t.count() as u32 };
    }
    if let Some(dir) = find_ref_dir(folder) {
        let n = ref_ass_files(&dir).len();
        if n > 0 {
            return ReferenceStatus { source: ReferenceSource::RefDir, count: n as u32 };
        }
    }
    ReferenceStatus { source: ReferenceSource::None, count: 0 }
}

/// Result of an explicit Import (O11 picker path).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ReferenceSummary {
    pub count: u32,
    pub files_processed: u32,
    pub errors: Vec<String>,
}

/// Dialogue lines from reference `.ass` files; unparseable files are skipped
/// (`reference_extractor.py:100-122`).
fn collect_dialogue_lines(files: &[PathBuf]) -> (Vec<String>, u32) {
    let mut lines = Vec::new();
    let mut ok = 0u32;
    for f in files {
        let Ok(text) = decode_file(f) else { continue };
        let dialogues = parse_dialogues(&text);
        if dialogues.is_empty() {
            continue;
        }
        ok += 1;
        lines.extend(dialogues.iter().map(|d| strip_for_text(&d.text)));
    }
    (lines, ok)
}

/// Extract English reference terms from `.ass` files: batch at `limit × 0.7`
/// lines, parallel via `LlmService` (its retries/AIMD replace Python's
/// BatchManager), merge sorted by batch index, dedupe. Both LLM failures and
/// unparseable responses are recorded in `errors` and skipped — NEVER fatal.
/// Returns (terms, files_processed, errors).
pub async fn extract_from_files(
    svc: &LlmService,
    files: &[PathBuf],
    batch_limit: Option<u32>,
    tx: &mpsc::Sender<GlossaryEvent>,
) -> (ReferenceTerminology, u32, Vec<String>) {
    let (lines, files_ok) = collect_dialogue_lines(files);
    if lines.is_empty() {
        return (
            ReferenceTerminology::default(),
            files_ok,
            vec!["no dialogue text in reference files".into()],
        );
    }
    let batches = crate::glossary::build::glossary_batches(&lines, batch_limit);
    let total = batches.len() as u32;
    let _ = tx.send(GlossaryEvent::Progress { done: 0, total }).await;

    // Drive all batch futures concurrently; join_all preserves input order so
    // merge order = batch index order (reference_extractor.py:86). Borrowing
    // `svc` directly — no Arc clone per task; drop-cancellation is free.
    let futures: Vec<_> = batches
        .into_iter()
        .map(|batch| {
            let req = LlmRequest {
                system: crate::glossary::prompts::REFERENCE_EXTRACT.to_string(),
                user: crate::glossary::prompts::extraction_user_prompt(&batch),
            };
            svc.request(req)
        })
        .collect();
    let results = join_all(futures).await;

    let mut merged = ReferenceTerminology::default();
    let mut errors = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        let done = (i + 1) as u32;
        match result {
            Ok(resp) => match parse_response::extract_object(&resp.text) {
                Ok(v) => merged.merge(&ReferenceTerminology::from_value(&v)),
                Err(e) => {
                    let msg = format!(
                        "reference batch {done}/{total}: unparseable response ({e})"
                    );
                    let _ = tx
                        .send(GlossaryEvent::Log {
                            level: LogLevel::Warning,
                            message: msg.clone(),
                        })
                        .await;
                    errors.push(msg);
                }
            },
            Err(e) => errors.push(format!("reference batch {done}/{total} failed: {e}")),
        }
        let _ = tx.send(GlossaryEvent::Progress { done, total }).await;
    }
    merged.deduplicate();
    (merged, files_ok, errors)
}

/// O11 auto path (`glossary_phase.py:122-182`): cached file → use; else ref/
/// dir with `.ass` files → extract + cache; else None.
// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
pub async fn load_or_extract(
    folder: &Path,
    svc: &LlmService,
    batch_limit: Option<u32>,
    tx: &mpsc::Sender<GlossaryEvent>,
) -> Option<ReferenceTerminology> {
    let cache_path = folder.join(CACHE_FILENAME);
    if cache_path.exists() {
        match load_cache(folder) {
            Some(t) => {
                let _ = tx
                    .send(GlossaryEvent::Log {
                        level: LogLevel::Info,
                        message: format!(
                            "loaded {} reference terms from {CACHE_FILENAME}",
                            t.count()
                        ),
                    })
                    .await;
                return Some(t);
            }
            None => {
                // File exists but is unreadable/corrupt — warn and fall through.
                // We do NOT delete it (user may want to fix by hand); it will be
                // overwritten if extraction succeeds.
                let _ = tx
                    .send(GlossaryEvent::Log {
                        level: LogLevel::Warning,
                        message: format!(
                            "{CACHE_FILENAME} is unreadable — ignoring; it will be overwritten if extraction succeeds"
                        ),
                    })
                    .await;
            }
        }
    }
    let ref_dir = find_ref_dir(folder)?;
    let files = ref_ass_files(&ref_dir);
    if files.is_empty() {
        return None;
    }
    let _ = tx
        .send(GlossaryEvent::Log {
            level: LogLevel::Info,
            message: format!(
                "extracting reference terminology from {} files in ref/",
                files.len()
            ),
        })
        .await;
    let (t, _files_ok, errors) = extract_from_files(svc, &files, batch_limit, tx).await;
    for e in errors {
        let _ = tx
            .send(GlossaryEvent::Log { level: LogLevel::Warning, message: e })
            .await;
    }
    if t.count() > 0 {
        if let Err(e) = save_cache(folder, &t) {
            let _ = tx
                .send(GlossaryEvent::Log {
                    level: LogLevel::Warning,
                    message: format!("could not cache reference terms: {e}"),
                })
                .await;
        }
        return Some(t);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_is_case_insensitive_and_order_preserving() {
        let mut a = ReferenceTerminology {
            characters: vec!["Lin Dong".into()],
            ..Default::default()
        };
        let b = ReferenceTerminology {
            characters: vec!["lin dong".into(), "Ying Huanhuan".into()],
            items: vec!["Ancestral Symbol".into()],
            ..Default::default()
        };
        a.merge(&b);
        assert_eq!(a.characters, vec!["Lin Dong", "Ying Huanhuan"]);
        assert_eq!(a.count(), 3);
    }

    #[test]
    fn deduplicate_within_categories() {
        let mut t = ReferenceTerminology {
            skills: vec!["Devouring".into(), "devouring".into(), "Soul Symbol".into()],
            ..Default::default()
        };
        t.deduplicate();
        assert_eq!(t.skills, vec!["Devouring", "Soul Symbol"]);
    }

    #[test]
    fn prompt_string_lists_nonempty_categories() {
        let t = ReferenceTerminology {
            characters: vec!["Lin Dong".into(), "Ying Huanhuan".into()],
            locations: vec!["Qingyang Town".into()],
            ..Default::default()
        };
        let s = t.to_prompt_string();
        assert!(s.contains("CHARACTER NAMES: Lin Dong, Ying Huanhuan"));
        assert!(s.contains("LOCATIONS: Qingyang Town"));
        assert!(!s.contains("CULTIVATION"));
    }

    #[test]
    fn from_value_is_lenient() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"characters":["A", 1, "B"],"junk":true}"#).unwrap();
        let t = ReferenceTerminology::from_value(&v);
        assert_eq!(t.characters, vec!["A", "B"]);
        assert_eq!(t.count(), 2);
    }

    #[test]
    fn cache_roundtrip_and_corrupt_ignored() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_cache(dir.path()).is_none());
        let t = ReferenceTerminology {
            organizations: vec!["Dao Sect".into()],
            ..Default::default()
        };
        save_cache(dir.path(), &t).unwrap();
        assert_eq!(load_cache(dir.path()).unwrap().organizations, vec!["Dao Sect"]);
        // Corrupt cache: ignored (None) but NOT deleted (deviation from Python).
        std::fs::write(dir.path().join(CACHE_FILENAME), "not json").unwrap();
        assert!(load_cache(dir.path()).is_none());
        assert!(dir.path().join(CACHE_FILENAME).exists());
        clear_cache(dir.path()).unwrap();
        assert!(!dir.path().join(CACHE_FILENAME).exists());
        clear_cache(dir.path()).unwrap(); // idempotent
    }

    #[test]
    fn ref_dir_found_at_three_levels() {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("a/b");
        std::fs::create_dir_all(&work).unwrap();

        // No ref/ anywhere yet.
        assert!(find_ref_dir(&work).is_none());

        // Grandparent level: root/ref — visible from root/a/b.
        std::fs::create_dir(root.path().join("ref")).unwrap();
        assert_eq!(find_ref_dir(&work).unwrap(), root.path().join("ref"));

        // Parent level wins over grandparent: root/a/ref closer.
        std::fs::create_dir(root.path().join("a/ref")).unwrap();
        assert_eq!(find_ref_dir(&work).unwrap(), root.path().join("a/ref"));

        // Own level wins over parent: root/a/b/ref is closest.
        std::fs::create_dir(work.join("ref")).unwrap();
        assert_eq!(find_ref_dir(&work).unwrap(), work.join("ref"));
    }

    #[test]
    fn category_keys_match_glossary_categories() {
        let label_keys: Vec<&str> = CATEGORY_LABELS.iter().map(|(c, _)| *c).collect();
        assert_eq!(label_keys.as_slice(), crate::glossary::model::CATEGORIES.as_slice());
    }

    #[test]
    fn status_prefers_cache_then_ref_dir() {
        let dir = tempfile::tempdir().unwrap();
        let s = reference_status(dir.path());
        assert_eq!(s.source, ReferenceSource::None);

        std::fs::create_dir(dir.path().join("ref")).unwrap();
        std::fs::write(dir.path().join("ref/e1.ass"), "x").unwrap();
        std::fs::write(dir.path().join("ref/notes.txt"), "x").unwrap();
        let s = reference_status(dir.path());
        assert_eq!(s.source, ReferenceSource::RefDir);
        assert_eq!(s.count, 1); // .ass files only

        let t = ReferenceTerminology {
            characters: vec!["A".into(), "B".into()],
            ..Default::default()
        };
        save_cache(dir.path(), &t).unwrap();
        let s = reference_status(dir.path());
        assert_eq!(s.source, ReferenceSource::Cached);
        assert_eq!(s.count, 2); // term count
    }

    // ── Async extractor tests ────────────────────────────────────────────────

    use crate::events::GlossaryEvent;
    use crate::llm::service::LlmService;
    use crate::llm::test_support::ScriptedDriver;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    fn make_svc(driver: Arc<ScriptedDriver>, cap: u32) -> LlmService {
        let (tx, _rx) = tokio::sync::mpsc::channel(64);
        LlmService::new(driver, cap, CancellationToken::new(), tx)
    }

    fn write_ass(dir: &Path, name: &str, lines: &[&str]) -> PathBuf {
        let mut content = String::from(
            "[Script Info]\nTitle: t\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );
        for (i, l) in lines.iter().enumerate() {
            content.push_str(&format!(
                "Dialogue: 0,0:00:0{i}.00,0:00:0{}.00,Default,,0,0,0,,{l}\n",
                i + 1
            ));
        }
        let p = dir.join(name);
        std::fs::write(&p, content).unwrap();
        p
    }

    #[tokio::test(start_paused = true)]
    async fn extractor_merges_batches_and_dedupes() {
        let dir = tempfile::tempdir().unwrap();
        let f = write_ass(dir.path(), "e1.ass", &["Lin Dong attacks", "The sect gathers"]);
        let d = ScriptedDriver::new(vec![Ok(
            r#"{"characters":["Lin Dong","lin dong"],"organizations":["Dao Sect"]}"#.into(),
        )]);
        let (gtx, _grx) = tokio::sync::mpsc::channel::<GlossaryEvent>(64);
        let svc = make_svc(d, 2);
        let (t, files_ok, errors) = extract_from_files(&svc, &[f], Some(300), &gtx).await;
        assert!(errors.is_empty());
        assert_eq!(files_ok, 1);
        assert_eq!(t.characters, vec!["Lin Dong"]); // deduped case-insensitively
        assert_eq!(t.organizations, vec!["Dao Sect"]);
    }

    #[tokio::test(start_paused = true)]
    async fn extractor_records_batch_failure_and_continues() {
        let dir = tempfile::tempdir().unwrap();
        // batch limit 2 → ×0.7 → 1 line per batch → 2 batches for 2 lines.
        // HTTP 400 is non-retryable and non-auth: exactly one driver call per
        // batch. Script order = join_all completion order (deterministic: no
        // spawned tasks, no interleaving).
        let f = write_ass(dir.path(), "e1.ass", &["line one", "line two"]);
        let d = ScriptedDriver::new(vec![
            Err(crate::llm::error::LlmError::Http { status: 400, body: "bad request".into() }),
            Ok(r#"{"locations":["Qingyang Town"]}"#.into()),
        ]);
        let (gtx, _grx) = tokio::sync::mpsc::channel::<GlossaryEvent>(64);
        let svc = make_svc(d, 1);
        let (t, _files_ok, errors) = extract_from_files(&svc, &[f], Some(2), &gtx).await;
        assert_eq!(errors.len(), 1, "expected 1 error, got: {:?}", errors);
        assert!(
            errors[0].contains("failed"),
            "error must say 'failed': got {:?}",
            errors[0]
        );
        assert_eq!(t.locations, vec!["Qingyang Town"]);
    }

    #[tokio::test(start_paused = true)]
    async fn load_or_extract_prefers_cache() {
        let dir = tempfile::tempdir().unwrap();
        // Struct literal, not field-reassign — clippy::field_reassign_with_default.
        let cached = ReferenceTerminology {
            items: vec!["Stone Talisman".into()],
            ..Default::default()
        };
        save_cache(dir.path(), &cached).unwrap();
        // Driver would panic if called (empty script) — cache short-circuits.
        let d = ScriptedDriver::new(vec![]);
        let (gtx, _grx) = tokio::sync::mpsc::channel::<GlossaryEvent>(64);
        let svc = make_svc(d, 2);
        let t = load_or_extract(dir.path(), &svc, Some(300), &gtx).await;
        assert_eq!(t.unwrap().items, vec!["Stone Talisman"]);
    }

    #[tokio::test(start_paused = true)]
    async fn load_or_extract_extracts_from_ref_dir_and_caches() {
        let dir = tempfile::tempdir().unwrap();
        let ref_dir = dir.path().join("ref");
        std::fs::create_dir(&ref_dir).unwrap();
        write_ass(&ref_dir, "e1.ass", &["Lin Dong strikes"]);
        let d = ScriptedDriver::new(vec![Ok(r#"{"characters":["Lin Dong"]}"#.into())]);
        let (gtx, _grx) = tokio::sync::mpsc::channel::<GlossaryEvent>(64);
        let svc = make_svc(d, 2);
        let t = load_or_extract(dir.path(), &svc, Some(300), &gtx).await.unwrap();
        assert_eq!(t.characters, vec!["Lin Dong"]);
        // Cached for next time.
        assert_eq!(load_cache(dir.path()).unwrap().characters, vec!["Lin Dong"]);
    }

    #[tokio::test(start_paused = true)]
    async fn load_or_extract_none_when_nothing_available() {
        let dir = tempfile::tempdir().unwrap();
        let d = ScriptedDriver::new(vec![]);
        let (gtx, _grx) = tokio::sync::mpsc::channel::<GlossaryEvent>(64);
        let svc = make_svc(d, 2);
        assert!(load_or_extract(dir.path(), &svc, Some(300), &gtx).await.is_none());
    }
}
