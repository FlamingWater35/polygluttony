//! Six-category translation glossary. Port of `glossary/glossary.py` (model
//! only; building arrives with the Glossary step).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const CATEGORIES: [&str; 6] =
    ["characters", "cultivation", "skills", "locations", "items", "organizations"];

fn header(category: &str) -> &'static str {
    match category {
        "characters" => "CHARACTER NAMES (use exactly as shown)",
        "cultivation" => "CULTIVATION LEVELS",
        "skills" => "SKILLS & ABILITIES",
        "locations" => "LOCATIONS",
        "items" => "ITEMS & ARTIFACTS",
        "organizations" => "ORGANIZATIONS",
        _ => "TERMS",
    }
}

/// Webview-facing glossary document (O9/O14). `terms` is category → map.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct GlossaryDoc {
    pub world_type: String,
    pub terms: BTreeMap<String, BTreeMap<String, String>>,
    pub count: u32,
}

impl From<&Glossary> for GlossaryDoc {
    fn from(g: &Glossary) -> Self {
        let mut terms = BTreeMap::new();
        for c in CATEGORIES {
            terms.insert(c.to_string(), g.category(c).clone());
        }
        GlossaryDoc { world_type: g.world_type.clone(), terms, count: g.count() as u32 }
    }
}

impl GlossaryDoc {
    /// Unknown categories in `terms` are dropped; `count` is ignored (derived).
    pub fn into_glossary(self) -> Glossary {
        let mut g = Glossary::new(&self.world_type);
        for c in CATEGORIES {
            if let Some(m) = self.terms.get(c) {
                *g.category_mut(c) = m.clone();
            }
        }
        g
    }
}

/// BTreeMap keeps category dumps deterministic (Python dicts preserved insert
/// order; determinism is what the prompts actually need).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Glossary {
    pub world_type: String,
    pub characters: BTreeMap<String, String>,
    pub cultivation: BTreeMap<String, String>,
    pub skills: BTreeMap<String, String>,
    pub locations: BTreeMap<String, String>,
    pub items: BTreeMap<String, String>,
    pub organizations: BTreeMap<String, String>,
}

impl Glossary {
    pub fn new(world_type: &str) -> Self {
        Glossary { world_type: world_type.into(), ..Default::default() }
    }

    pub fn category(&self, name: &str) -> &BTreeMap<String, String> {
        match name {
            "characters" => &self.characters,
            "cultivation" => &self.cultivation,
            "skills" => &self.skills,
            "locations" => &self.locations,
            "items" => &self.items,
            "organizations" => &self.organizations,
            _ => panic!("unknown glossary category: {name}"),
        }
    }

    pub(crate) fn category_mut(&mut self, name: &str) -> &mut BTreeMap<String, String> {
        match name {
            "characters" => &mut self.characters,
            "cultivation" => &mut self.cultivation,
            "skills" => &mut self.skills,
            "locations" => &mut self.locations,
            "items" => &mut self.items,
            "organizations" => &mut self.organizations,
            _ => panic!("unknown glossary category: {name}"),
        }
    }

