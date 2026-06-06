//! Reference terminology (O11): English terms lifted from already-translated
//! `.ass` files, injected into the extraction prompt for consistency. Ports
//! `glossary/reference_terminology.py` + `reference_loader.py`. The async
//! LLM extractor lives in this module too (added in a later task).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppResult;

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

// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
pub fn save_cache(folder: &Path, t: &ReferenceTerminology) -> AppResult<()> {
    let json = serde_json::to_string_pretty(t)?;
    std::fs::write(folder.join(CACHE_FILENAME), json)?;
    Ok(())
}

/// Idempotent delete (missing file is fine).
// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
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
// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ReferenceStatus {
    pub source: ReferenceSource,
    pub count: u32,
}

// consumed by the O11 command (later step-4 task)
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_is_case_insensitive_and_order_preserving() {
        let mut a = ReferenceTerminology::default();
        a.characters = vec!["Lin Dong".into()];
        let mut b = ReferenceTerminology::default();
        b.characters = vec!["lin dong".into(), "Ying Huanhuan".into()];
        b.items = vec!["Ancestral Symbol".into()];
        a.merge(&b);
        assert_eq!(a.characters, vec!["Lin Dong", "Ying Huanhuan"]);
        assert_eq!(a.count(), 3);
    }

    #[test]
    fn deduplicate_within_categories() {
        let mut t = ReferenceTerminology::default();
        t.skills = vec!["Devouring".into(), "devouring".into(), "Soul Symbol".into()];
        t.deduplicate();
        assert_eq!(t.skills, vec!["Devouring", "Soul Symbol"]);
    }

    #[test]
    fn prompt_string_lists_nonempty_categories() {
        let mut t = ReferenceTerminology::default();
        t.characters = vec!["Lin Dong".into(), "Ying Huanhuan".into()];
        t.locations = vec!["Qingyang Town".into()];
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
        let mut t = ReferenceTerminology::default();
        t.organizations = vec!["Dao Sect".into()];
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
        assert!(find_ref_dir(&work).is_none());
        std::fs::create_dir(root.path().join("a/ref")).unwrap(); // parent level
        assert_eq!(find_ref_dir(&work).unwrap(), root.path().join("a/ref"));
        std::fs::create_dir(work.join("ref")).unwrap(); // own level wins
        assert_eq!(find_ref_dir(&work).unwrap(), work.join("ref"));
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

        let mut t = ReferenceTerminology::default();
        t.characters = vec!["A".into(), "B".into()];
        save_cache(dir.path(), &t).unwrap();
        let s = reference_status(dir.path());
        assert_eq!(s.source, ReferenceSource::Cached);
        assert_eq!(s.count, 2); // term count
    }
}
