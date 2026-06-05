//! Strip ASS inline override tags (`{\pos(..)}`, `{\an8}`, `{\i1}`) from dialogue
//! text, leaving plain words for language + world detection. (The positional
//! strip/reapply needed for translation output is a later step.)

use std::sync::LazyLock;

use regex::Regex;

static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{[^}]*\}").unwrap());

/// Remove all `{...}` override blocks, returning the plain text.
pub fn strip_for_text(s: &str) -> String {
    TAG_RE.replace_all(s, "").into_owned()
}

/// Matches only ASS override tag blocks: `{` followed by `\` then non-`}` chars then `}`.
/// Plain brace groups like `{note}` do not match (`ass_tags.py:68`).
/// Note: positions are tracked in CHARS (not bytes) unlike Python's `original_position`
/// which is a code-point (character) offset from `match.start()`. Both yield the same result for stripped_position
/// because Python's `_calculate_stripped_position` also slices by chars via str indexing.
static TAG: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\{\\[^}]+\}").unwrap());

/// One override tag with where it sat in the original and stripped text.
/// Positions are in CHARS (not bytes) — translations are sliced by chars.
/// (`ass_tags.py` `AssTag` dataclass, lines 7-18)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedTag {
    pub content: String,
    /// Char offset of the tag in the original text.
    pub original_position: usize,
    /// Char offset where the tag would sit in the stripped text (all preceding tags removed).
    pub stripped_position: usize,
}

/// Result of a positional strip: original text, stripped text, and tags with positions.
/// (`ass_tags.py` `AssTagResult` dataclass, lines 25-37)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagStrip {
    pub original: String,
    pub stripped: String,
    pub tags: Vec<PlacedTag>,
}

/// Strip `{\...}` override blocks recording char positions (`ass_tags.py:70-107`).
///
/// Unlike Python's `_calculate_stripped_position` which re-strips the prefix each iteration,
/// we maintain a running `removed_chars` counter — identical result, O(n) vs O(n²).
pub fn strip_positional(text: &str) -> TagStrip {
    let mut tags = Vec::new();
    let mut removed_chars = 0usize;
    for m in TAG.find_iter(text) {
        let original_position = text[..m.start()].chars().count();
        tags.push(PlacedTag {
            content: m.as_str().to_string(),
            original_position,
            stripped_position: original_position - removed_chars,
        });
        removed_chars += m.as_str().chars().count();
    }
    TagStrip {
        original: text.to_string(),
        stripped: TAG.replace_all(text, "").to_string(),
        tags,
    }
}

/// Reinsert tags into `translated`: position 0 → prepend, ≥ stripped end →
/// append, mid-text → proportional (`ass_tags.py:109-150`).
///
/// `already_inserted` tracks cumulative char count of tags already spliced in,
/// mirroring Python's `inserted_length` (`ass_tags.py:137`).
pub fn reapply(strip: &TagStrip, translated: &str) -> String {
    if strip.tags.is_empty() {
        return translated.to_string();
    }
    let original_len = strip.stripped.chars().count();
    if original_len == 0 {
        // All tags prefix the translation (`ass_tags.py:131-133`).
        let prefix: String = strip.tags.iter().map(|t| t.content.as_str()).collect();
        return format!("{prefix}{translated}");
    }
    let translated_len = translated.chars().count();
    let mut out: Vec<char> = translated.chars().collect();
    let mut already_inserted = 0usize;
    for tag in &strip.tags {
        // Mirror Python's _calculate_insert_position (`ass_tags.py:189-219`).
        let insert_pos = if tag.stripped_position == 0 {
            // Tags at start: place right after any previously-inserted prefix tags.
            // Python: `return already_inserted` (`ass_tags.py:208-209`).
            already_inserted.min(out.len())
        } else if tag.stripped_position >= original_len {
            // Tags at or beyond end: append after all real translated chars + inserted tags.
            // Python: `return translated_length + already_inserted` (`ass_tags.py:212-213`).
            out.len()
        } else {
            // Proportional positioning (`ass_tags.py:216-219`).
            let ratio = tag.stripped_position as f64 / original_len as f64;
            let calc = (ratio * translated_len as f64).round() as usize;
            (calc + already_inserted).min(out.len())
        };
        let tag_chars: Vec<char> = tag.content.chars().collect();
        already_inserted += tag_chars.len();
        out.splice(insert_pos..insert_pos, tag_chars);
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_override_blocks() {
        assert_eq!(strip_for_text("{\\pos(1,2)\\an5}Episode Title"), "Episode Title");
        assert_eq!(strip_for_text("{\\i1}斜体{\\i0}文字"), "斜体文字");
        assert_eq!(strip_for_text("plain text"), "plain text");
    }

    #[test]
    fn strip_records_positions() {
        let r = strip_positional(r"{\pos(100,200)}Episode Title");
        assert_eq!(r.stripped, "Episode Title");
        assert_eq!(r.tags.len(), 1);
        assert_eq!(r.tags[0].content, r"{\pos(100,200)}");
        assert_eq!(r.tags[0].stripped_position, 0);
    }

    #[test]
    fn strip_mid_text_tag_position() {
        let r = strip_positional(r"Hello {\i1}world");
        assert_eq!(r.stripped, "Hello world");
        assert_eq!(r.tags[0].stripped_position, 6);
    }

    #[test]
    fn reapply_prefix_tag() {
        let r = strip_positional(r"{\an8}你好");
        assert_eq!(reapply(&r, "Hello"), r"{\an8}Hello");
    }

    #[test]
    fn reapply_mid_text_is_proportional() {
        // Tag at 5/10 chars of the stripped source → middle of the translation.
        let r = strip_positional(r"01234{\i1}56789");
        assert_eq!(r.tags[0].stripped_position, 5);
        assert_eq!(reapply(&r, "abcdefgh"), r"abcd{\i1}efgh");
    }

    #[test]
    fn reapply_end_tag_appends() {
        let r = strip_positional(r"你好{\r}");
        assert_eq!(reapply(&r, "Hello"), r"Hello{\r}");
    }

    #[test]
    fn reapply_on_empty_stripped_text_prefixes_all_tags() {
        let r = strip_positional(r"{\pos(1,2)}{\an8}");
        assert_eq!(r.stripped, "");
        assert_eq!(reapply(&r, "Hi"), r"{\pos(1,2)}{\an8}Hi");
    }

    #[test]
    fn non_override_braces_are_not_tags() {
        let r = strip_positional("{note} hello");
        assert!(r.tags.is_empty());
        assert_eq!(r.stripped, "{note} hello");
    }

    #[test]
    fn no_tags_roundtrip() {
        let r = strip_positional("plain");
        assert_eq!(reapply(&r, "translated"), "translated");
    }

    #[test]
    fn reapply_two_tags_accumulates_offsets() {
        let r = strip_positional(r"{\an8}ab{\i1}cd");
        assert_eq!(r.stripped, "abcd");
        assert_eq!(reapply(&r, "WXYZ"), r"{\an8}WX{\i1}YZ");
    }
}
