//! Glossary prompt assembly. Port of `glossary_builder.py:378-439` (+ the
//! normalize/personalize prompt loads at 459-467, 542-553).
//!
//! Python bug fixed here: `glossary.txt` uses lowercase `{world_type}` but the
//! builder only ever replaced `{WORLD_TYPE}` — the placeholder reached the LLM
//! verbatim. We fill BOTH cases of every placeholder (same fix as
//! `translation/prompts.rs`).

use std::collections::BTreeMap;

use crate::glossary::model::Glossary;
use crate::glossary::reference::ReferenceTerminology;
use crate::models::language_pair::LanguagePair;

const GLOSSARY: &str = include_str!("../../prompts/glossary.txt");
const GLOSSARY_QWEN: &str = include_str!("../../prompts/glossary.qwen.txt");
const NORM_CHARACTERS: &str = include_str!("../../prompts/glossary-normalize-characters.txt");
const NORM_CULTIVATION: &str = include_str!("../../prompts/glossary-normalize-cultivation.txt");
const NORM_SKILLS: &str = include_str!("../../prompts/glossary-normalize-skills.txt");
const NORM_LOCATIONS: &str = include_str!("../../prompts/glossary-normalize-locations.txt");
const NORM_ITEMS: &str = include_str!("../../prompts/glossary-normalize-items.txt");
const NORM_ORGANIZATIONS: &str =
    include_str!("../../prompts/glossary-normalize-organizations.txt");
const PERSONALIZE: &str = include_str!("../../prompts/glossary-personalize.txt");
pub const REFERENCE_EXTRACT: &str = include_str!("../../prompts/reference-extract.txt");

/// Strip the first `## {heading}` section (everything from that heading up to
/// the next `##` heading, or end of string). The `regex` crate does not support
/// lookaheads, so we implement this with plain string search. This is a no-op
/// when the section is absent.
///
/// Note: because we search for `"\n##"`, a `###` subsection also terminates
/// the strip early — matching Python's `(?=##|\Z)` lookahead behaviour.
fn strip_section(text: &str, heading: &str) -> String {
    let needle = format!("## {heading}");
    let Some(start) = text.find(&needle) else {
        return text.to_string();
    };
    // Find the next `##` after the section start (skip past the heading itself).
    let after_heading = start + needle.len();
    let end = text[after_heading..]
        .find("\n##")
        // `+ 1` skips the '\n' so the next `##` heading starts flush at column 0.
        .map(|pos| after_heading + pos + 1)
        .unwrap_or(text.len());
    format!("{}{}", &text[..start], &text[end..])
}

/// System prompt for one extraction batch. `variant` comes from
/// `Connection.prompt_template` ("qwen" selects `glossary.qwen.txt`).
pub fn extraction_prompt(
    world: &str,
    pair: &LanguagePair,
    reference: Option<&ReferenceTerminology>,
    variant: Option<&str>,
) -> String {
    let template = match variant {
        Some("qwen") => GLOSSARY_QWEN,
        _ => GLOSSARY,
    };
    let mut p = template
        .replace("{WORLD_TYPE}", world)
        .replace("{world_type}", world)
        // Not present in today's templates; filled for forward-compat
        // (Python replaced the uppercase pair too).
        .replace("{SOURCE_LANGUAGE}", &pair.source_name)
        .replace("{source_language}", &pair.source_name)
        .replace("{TARGET_LANGUAGE}", &pair.target_name)
        .replace("{target_language}", &pair.target_name);
    // Build path never injects established terms (glossary_builder.py:274-280
    // hardcodes context=None) — strip the section unconditionally.
    p = strip_section(&p, "ESTABLISHED TERMINOLOGY");
    p = match reference {
        Some(r) if !r.is_empty() => p.replace("{reference_terminology}", &r.to_prompt_string()),
        _ => strip_section(&p, "REFERENCE TERMINOLOGY"),
    };
    p
}

pub fn extraction_user_prompt(batch: &str) -> String {
    format!("Extract terms from this text:\n\n{batch}")
}

