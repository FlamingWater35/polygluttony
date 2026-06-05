//! Source/target language pair with validation + output-filename helpers.
//! Ported from `models/language_pair.py`.

use std::path::Path;

use crate::config::languages::{get_language, languages, resolve_language_code};
use crate::error::{AppError, AppResult};

/// A validated source→target language pair.
#[derive(Debug, Clone)]
pub struct LanguagePair {
    pub source: String,
    pub target: String,
    pub source_name: String,
    pub target_name: String,
    pub output_suffix: String,
    // Step 4 (Glossary): pipeline reads these to gate glossary injection.
    #[allow(dead_code)]
    pub supports_glossary: bool,
    #[allow(dead_code)]
    pub supports_world_detection: bool,
}

impl LanguagePair {
    /// Resolve + validate a pair. Errors if either code is unknown or if they are equal.
    pub fn from_codes(source: &str, target: &str) -> AppResult<LanguagePair> {
        let src = resolve_language_code(source)
            .ok_or_else(|| AppError::Other(format!("unsupported source language: {source}")))?;
        let tgt = resolve_language_code(target)
            .ok_or_else(|| AppError::Other(format!("unsupported target language: {target}")))?;
        if src == tgt {
            return Err(AppError::Other(
                "source and target languages must differ".into(),
            ));
        }
        let s = get_language(&src).expect("resolved code exists");
        let t = get_language(&tgt).expect("resolved code exists");
        Ok(LanguagePair {
            source: s.code,
            target: t.code,
            source_name: s.name,
            target_name: t.name,
            output_suffix: t.output_suffix,
            supports_glossary: s.supports_glossary,
            supports_world_detection: s.supports_world_detection,
        })
    }

    /// Suffixes that mark an already-translated file for this target.
    pub fn target_suffixes(&self) -> Vec<String> {
        let mut v = vec![self.output_suffix.clone()];
        if self.target != self.output_suffix {
            v.push(self.target.clone());
        }
        v
    }
}

/// All language code/alias/suffix tokens, longest first (for stripping).
fn all_language_suffixes() -> Vec<String> {
    let mut set: Vec<String> = Vec::new();
    for l in languages() {
        set.push(l.code.clone());
        set.push(l.output_suffix.clone());
        for a in &l.aliases {
            set.push(a.clone());
        }
    }
    set.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    set.dedup();
    set
}

/// Strip a trailing `.{lang-suffix}` from a filename stem (case-insensitive).
pub fn strip_language_suffix(stem: &str) -> String {
    let lower = stem.to_lowercase();
    for suffix in all_language_suffixes() {
        let dotted = format!(".{}", suffix.to_lowercase());
        if lower.ends_with(&dotted) && stem.len() > dotted.len() {
            return stem[..stem.len() - dotted.len()].to_string();
        }
    }
    stem.to_string()
}

fn base_stem(source: &Path) -> String {
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    strip_language_suffix(stem)
}

/// Output filename for a translated file: `{stem}.{suffix}.ass` (suffix stripped first).
pub fn output_filename(source: &Path, pair: &LanguagePair) -> String {
    format!("{}.{}.ass", base_stem(source), pair.output_suffix)
}

/// Warning-variant filename: `{stem}.warning.{suffix}.ass`.
pub fn warning_filename(source: &Path, pair: &LanguagePair) -> String {
    format!("{}.warning.{}.ass", base_stem(source), pair.output_suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn from_codes_validates() {
        let p = LanguagePair::from_codes("zh", "en").unwrap();
        assert_eq!(p.output_suffix, "eng");
        assert!(p.supports_glossary);
        assert!(LanguagePair::from_codes("zh", "zh").is_err());
        assert!(LanguagePair::from_codes("zz", "en").is_err());
    }

    #[test]
    fn output_and_warning_filenames() {
        let p = LanguagePair::from_codes("zh", "en").unwrap();
        assert_eq!(output_filename(Path::new("/x/ep01.ass"), &p), "ep01.eng.ass");
        assert_eq!(output_filename(Path::new("/x/ep01.chi.ass"), &p), "ep01.eng.ass");
        assert_eq!(warning_filename(Path::new("/x/ep01.ass"), &p), "ep01.warning.eng.ass");
    }

    #[test]
    fn target_suffixes_include_code_when_different() {
        let p = LanguagePair::from_codes("zh", "en").unwrap();
        assert_eq!(p.target_suffixes(), vec!["eng".to_string(), "en".to_string()]);
    }
}
