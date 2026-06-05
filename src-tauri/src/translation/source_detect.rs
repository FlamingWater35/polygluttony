//! Detect residual source-language characters in translations (cleanup
//! trigger). Port of `utils/source_detector.py`; threshold strictly > 0.1.

use regex::Regex;

use crate::config::languages::get_language;

pub const CLEANUP_THRESHOLD: f64 = 0.1;

pub struct SourceDetector {
    pattern: Regex,
}

impl SourceDetector {
    /// `None` when the source language has no character pattern (e.g. en):
    /// cleanup is then structurally impossible, matching Python.
    pub fn for_language(code: &str) -> Option<SourceDetector> {
        let lang = get_language(code)?;
        let pat = lang.character_pattern?;
        Some(SourceDetector { pattern: Regex::new(&pat).expect("valid language pattern") })
    }

    pub fn source_ratio(&self, text: &str) -> f64 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        self.pattern.find_iter(text).count() as f64 / total as f64
    }

    pub fn needs_cleanup(&self, text: &str) -> bool {
        self.source_ratio(text) > CLEANUP_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zh() -> SourceDetector {
        SourceDetector::for_language("zh").unwrap()
    }

    #[test]
    fn ratio_counts_source_chars() {
        let d = zh();
        assert_eq!(d.source_ratio("hello"), 0.0);
        assert!(d.source_ratio("你好") > 0.9);
        // "你好 ab" → 2 CJK of 5 chars = 0.4
        assert!((d.source_ratio("你好 ab") - 0.4).abs() < 1e-9);
    }

    #[test]
    fn needs_cleanup_above_10_percent() {
        let d = zh();
        assert!(d.needs_cleanup("还有 some leftover 中文 here"));
        assert!(!d.needs_cleanup("Perfectly clean English sentence."));
        // Exactly at threshold is NOT cleanup (strictly greater).
        assert!(!d.needs_cleanup("中aaaaaaaaa")); // 1/10 == 0.1
    }

    #[test]
    fn language_without_pattern_never_needs_cleanup() {
        assert!(SourceDetector::for_language("en").is_none());
    }
}
