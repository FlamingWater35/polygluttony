//! Supported languages + alias resolution + source-language detection.
//! Ported from `config/languages.py`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One supported language.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Language {
    pub code: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub output_suffix: String,
    pub character_pattern: Option<String>,
    pub supports_glossary: bool,
    pub supports_world_detection: bool,
}

fn lang(
    code: &str,
    name: &str,
    aliases: &[&str],
    suffix: &str,
    pattern: Option<&str>,
    gloss: bool,
    world: bool,
) -> Language {
    Language {
        code: code.into(),
        name: name.into(),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        output_suffix: suffix.into(),
        character_pattern: pattern.map(|s| s.to_string()),
        supports_glossary: gloss,
        supports_world_detection: world,
    }
}

/// The supported-language table (ported from `config/languages.py:LANGUAGES`).
pub fn languages() -> Vec<Language> {
    vec![
        lang("zh", "Chinese", &["chi", "chs", "cht", "cn", "chinese"], "chi", Some(r"[\u{4E00}-\u{9FFF}]"), true, true),
        lang("ko", "Korean", &["kor", "korean"], "kor", Some(r"[\u{AC00}-\u{D7AF}\u{1100}-\u{11FF}]"), false, false),
        lang("ja", "Japanese", &["jpn", "japanese"], "jpn", Some(r"[\u{3040}-\u{309F}\u{30A0}-\u{30FF}\u{4E00}-\u{9FFF}]"), false, false),
        lang("en", "English", &["eng", "english"], "eng", None, false, false),
        lang("es", "Spanish", &["spa", "spanish"], "spa", None, false, false),
        lang("fr", "French", &["fra", "french"], "fra", None, false, false),
        lang("de", "German", &["ger", "deu", "german"], "ger", None, false, false),
        lang("pt", "Portuguese", &["por", "portuguese"], "por", None, false, false),
        lang("ru", "Russian", &["rus", "russian"], "rus", Some(r"[\u{0400}-\u{04FF}]"), false, false),
        lang("ar", "Arabic", &["ara", "arabic"], "ara", Some(r"[\u{0600}-\u{06FF}]"), false, false),
        lang("th", "Thai", &["tha", "thai"], "tha", Some(r"[\u{0E00}-\u{0E7F}]"), false, false),
        lang("vi", "Vietnamese", &["vie", "vietnamese"], "vie", None, false, false),
        lang("id", "Indonesian", &["ind", "indonesian"], "ind", None, false, false),
        lang("ms", "Malay", &["msa", "malay"], "msa", None, false, false),
        lang("bg", "Bulgarian", &["bul", "bulgarian"], "bul", Some(r"[\u{0400}-\u{04FF}]"), false, false),
    ]
}

/// Resolve a code or alias (case-insensitive) to its canonical code.
pub fn resolve_language_code(code: &str) -> Option<String> {
    let norm = code.trim().to_lowercase();
    let langs = languages();
    if langs.iter().any(|l| l.code == norm) {
        return Some(norm);
    }
    langs
        .into_iter()
        .find(|l| l.aliases.iter().any(|a| a == &norm))
        .map(|l| l.code)
}

/// Get a language by code or alias.
pub fn get_language(code: &str) -> Option<Language> {
    let resolved = resolve_language_code(code)?;
    languages().into_iter().find(|l| l.code == resolved)
}

/// Guess the source language from dialogue text by counting characters matching
/// each language's Unicode range; the highest count wins. None if no script chars.
pub fn detect_source_language(text: &str) -> Option<String> {
    let mut best: Option<(String, usize)> = None;
    for l in languages() {
        let Some(pat) = &l.character_pattern else {
            continue;
        };
        let re = regex::Regex::new(pat).expect("valid language pattern");
        let n = re.find_iter(text).count();
        if n > 0 && best.as_ref().map_or(true, |(_, b)| n > *b) {
            best = Some((l.code.clone(), n));
        }
    }
    best.map(|(c, _)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_codes_and_aliases() {
        assert_eq!(resolve_language_code("ZH").as_deref(), Some("zh"));
        assert_eq!(resolve_language_code("chinese").as_deref(), Some("zh"));
        assert_eq!(resolve_language_code("eng").as_deref(), Some("en"));
        assert_eq!(resolve_language_code("xx"), None);
    }

    #[test]
    fn detects_source_language_from_text() {
        assert_eq!(detect_source_language("修仙者突破金丹").as_deref(), Some("zh"));
        assert_eq!(detect_source_language("hello world"), None);
        assert_eq!(detect_source_language("Привет мир").as_deref(), Some("ru"));
    }

    #[test]
    fn table_has_expected_size_and_zh_flags() {
        let langs = languages();
        assert_eq!(langs.len(), 15);
        let zh = langs.iter().find(|l| l.code == "zh").unwrap();
        assert!(zh.supports_glossary && zh.supports_world_detection);
        let en = langs.iter().find(|l| l.code == "en").unwrap();
        assert!(!en.supports_glossary && !en.supports_world_detection);
    }
}
