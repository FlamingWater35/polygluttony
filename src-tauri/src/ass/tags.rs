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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_override_blocks() {
        assert_eq!(strip_for_text("{\\pos(1,2)\\an5}Episode Title"), "Episode Title");
        assert_eq!(strip_for_text("{\\i1}斜体{\\i0}文字"), "斜体文字");
        assert_eq!(strip_for_text("plain text"), "plain text");
    }
}