    pub fn count(&self) -> usize {
        CATEGORIES.iter().map(|c| self.category(c).len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// term → translation across all categories.
    pub fn all_terms(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for c in CATEGORIES {
            for (k, v) in self.category(c) {
                out.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        out
    }

    /// Keep only terms that literally appear in `batch_content`
    /// (`glossary.py:171-189`).
    pub fn filter_for_batch(&self, batch_content: &str) -> Glossary {
        let mut g = Glossary::new(&self.world_type);
        for c in CATEGORIES {
            for (term, tr) in self.category(c) {
                if batch_content.contains(term.as_str()) {
                    g.category_mut(c).insert(term.clone(), tr.clone());
                }
            }
        }
        g
    }

    /// `星汉那边` → `星汉[→Xinghan]那边`, longest term first so compounds beat
    /// their prefixes (`batch_translator.py:486-512`).
    ///
    /// Deliberate improvement over Python: single left-to-right scan instead of
    /// sequential `str.replace`, so a short term can never re-match inside an
    /// already-injected longer hint (Python corrupts `星汉[→Xinghan]` to
    /// `星[→Star]汉[→Xinghan]` when both `星汉` and `星` are in the glossary).
    pub fn inject_hints(&self, src: &str) -> String {
        if self.is_empty() {
            return src.to_string();
        }
        let mut terms: Vec<(&String, &String)> = self.all_terms_ref();
        terms.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()).then(a.0.cmp(b.0)));

        let chars: Vec<char> = src.chars().collect();
        let mut out = String::with_capacity(src.len());
        let mut i = 0usize;
        'outer: while i < chars.len() {
            let rest: String = chars[i..].iter().collect();
            for (term, tr) in &terms {
                if rest.starts_with(term.as_str()) {
                    out.push_str(term);
                    out.push_str(&format!("[→{tr}]"));
                    i += term.chars().count();
                    continue 'outer;
                }
            }
            out.push(chars[i]);
            i += 1;
        }
        out
    }

    /// Borrowed view of all terms (helper for `inject_hints`; first category
    /// wins on duplicates, matching `all_terms`).
    fn all_terms_ref(&self) -> Vec<(&String, &String)> {
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for c in CATEGORIES {
            for (k, v) in self.category(c) {
                if seen.insert(k) {
                    out.push((k, v));
                }
            }
        }
        out
    }

    /// Human-readable block for the `{GLOSSARY}` placeholder
    /// (`glossary.py:242-260`).
    pub fn to_formatted_string(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        for c in CATEGORIES {
            let terms = self.category(c);
            if terms.is_empty() {
                continue;
            }
            out.push(format!("{}:", header(c)));
            for (term, tr) in terms {
                out.push(format!("  {term} → {tr}"));
            }
            out.push(String::new());
        }
        out.join("\n")
    }

    /// True if `key` exists in ANY category (`glossary.py:149-158`).
    // consumed by the build pipeline (later step-4 task)
    #[allow(dead_code)]
    pub fn has_key(&self, key: &str) -> bool {
        CATEGORIES.iter().any(|c| self.category(c).contains_key(key))
    }

    /// Reject empty/whitespace-only or absurdly long values (likely
    /// hallucinations) — `glossary.py:327-339`.
    pub(crate) fn is_valid_translation(value: &str) -> bool {
        let t = value.trim();
        !t.is_empty() && t.chars().count() <= 200
    }

    /// First-wins merge: only keys absent from ALL categories are added, values
    /// are validated and trimmed (`glossary.py:109-138`). Keys added earlier in
    /// this merge also block later categories (cross-category dedupe).
    // consumed by the build pipeline (later step-4 task)
    #[allow(dead_code)]
    pub fn merge_first_wins(&mut self, other: &Glossary) {
        for c in CATEGORIES {
            for (source, translation) in other.category(c) {
                if !self.has_key(source) && Self::is_valid_translation(translation) {
                    self.category_mut(c).insert(source.clone(), translation.trim().to_string());
                }
            }
        }
    }

    /// Drop empty values from all categories (`glossary.py:140-147`).
    // consumed by the build pipeline (later step-4 task)
    #[allow(dead_code)]
    pub fn deduplicate(&mut self) {
        for c in CATEGORIES {
            self.category_mut(c).retain(|_, v| !v.is_empty());
        }
    }

    /// Parse an LLM extraction response: accepts `{"terms": {...}}` or a bare
    /// category object; non-string values dropped
    /// (`glossary_builder.py:375` + `glossary.py:202-220`).
    // consumed by the build pipeline (later step-4 task)
    #[allow(dead_code)]
    pub fn from_terms_value(v: &serde_json::Value, world_type: &str) -> Glossary {
        let terms = match v.get("terms") {
            Some(t) if t.is_object() => t,
            _ => v,
        };
        let mut g = Glossary::new(world_type);
        if let Some(obj) = terms.as_object() {
            for c in CATEGORIES {
                if let Some(cat) = obj.get(c).and_then(|x| x.as_object()) {
                    for (k, val) in cat {
                        if let Some(s) = val.as_str() {
                            g.category_mut(c).insert(k.clone(), s.to_string());
                        }
                    }
                }
            }
        }
        g
    }

    /// Build the `{"world_type", "terms"}` JSON document value shared by both
    /// serialization methods.
    fn to_value(&self) -> serde_json::Value {
        let mut terms = serde_json::Map::new();
        for c in CATEGORIES {
            let m: serde_json::Map<String, serde_json::Value> = self
                .category(c)
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            terms.insert(c.to_string(), serde_json::Value::Object(m));
        }
        serde_json::json!({ "world_type": self.world_type, "terms": terms })
    }

    /// Pretty-printed glossary.json — the file is meant to be hand-editable
    /// ("Open in editor", O15), so we write indent-2 like the Python tool.
    // consumed by save_folder_glossary (io.rs)
    #[allow(dead_code)]
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.to_value()).expect("serializable")
    }

    /// Lenient parse (`glossary.py:210-241`): unknown/garbage values dropped,
    /// `None` only if the document isn't JSON at all.
    pub fn from_json(s: &str) -> Option<Glossary> {
        let v: serde_json::Value = serde_json::from_str(s).ok()?;
        let world = v.get("world_type").and_then(|w| w.as_str()).unwrap_or("xianxia");
        // Re-use from_terms_value: it accepts the whole doc and finds "terms"
        // itself (or treats it as bare categories), and already sets world_type.
        Some(Glossary::from_terms_value(&v, world))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Glossary {
        let mut g = Glossary::new("xianxia");
        g.characters.insert("星汉".into(), "Xinghan".into());
        g.characters.insert("星".into(), "Star".into()); // prefix of 星汉 — tests longest-first
        g.locations.insert("凌天门".into(), "Lingtian Sect".into());
        g
    }

    #[test]
    fn json_roundtrip_matches_python_shape() {
        let g = sample();
        let json = g.to_json_pretty();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["world_type"], "xianxia");
        assert_eq!(v["terms"]["characters"]["星汉"], "Xinghan");
        let back = Glossary::from_json(&json).unwrap();
        assert_eq!(back.characters.get("星汉").unwrap(), "Xinghan");
        assert_eq!(back.world_type, "xianxia");
    }

    #[test]
    fn from_json_tolerates_garbage() {
        assert!(Glossary::from_json("not json").is_none());
        let g = Glossary::from_json(r#"{"world_type":"modern","terms":{"characters":{"a":1}}}"#).unwrap();
        assert!(g.characters.is_empty()); // non-string values dropped
    }

    #[test]
    fn filter_for_batch_keeps_only_present_terms() {
        let g = sample();
        let f = g.filter_for_batch("星汉那边如何");
        assert!(f.characters.contains_key("星汉"));
        assert!(f.characters.contains_key("星")); // substring of the text too
        assert!(!f.locations.contains_key("凌天门"));
    }

    #[test]
    fn inject_hints_longest_match_first() {
        let g = sample();
        // 星汉 must win over its prefix 星.
        assert_eq!(g.inject_hints("星汉那边"), "星汉[→Xinghan]那边");
        assert_eq!(g.inject_hints("星光"), "星[→Star]光");
    }

    #[test]
    fn formatted_string_has_category_headers() {
        let s = sample().to_formatted_string();
        assert!(s.contains("CHARACTER NAMES (use exactly as shown):"));
        assert!(s.contains("  星汉 → Xinghan"));
        assert!(s.contains("LOCATIONS:"));
    }

    #[test]
    fn all_terms_and_counts() {
        let g = sample();
        assert_eq!(g.count(), 3);
        assert!(!g.is_empty());
        assert!(Glossary::new("modern").is_empty());
    }

    #[test]
    fn merge_first_wins_keeps_existing_and_validates() {
        let mut a = Glossary::new("xianxia");
        a.characters.insert("林动".into(), "Lin Dong".into());
        let mut b = Glossary::new("xianxia");
        b.characters.insert("林动".into(), "WRONG".into()); // existing key — ignored
        b.characters.insert("应欢欢".into(), "  Ying Huanhuan  ".into()); // trimmed on insert
        b.locations.insert("林动".into(), "Cross-cat dup".into()); // dup across categories — ignored
        b.items.insert("空".into(), "   ".into()); // whitespace-only — rejected
        b.skills.insert("长".into(), "x".repeat(201)); // >200 chars — rejected
        a.merge_first_wins(&b);
        assert_eq!(a.characters.get("林动").unwrap(), "Lin Dong");
        assert_eq!(a.characters.get("应欢欢").unwrap(), "Ying Huanhuan");
        assert!(!a.locations.contains_key("林动"));
        assert!(!a.items.contains_key("空"));
        assert!(!a.skills.contains_key("长"));
    }

    #[test]
    fn merge_first_wins_dedupes_within_other_across_categories() {
        // A key occurring twice inside `other` (different categories): first
        // category in CATEGORIES order wins, the later one is blocked by has_key.
        let mut a = Glossary::new("xianxia");
        let mut b = Glossary::new("xianxia");
        b.characters.insert("星".into(), "Star (char)".into());
        b.locations.insert("星".into(), "Star (loc)".into());
        a.merge_first_wins(&b);
        assert_eq!(a.characters.get("星").unwrap(), "Star (char)");
        assert!(!a.locations.contains_key("星"));
    }

    #[test]
    fn deduplicate_drops_empty_values() {
        let mut g = Glossary::new("modern");
        g.characters.insert("a".into(), "".into());
        g.characters.insert("b".into(), "B".into());
        g.deduplicate();
        assert!(!g.characters.contains_key("a"));
        assert_eq!(g.count(), 1);
    }

    #[test]
    fn has_key_checks_all_categories() {
        let g = sample();
        assert!(g.has_key("凌天门")); // in locations
        assert!(!g.has_key("不存在"));
    }

    #[test]
    fn from_terms_value_accepts_wrapped_and_bare() {
        let wrapped: serde_json::Value =
            serde_json::from_str(r#"{"terms":{"characters":{"林动":"Lin Dong"},"items":{"bad":1}}}"#)
                .unwrap();
        let g = Glossary::from_terms_value(&wrapped, "wuxia");
        assert_eq!(g.world_type, "wuxia");
        assert_eq!(g.characters.get("林动").unwrap(), "Lin Dong");
        assert!(g.items.is_empty()); // non-string value dropped

        let bare: serde_json::Value =
            serde_json::from_str(r#"{"locations":{"青阳镇":"Qingyang Town"}}"#).unwrap();
        let g2 = Glossary::from_terms_value(&bare, "xianxia");
        assert_eq!(g2.locations.get("青阳镇").unwrap(), "Qingyang Town");
    }

    #[test]
    fn to_json_pretty_is_indented_and_roundtrips() {
        let g = sample();
        let s = g.to_json_pretty();
        assert!(s.contains("\n  ")); // indented
        let back = Glossary::from_json(&s).unwrap();
        assert_eq!(back, g);
    }

    #[test]
    fn glossary_doc_roundtrip() {
        let g = sample();
        let doc = GlossaryDoc::from(&g);
        assert_eq!(doc.world_type, "xianxia");
        assert_eq!(doc.count, 3);
        assert_eq!(doc.terms["characters"]["星汉"], "Xinghan");
        let back = doc.into_glossary();
        assert_eq!(back, g);
    }
}
