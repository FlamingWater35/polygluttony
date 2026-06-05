//! Prompt assembly from embedded templates. Port of
//! `core/batch_translator.py:415-484`. Templates live in `src-tauri/prompts/`.
//!
//! # Placeholder mapping per template
//!
//! - `translate.zh-en.txt`:        `{GLOSSARY}`, `{TONE}`
//! - `translate.zh-en.qwen.txt`:   `{localization_style}`, `{glossary_formatted}`
//! - `translate.generic.txt`:      `{source_language}`, `{target_language}`, `{localization_style}`

use crate::config::projects::Tone;
use crate::glossary::model::Glossary;
use crate::models::language_pair::LanguagePair;

const TRANSLATE_GENERIC: &str = include_str!("../../prompts/translate.generic.txt");
const TRANSLATE_ZH_EN: &str = include_str!("../../prompts/translate.zh-en.txt");
const TRANSLATE_ZH_EN_QWEN: &str = include_str!("../../prompts/translate.zh-en.qwen.txt");
pub const VERIFY: &str = include_str!("../../prompts/verify.txt");

const TONE_STANDARD: &str = include_str!("../../prompts/tones/standard.txt");
const TONE_XIANXIA: &str = include_str!("../../prompts/tones/xianxia.txt");
const TONE_WUXIA: &str = include_str!("../../prompts/tones/wuxia.txt");
const TONE_COMEDIC: &str = include_str!("../../prompts/tones/comedic.txt");
const TONE_FUNNY: &str = include_str!("../../prompts/tones/funny.txt");

fn tone_text(tone: Tone) -> &'static str {
    match tone {
        Tone::Standard => TONE_STANDARD,
        Tone::Xianxia => TONE_XIANXIA,
        Tone::Wuxia => TONE_WUXIA,
        Tone::Comedic => TONE_COMEDIC,
        Tone::Funny => TONE_FUNNY,
    }
}

/// Template selection: `translate.{src}-{tgt}.{variant}.txt` →
/// `translate.{src}-{tgt}.txt` → `translate.generic.txt`. `variant` comes from
/// `Connection.prompt_template` (e.g. "qwen").
fn template(pair: &LanguagePair, variant: Option<&str>) -> &'static str {
    match (pair.source.as_str(), pair.target.as_str(), variant) {
        ("zh", "en", Some("qwen")) => TRANSLATE_ZH_EN_QWEN,
        ("zh", "en", _) => TRANSLATE_ZH_EN,
        _ => TRANSLATE_GENERIC,
    }
}

/// Fill all known placeholders for the selected template. Each template uses
/// its own placeholder names (see module doc); we apply all substitutions and
/// any that don't appear in the chosen template are simply no-ops. The templates
/// define both UPPERCASE and lowercase variants; we fill both to prevent latent
/// placeholder leaks (`core/batch_translator.py:433-445` only fills UPPERCASE).
pub fn system_prompt(
    pair: &LanguagePair,
    glossary: &Glossary,
    tone: Tone,
    variant: Option<&str>,
) -> String {
    let tone_str = tone_text(tone);
    let glossary_str = glossary.to_formatted_string();

    template(pair, variant)
        // translate.zh-en.txt placeholders
        .replace("{GLOSSARY}", &glossary_str)
        .replace("{TONE}", tone_str)
        // translate.generic.txt placeholders (lowercase)
        .replace("{source_language}", &pair.source_name)
        .replace("{target_language}", &pair.target_name)
        .replace("{localization_style}", tone_str)
        // translate.zh-en.qwen.txt placeholders
        .replace("{glossary_formatted}", &glossary_str)
}

/// Build the user prompt. `lines` are (id, marked+hinted src). `context` is
/// up to the last 7 carried (src, tgt) pairs
/// (`batch_translator.py:469-484`).
pub fn user_prompt(lines: &[(u32, String)], context: &[(String, String)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !context.is_empty() {
        parts.push("Previous translations for context:".into());
        for (src, tgt) in context.iter().rev().take(7).rev() {
            parts.push(format!("  {src} -> {tgt}"));
        }
        parts.push(String::new());
    }
    parts.push("Translate the following lines:".into());
    let arr: Vec<serde_json::Value> = lines
        .iter()
        .map(|(id, src)| serde_json::json!({ "id": id, "src": src }))
        .collect();
    parts.push(serde_json::to_string_pretty(&arr).expect("serializable"));
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::projects::Tone;
    use crate::glossary::model::Glossary;
    use crate::models::language_pair::LanguagePair;

    fn pair() -> LanguagePair {
        LanguagePair::from_codes("zh", "en").unwrap()
    }

    /// translate.zh-en.txt contains {GLOSSARY} and {TONE} only —
    /// not {SOURCE_LANGUAGE} or {TARGET_LANGUAGE}.
    #[test]
    fn system_prompt_fills_placeholders() {
        let mut g = Glossary::new("xianxia");
        g.characters.insert("星汉".into(), "Xinghan".into());
        let p = system_prompt(&pair(), &g, Tone::Xianxia, None);
        assert!(!p.contains("{GLOSSARY}"));
        assert!(!p.contains("{TONE}"));
        assert!(p.contains("星汉 → Xinghan"));
    }

    /// translate.generic.txt uses {source_language} / {target_language} (lowercase).
    #[test]
    fn unknown_pair_falls_back_to_generic() {
        let pair = LanguagePair::from_codes("ko", "en").unwrap();
        let p = system_prompt(&pair, &Glossary::default(), Tone::Standard, None);
        assert!(p.contains("Korean"));
        assert!(p.contains("English"));
    }

    #[test]
    fn qwen_template_selected_when_requested() {
        let p = system_prompt(&pair(), &Glossary::default(), Tone::Standard, Some("qwen"));
        let generic = system_prompt(&pair(), &Glossary::default(), Tone::Standard, None);
        assert_ne!(p, generic);
    }

    #[test]
    fn user_prompt_carries_context_and_lines() {
        let ctx = vec![("前文".to_string(), "Earlier line".to_string())];
        let lines = vec![(1u32, "<0001:D> 你好".to_string())];
        let p = user_prompt(&lines, &ctx);
        assert!(p.contains("Previous translations for context:"));
        assert!(p.contains("前文 -> Earlier line"));
        assert!(p.contains("Translate the following lines:"));
        assert!(p.contains(r#""id": 1"#));
        assert!(p.contains("<0001:D> 你好"));
    }

    #[test]
    fn user_prompt_without_context_skips_header() {
        let p = user_prompt(&[(1, "<0001:D> 你好".into())], &[]);
        assert!(!p.contains("Previous translations"));
    }
}