/// Per-category normalize prompt (`glossary_builder.py:459-467`).
pub fn normalize_prompt(category: &str, world: &str) -> String {
    let template = match category {
        "characters" => NORM_CHARACTERS,
        "cultivation" => NORM_CULTIVATION,
        "skills" => NORM_SKILLS,
        "locations" => NORM_LOCATIONS,
        "items" => NORM_ITEMS,
        "organizations" => NORM_ORGANIZATIONS,
        _ => unreachable!("unknown glossary category: {category}"),
    };
    template.replace("{WORLD_TYPE}", world).replace("{world_type}", world)
}

/// User prompt = the category's terms as pretty JSON
/// (`glossary_builder.py:467`).
pub fn normalize_user_prompt(terms: &BTreeMap<String, String>) -> String {
    serde_json::to_string_pretty(terms).expect("serializable")
}

/// Personalize prompt: `{donghua_title}` = first context line or "Unknown"
/// (`glossary_builder.py:548-553`).
pub fn personalize_prompt(world: &str, context: &str) -> String {
    let title =
        context.lines().next().map(str::trim).filter(|t| !t.is_empty()).unwrap_or("Unknown");
    PERSONALIZE
        .replace("{donghua_title}", title)
        .replace("{DONGHUA_TITLE}", title)
        .replace("{world_type}", world)
        .replace("{WORLD_TYPE}", world)
}

