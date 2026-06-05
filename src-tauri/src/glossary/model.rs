//! Six-category translation glossary. Port of `glossary/glossary.py` (model
//! only; building arrives with the Glossary step).

use std::collections::BTreeMap;

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

    fn category_mut(&mut self, name: &str) -> &mut BTreeMap<String, String> {
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

    /// `{"world_type": ..., "terms": {category: {term: translation}}}` —
    /// byte-compatible with the Python tool's `glossary.json`.
    // Step 4 (Glossary view): commands/glossary will call this to persist the file.
    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        let mut terms = serde_json::Map::new();
        for c in CATEGORIES {
            let m: serde_json::Map<String, serde_json::Value> = self
                .category(c)
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            terms.insert(c.to_string(), serde_json::Value::Object(m));
        }
        serde_json::json!({ "world_type": self.world_type, "terms": terms }).to_string()
    }

    /// Lenient parse (`glossary.py:210-241`): unknown/garbage values dropped,
    /// `None` only if the document isn't JSON at all.
    pub fn from_json(s: &str) -> Option<Glossary> {
        let v: serde_json::Value = serde_json::from_str(s).ok()?;
        let world = v.get("world_type").and_then(|w| w.as_str()).unwrap_or("xianxia");
        let mut g = Glossary::new(world);
        if let Some(terms) = v.get("terms").and_then(|t| t.as_object()) {
            for c in CATEGORIES {
                if let Some(cat) = terms.get(c).and_then(|x| x.as_object()) {
                    for (k, val) in cat {
                        if let Some(s) = val.as_str() {
                            g.category_mut(c).insert(k.clone(), s.to_string());
                        }
                    }
                }
            }
        }
        Some(g)
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
        let json = g.to_json();
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
}