/// `glossary_builder.py:554-556`.
pub fn personalize_user_prompt(glossary: &Glossary, context: &str) -> String {
    let mut u = format!("Personalize this glossary:\n\n{}", glossary.to_json_pretty());
    if !context.is_empty() {
        u.push_str(&format!("\n\n## Additional Context\n\n{context}"));
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::reference::ReferenceTerminology;
    use crate::models::language_pair::LanguagePair;

    fn pair() -> LanguagePair {
        LanguagePair::from_codes("zh", "en").unwrap()
    }

    #[test]
    fn extraction_prompt_fills_both_cases_and_strips_established() {
        let p = extraction_prompt("wuxia", &pair(), None, None);
        assert!(!p.contains("{world_type}"), "lowercase placeholder must be filled");
        assert!(!p.contains("{WORLD_TYPE}"));
        assert!(p.contains("wuxia"));
        // Established section always stripped (build never passes context).
        assert!(!p.contains("ESTABLISHED TERMINOLOGY"));
        assert!(!p.contains("{established_terms}"));
        // No reference terms → reference section stripped too.
        assert!(!p.contains("REFERENCE TERMINOLOGY"));
        assert!(!p.contains("{reference_terminology}"));
    }

    #[test]
    fn extraction_prompt_injects_reference_terms() {
        let r = ReferenceTerminology {
            characters: vec!["Lin Dong".into()],
            ..Default::default()
        };
        let p = extraction_prompt("xianxia", &pair(), Some(&r), None);
        assert!(p.contains("## REFERENCE TERMINOLOGY"));
        assert!(p.contains("CHARACTER NAMES: Lin Dong"));
        assert!(!p.contains("{reference_terminology}"));
    }

    #[test]
    fn qwen_variant_differs_unknown_falls_back() {
        let base = extraction_prompt("xianxia", &pair(), None, None);
        let qwen = extraction_prompt("xianxia", &pair(), None, Some("qwen"));
        let unknown = extraction_prompt("xianxia", &pair(), None, Some("nope"));
        assert_ne!(base, qwen);
        assert_eq!(base, unknown);
        // No raw placeholders should survive substitution in the qwen variant.
        assert!(!qwen.contains("{established_terms}"));
        assert!(!qwen.contains("{world_type}"));
    }

    // --- Direct unit tests for the private `strip_section` helper ---

    #[test]
    fn strip_section_middle() {
        // Section is between two others; the prefix keeps its content and the
        // following heading starts flush immediately after the splice point.
        let text = "## INTRO\nkeep this\n## REMOVE ME\ndrop this\n## OUTRO\nkeep too";
        let result = strip_section(text, "REMOVE ME");
        assert!(result.contains("## INTRO\nkeep this\n"), "prefix intact");
        assert!(!result.contains("REMOVE ME"), "section heading gone");
        assert!(!result.contains("drop this"), "section body gone");
        assert!(result.contains("## OUTRO\nkeep too"), "suffix intact, flush");
    }

    #[test]
    fn strip_section_at_end() {
        // Section runs to the end of the string — exercises the `unwrap_or(text.len())` arm.
        let text = "## FIRST\nkeep\n## LAST\ndrop everything here";
        let result = strip_section(text, "LAST");
        assert!(result.contains("## FIRST\nkeep\n"), "prefix intact");
        assert!(!result.contains("LAST"), "section heading gone");
        assert!(!result.contains("drop everything here"), "trailing body gone");
        // Nothing after the stripped section.
        assert!(result.ends_with("keep\n") || result.ends_with("keep"), "no trailing junk");
    }

    #[test]
    fn strip_section_heading_not_found() {
        // When the heading is absent the input must be returned unchanged.
        let text = "## ALPHA\nsome text\n## BETA\nmore text";
        assert_eq!(strip_section(text, "GAMMA"), text);
    }

    #[test]
    fn strip_section_heading_at_offset_zero() {
        // The target heading is the very first thing in the string.
        let text = "## REMOVE\ndrop\n## KEEP\nthis stays";
        let result = strip_section(text, "REMOVE");
        assert!(!result.contains("REMOVE"), "leading section gone");
        assert!(!result.contains("drop"), "leading body gone");
        assert!(result.starts_with("## KEEP"), "next heading now at start");
        assert!(result.contains("this stays"), "trailing content intact");
    }

    #[test]
    fn strip_section_only_first_occurrence_removed() {
        // Deliberate divergence from Python's re.sub-all: only the FIRST matching
        // section is stripped; a second identical heading survives untouched.
        let text = "## DUP\nfirst body\n## OTHER\nmiddle\n## DUP\nsecond body";
        let result = strip_section(text, "DUP");
        assert!(!result.contains("first body"), "first instance stripped");
        assert!(result.contains("## DUP\nsecond body"), "second instance preserved");
    }

    #[test]
    fn normalize_prompts_exist_for_all_categories_and_fill_world() {
        for c in crate::glossary::model::CATEGORIES {
            let p = normalize_prompt(c, "xianxia");
            assert!(!p.contains("{world_type}"), "{c}: lowercase filled");
            assert!(!p.contains("{WORLD_TYPE}"), "{c}: uppercase filled");
        }
    }

    #[test]
    fn normalize_user_prompt_is_pretty_json_of_terms() {
        let mut terms = std::collections::BTreeMap::new();
        terms.insert("林动".to_string(), "Lin Dong".to_string());
        let u = normalize_user_prompt(&terms);
        assert!(u.contains("\"林动\": \"Lin Dong\""));
    }

    #[test]
    fn personalize_prompt_uses_first_context_line_as_title() {
        let p = personalize_prompt("xianxia", "Martial Universe\nextra notes");
        assert!(p.contains("Martial Universe"));
        assert!(!p.contains("{donghua_title}"));
        assert!(!p.contains("{world_type}"));
        let p2 = personalize_prompt("modern", "");
        assert!(p2.contains("Unknown"));
    }

    #[test]
    fn personalize_user_prompt_appends_context_section() {
        let mut g = crate::glossary::model::Glossary::new("xianxia");
        g.characters.insert("林动".into(), "Lin Dong".into());
        let u = personalize_user_prompt(&g, "Martial Universe\nwiki: …");
        assert!(u.starts_with("Personalize this glossary:"));
        assert!(u.contains("Lin Dong"));
        assert!(u.contains("## Additional Context"));
        let bare = personalize_user_prompt(&g, "");
        assert!(!bare.contains("## Additional Context"));
    }

    #[test]
    fn extraction_user_prompt_wraps_batch() {
        assert_eq!(
            extraction_user_prompt("line1\nline2"),
            "Extract terms from this text:\n\nline1\nline2"
        );
    }
}
