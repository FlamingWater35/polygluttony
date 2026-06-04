# Step 2 — Welcome screen + Project view (folder pickup) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the read-only folder-pickup experience — a Welcome landing screen and a per-folder Project setup view — backed by a new Rust read engine (charset-robust ASS reader, world detection, languages, file discovery) and a backend project store (recents + per-folder prefs).

**Architecture:** New pure Rust modules under `src-tauri/src/` (`ass/`, `glossary/world_detector`, `config/languages`, `config/projects`, `models/language_pair`, `utils/discover`) are exercised by thin `#[tauri::command]`s in `commands/project.rs`; one `open_folder` command bundles discovery + counts + world/language detection + recents/prefs. ts-rs generates TS types. React views (`features/welcome`, `features/project`) call the commands via `lib/ipc.ts`, sync shell state into the Zustand store, and the nav-rail/status-bar reflect it. No file writing — the translated-output writer is Step 3.

**Tech Stack:** Rust (encoding_rs, chardetng, regex, serde, ts-rs, tauri), React 19 + TypeScript, TanStack Router/Query, Zustand, Tauri plugins (dialog, store), sonner.

**Spec:** `docs/superpowers/specs/2026-06-04-step2-welcome-and-project-design.md`

---

## Conventions for every task

- **Run Rust tests:** `cargo test --manifest-path src-tauri/Cargo.toml <filter>` (omit `<filter>` to run all). Working dir = repo root `/Users/user/Code/subs/polygluttony`.
- **Build the backend:** `cargo build --manifest-path src-tauri/Cargo.toml`.
- **Frontend has no JS unit-test runner** (matches Step 1). Frontend tasks verify with `bun run build` (route-gen → `tsc` → vite) and a manual smoke note. Write no fabricated JS tests.
- The crate root sets `#![allow(dead_code)]`, so code added in an early task that's only *used* by a later task will not fail the build.
- **Commit** at the end of each task. End every commit message body with:
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`
- Branch is already `step2-welcome-and-project`. Do not switch branches.

---

## Task 1: Dependencies + charset-robust file decode (`ass::decode`)

**Files:**
- Modify: `src-tauri/Cargo.toml` (add deps)
- Modify: `src-tauri/src/ass/mod.rs` (declare submodule)
- Create: `src-tauri/src/ass/decode.rs`

- [ ] **Step 1: Add dependencies**

In `src-tauri/Cargo.toml`, under `[dependencies]` add (after the `async-trait = "0.1"` line):

```toml
encoding_rs = "0.8"
chardetng = "0.1"
```

Under `[dev-dependencies]` (after `wiremock = "0.6"`) add:

```toml
tempfile = "3"
```

- [ ] **Step 2: Declare the submodule**

Append to `src-tauri/src/ass/mod.rs` (after the existing doc comment):

```rust

pub mod decode;
```

- [ ] **Step 3: Write the failing test**

Create `src-tauri/src/ass/decode.rs` with ONLY the test module first:

```rust
//! Charset-robust decoding of subtitle files. The Python parser assumed UTF-8;
//! donghua/anime `.ass` rips are frequently GBK/Big5/UTF-16, so we detect the
//! encoding (honoring a BOM, else chardetng) and decode with encoding_rs.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_utf8() {
        assert_eq!(decode_bytes("Hello 世界".as_bytes()), "Hello 世界");
    }

    #[test]
    fn strips_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("Hello".as_bytes());
        assert_eq!(decode_bytes(&bytes), "Hello");
    }

    #[test]
    fn decodes_utf16le_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        for u in "Hi 世界".encode_utf16() {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
        assert_eq!(decode_bytes(&bytes), "Hi 世界");
    }

    #[test]
    fn decodes_gbk_chinese() {
        // A long-enough Chinese sample so chardetng reliably picks GBK/GB18030.
        let sample = "修仙者突破了金丹期，灵气充沛，准备渡劫飞升。武林盟主召集江湖各大门派，讨论轻功和内力修炼之道。";
        let (bytes, _, _) = encoding_rs::GBK.encode(sample);
        assert_eq!(decode_bytes(&bytes), sample);
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::decode`
Expected: FAIL to compile — `cannot find function decode_bytes`.

- [ ] **Step 5: Implement**

Insert above the `#[cfg(test)]` module in `src-tauri/src/ass/decode.rs`:

```rust
use std::path::Path;

use chardetng::EncodingDetector;
use encoding_rs::Encoding;

use crate::error::AppResult;

/// Decode raw subtitle bytes to a `String`. Honors a UTF-8/UTF-16 BOM if present;
/// otherwise sniffs the encoding with chardetng. Malformed sequences are replaced
/// (never errors), so a best-effort string always comes back.
pub fn decode_bytes(bytes: &[u8]) -> String {
    let encoding = match Encoding::for_bom(bytes) {
        Some((enc, _bom_len)) => enc,
        None => {
            let mut detector = EncodingDetector::new();
            detector.feed(bytes, true);
            detector.guess(None, true)
        }
    };
    // `decode` re-sniffs and strips a leading BOM that matches `encoding`.
    let (text, _enc, _had_errors) = encoding.decode(bytes);
    text.into_owned()
}

/// Read and decode an `.ass` file from disk.
pub fn decode_file(path: &Path) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    Ok(decode_bytes(&bytes))
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::decode`
Expected: PASS (4 tests). If `decodes_gbk_chinese` fails because chardetng guessed a different encoding, lengthen the Chinese `sample` string — short samples are ambiguous.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/ass/mod.rs src-tauri/src/ass/decode.rs
git commit -m "$(printf 'feat(ass): charset-robust file decoding (encoding_rs + chardetng)\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 2: Dialogue line parser (`ass::parse`)

**Files:**
- Modify: `src-tauri/src/ass/mod.rs`
- Create: `src-tauri/src/ass/parse.rs`

- [ ] **Step 1: Declare the submodule**

Append to `src-tauri/src/ass/mod.rs`:

```rust
pub mod parse;
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/ass/parse.rs` with the test module first:

```rust
//! Parse the `[Events]` section of an `.ass` file into dialogue lines. The ASS
//! `Dialogue:` format is `Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text`
//! — exactly nine commas before the free-form text, so we split on the first nine.

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello, world
Comment: 0,0:00:05.00,0:00:06.00,Default,,0,0,0,,not counted
Dialogue: 0,0:00:06.00,0:00:09.00,Default,,0,0,0,,{\\i1}第一集{\\i0}
";

    #[test]
    fn counts_only_dialogue_lines() {
        assert_eq!(dialogue_count(SAMPLE), 2);
    }

    #[test]
    fn preserves_commas_and_tags_in_text() {
        let d = parse_dialogues(SAMPLE);
        assert_eq!(d[0].text, "Hello, world");
        assert_eq!(d[1].text, "{\\i1}第一集{\\i0}");
    }

    #[test]
    fn parses_timestamp_to_centiseconds() {
        assert_eq!(parse_timestamp_cs("0:00:01.50"), 150);
        assert_eq!(parse_timestamp_cs("1:02:03.04"), (1 * 3600 + 2 * 60 + 3) * 100 + 4);
        assert_eq!(parse_timestamp_cs("0:00:00.5"), 50);
    }

    #[test]
    fn skips_malformed_dialogue() {
        let text = "[Events]\nDialogue: 0,0:00:01.00\n";
        assert_eq!(dialogue_count(text), 0);
    }

    #[test]
    fn ignores_dialogue_outside_events() {
        let text = "[Script Info]\nDialogue: 0,0:00:01.00,0:00:02.00,D,,0,0,0,,x\n";
        assert_eq!(dialogue_count(text), 0);
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::parse`
Expected: FAIL to compile — missing `dialogue_count`, `parse_dialogues`, `parse_timestamp_cs`, `DialogueLine`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
/// One `Dialogue:` event. All ten fields are parsed (the Step-3 writer reuses
/// this); Step 2 reads `text` (count + detection) and `start_cs` (sorting later).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueLine {
    pub layer: i64,
    pub start_cs: i64,
    pub end_cs: i64,
    pub style: String,
    pub name: String,
    pub margin_l: i64,
    pub margin_r: i64,
    pub margin_v: i64,
    pub effect: String,
    pub text: String,
}

/// Parse `H:MM:SS.cc` into centiseconds. Returns 0 for unparseable input — a bad
/// timestamp shouldn't drop an otherwise-valid dialogue line.
pub fn parse_timestamp_cs(ts: &str) -> i64 {
    let mut parts = ts.trim().split(':');
    let h: i64 = parts.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let m: i64 = parts.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let sec = parts.next().unwrap_or("0").trim();
    let (s, cs) = match sec.split_once('.') {
        Some((s, frac)) => {
            let s: i64 = s.parse().unwrap_or(0);
            // First two fractional digits = centiseconds; right-pad short fractions.
            let frac2 = format!("{:0<2}", frac);
            let cs: i64 = frac2[..2].parse().unwrap_or(0);
            (s, cs)
        }
        None => (sec.parse().unwrap_or(0), 0),
    };
    ((h * 60 + m) * 60 + s) * 100 + cs
}

fn strip_dialogue_prefix(line: &str) -> Option<&str> {
    const PREFIX: &str = "dialogue:";
    if line.len() >= PREFIX.len() && line[..PREFIX.len()].eq_ignore_ascii_case(PREFIX) {
        Some(line[PREFIX.len()..].trim_start())
    } else {
        None
    }
}

/// Parse all `Dialogue:` lines within the `[Events]` section, in file order.
pub fn parse_dialogues(text: &str) -> Vec<DialogueLine> {
    let mut out = Vec::new();
    let mut in_events = false;
    for raw in text.lines() {
        let line = raw.trim_start();
        if line.starts_with('[') {
            in_events = line.trim_end().eq_ignore_ascii_case("[events]");
            continue;
        }
        if !in_events {
            continue;
        }
        let Some(rest) = strip_dialogue_prefix(line) else {
            continue;
        };
        let parts: Vec<&str> = rest.splitn(10, ',').collect();
        if parts.len() < 10 {
            continue; // malformed — don't count
        }
        out.push(DialogueLine {
            layer: parts[0].trim().parse().unwrap_or(0),
            start_cs: parse_timestamp_cs(parts[1]),
            end_cs: parse_timestamp_cs(parts[2]),
            style: parts[3].to_string(),
            name: parts[4].to_string(),
            margin_l: parts[5].trim().parse().unwrap_or(0),
            margin_r: parts[6].trim().parse().unwrap_or(0),
            margin_v: parts[7].trim().parse().unwrap_or(0),
            effect: parts[8].to_string(),
            text: parts[9].to_string(),
        });
    }
    out
}

/// Number of dialogue lines (mirrors Python `AssFile.get_dialogue_count`).
pub fn dialogue_count(text: &str) -> usize {
    parse_dialogues(text).len()
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::parse`
Expected: PASS (5 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ass/mod.rs src-tauri/src/ass/parse.rs
git commit -m "$(printf 'feat(ass): Dialogue line parser + timestamp parsing\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 3: Plain tag stripping (`ass::tags`)

**Files:**
- Modify: `src-tauri/src/ass/mod.rs`
- Create: `src-tauri/src/ass/tags.rs`

- [ ] **Step 1: Declare the submodule**

Append to `src-tauri/src/ass/mod.rs`:

```rust
pub mod tags;
```

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/ass/tags.rs` test module:

```rust
//! Strip ASS inline override tags (`{\pos(..)}`, `{\an8}`, `{\i1}`) from dialogue
//! text, leaving plain words for language + world detection. (The positional
//! strip/reapply needed for translation output is a later step.)

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
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::tags`
Expected: FAIL — `cannot find function strip_for_text`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
use std::sync::LazyLock;

use regex::Regex;

static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{[^}]*\}").unwrap());

/// Remove all `{...}` override blocks, returning the plain text.
pub fn strip_for_text(s: &str) -> String {
    TAG_RE.replace_all(s, "").into_owned()
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml ass::tags`
Expected: PASS. (If `std::sync::LazyLock` is unavailable on the toolchain, the build will say so; then construct the `Regex` inside the function instead.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ass/mod.rs src-tauri/src/ass/tags.rs
git commit -m "$(printf 'feat(ass): plain override-tag stripping for detection text\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 4: World-type detector (`glossary::world_detector`)

**Files:**
- Modify: `src-tauri/src/glossary/mod.rs`
- Create: `src-tauri/src/glossary/world_detector.rs`

- [ ] **Step 1: Declare the submodule**

Append to `src-tauri/src/glossary/mod.rs`:

```rust

pub mod world_detector;
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/glossary/world_detector.rs` test module:

```rust
//! Instant, keyword-only world-type detection (no LLM). Ported from
//! `glossary/world_detector.py`. Counts Chinese keyword occurrences per category;
//! the highest wins (ties: xianxia > wuxia > historical); none → modern.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_world() {
        assert_eq!(detect("修仙者突破了金丹期，灵气充沛，准备渡劫飞升", true), WorldType::Xianxia);
        assert_eq!(detect("武林盟主召集江湖各大门派，讨论轻功和内力修炼", true), WorldType::Wuxia);
        assert_eq!(detect("皇帝在朝廷上接见了宰相和大臣，商议科举事宜", true), WorldType::Historical);
        assert_eq!(detect("今天天气不错，我们去公园散步吧", true), WorldType::Modern);
    }

    #[test]
    fn no_detection_when_unsupported() {
        assert_eq!(detect("修仙金丹渡劫", false), WorldType::Modern);
    }

    #[test]
    fn ties_break_in_priority_order() {
        // one xianxia + one wuxia keyword → xianxia wins the tie.
        assert_eq!(detect("修仙 江湖", true), WorldType::Xianxia);
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml world_detector`
Expected: FAIL — missing `detect`, `WorldType`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Detected story world. Tunes glossary extraction + tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum WorldType {
    Xianxia,
    Wuxia,
    Historical,
    Modern,
}

const XIANXIA: &[&str] = &[
    "修仙", "筑基", "金丹", "元婴", "渡劫", "灵气", "仙人", "修炼", "灵石", "丹药",
    "法宝", "飞剑", "结丹", "化神", "修真", "仙界", "魔界", "灵根", "天劫", "飞升",
];
const WUXIA: &[&str] = &[
    "武林", "江湖", "门派", "内力", "轻功", "武功", "剑法", "掌法", "拳法", "气功",
    "真气", "武者", "侠客", "大侠", "盟主", "帮派",
];
const HISTORICAL: &[&str] = &[
    "皇帝", "朝廷", "太监", "皇后", "大臣", "科举", "宰相", "王爷", "公主", "皇宫",
    "后宫", "朝代", "太子", "皇上", "圣旨",
];

fn count(content: &str, keywords: &[&str]) -> usize {
    keywords.iter().map(|kw| content.matches(kw).count()).sum()
}

/// Detect the world type from combined dialogue `content`. When the source
/// language doesn't support detection, returns `Modern` without scanning.
pub fn detect(content: &str, supports_world_detection: bool) -> WorldType {
    if !supports_world_detection {
        return WorldType::Modern;
    }
    let x = count(content, XIANXIA);
    let w = count(content, WUXIA);
    let h = count(content, HISTORICAL);
    let max = x.max(w).max(h);
    if max == 0 {
        WorldType::Modern
    } else if x == max {
        WorldType::Xianxia
    } else if w == max {
        WorldType::Wuxia
    } else {
        WorldType::Historical
    }
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml world_detector`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/glossary/mod.rs src-tauri/src/glossary/world_detector.rs
git commit -m "$(printf 'feat(glossary): keyword-only world-type detector (WorldType)\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 5: Language table (`config::languages`)

**Files:**
- Modify: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/languages.rs`

- [ ] **Step 1: Declare the submodule**

In `src-tauri/src/config/mod.rs`, add after `pub mod presets;` / `pub mod store;`:

```rust
pub mod languages;
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/config/languages.rs` test module:

```rust
//! Supported languages + alias resolution + source-language detection.
//! Ported from `config/languages.py`.

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
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::languages`
Expected: FAIL — missing `Language`, `languages`, `resolve_language_code`, `detect_source_language`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
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
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::languages`
Expected: PASS (3 tests). (zh/ja both match Han; zh is listed first and ties don't replace, so Chinese text resolves to `zh`.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/config/mod.rs src-tauri/src/config/languages.rs
git commit -m "$(printf 'feat(config): language table + alias resolution + source detection\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 6: Language pair + filename helpers (`models::language_pair`)

**Files:**
- Modify: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/language_pair.rs`

- [ ] **Step 1: Declare the submodule**

In `src-tauri/src/models/mod.rs`, add after the existing `use` lines (near the top, after `use crate::config::Driver;`):

```rust
pub mod language_pair;
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/models/language_pair.rs` test module:

```rust
//! Source/target language pair with validation + output-filename helpers.
//! Ported from `models/language_pair.py`.

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
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml language_pair`
Expected: FAIL — missing `LanguagePair`, `output_filename`, `warning_filename`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
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
    pub supports_glossary: bool,
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
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml language_pair`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/models/mod.rs src-tauri/src/models/language_pair.rs
git commit -m "$(printf 'feat(models): LanguagePair validation + output filename helpers\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 7: File discovery (`utils::discover`)

**Files:**
- Modify: `src-tauri/src/lib.rs` (declare `utils` module)
- Create: `src-tauri/src/utils/mod.rs`
- Create: `src-tauri/src/utils/discover.rs`

- [ ] **Step 1: Declare the module**

In `src-tauri/src/lib.rs`, add to the engine module block (after `mod ass;`):

```rust
mod utils;
```

Create `src-tauri/src/utils/mod.rs`:

```rust
//! Filesystem helpers for the engine.

pub mod discover;
```

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/utils/discover.rs` test module:

```rust
//! Discover source `.ass` files in a folder and detect existing translations.
//! Ported from `utils/file_utils.py`.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::language_pair::LanguagePair;
    use std::fs;

    #[test]
    fn discovers_sources_excluding_translations() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        for name in [
            "ep01.ass",
            "ep01.eng.ass",
            "ep02.ass",
            "ep03.ass",
            "ep03.warning.eng.ass",
            "notes.txt",
        ] {
            fs::write(p.join(name), "x").unwrap();
        }
        let pair = LanguagePair::from_codes("zh", "en").unwrap();

        let names: Vec<String> = discover_source_files(p, &pair)
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["ep01.ass", "ep02.ass", "ep03.ass"]);

        assert!(has_existing_translation(&p.join("ep01.ass"), &pair));
        assert!(has_existing_translation(&p.join("ep03.ass"), &pair));
        assert!(!has_existing_translation(&p.join("ep02.ass"), &pair));
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml discover`
Expected: FAIL — missing `discover_source_files`, `has_existing_translation`.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
use std::path::{Path, PathBuf};

use crate::models::language_pair::{output_filename, warning_filename, LanguagePair};

/// All `.ass` files in `dir` (non-recursive) that are *source* files — not
/// themselves translation outputs. Sorted by path.
pub fn discover_source_files(dir: &Path, pair: &LanguagePair) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    let target_suffixes = pair.target_suffixes();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_ass = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("ass"))
            == Some(true);
        if !is_ass {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let lower = stem.to_lowercase();
        if lower.contains(".warning.") {
            continue;
        }
        let is_translation = target_suffixes
            .iter()
            .any(|suf| lower.ends_with(&format!(".{}", suf.to_lowercase())));
        if is_translation {
            continue;
        }
        files.push(path);
    }
    files.sort();
    files
}

/// Whether `source` already has a translation (output or warning file present).
pub fn has_existing_translation(source: &Path, pair: &LanguagePair) -> bool {
    let dir = source.parent().unwrap_or_else(|| Path::new("."));
    dir.join(output_filename(source, pair)).exists()
        || dir.join(warning_filename(source, pair)).exists()
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml discover`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/utils/mod.rs src-tauri/src/utils/discover.rs
git commit -m "$(printf 'feat(utils): .ass source discovery + existing-translation detection\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 8: Project store — recents + per-folder prefs (`config::projects`)

**Files:**
- Modify: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/projects.rs`

- [ ] **Step 1: Declare the submodule**

In `src-tauri/src/config/mod.rs`, add (after `pub mod languages;`):

```rust
pub mod projects;
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/config/projects.rs` test module:

```rust
//! Per-folder project state: recent folders + saved preferences. Persisted in its
//! own Tauri store file (`projects.json`), separate from `config.json`. Pure
//! helpers are unit-tested; the Tauri glue is thin.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::world_detector::WorldType;

    #[test]
    fn record_recent_is_mru_and_capped() {
        let mut c = ProjectsConfig::default();
        for i in 0..12 {
            record_recent(&mut c, &format!("/f{i}"), i as u32, i as i64);
        }
        assert_eq!(c.recents.len(), 10);
        assert_eq!(c.recents[0].path, "/f11"); // newest first
        record_recent(&mut c, "/f5", 5, 99);
        assert_eq!(c.recents[0].path, "/f5");
        assert_eq!(c.recents.iter().filter(|r| r.path == "/f5").count(), 1);
    }

    #[test]
    fn prune_drops_missing() {
        let mut c = ProjectsConfig::default();
        record_recent(&mut c, "/keep", 1, 1);
        record_recent(&mut c, "/gone", 1, 2);
        prune_recents(&mut c, |p| p == "/keep");
        assert_eq!(c.recents.len(), 1);
        assert_eq!(c.recents[0].path, "/keep");
    }

    #[test]
    fn prefs_round_trip_and_resolution() {
        let mut c = ProjectsConfig::default();
        assert!(get_prefs(&c, "/x").is_none());
        let p = FolderPrefs {
            source_lang: "zh".into(),
            target_lang: "en".into(),
            world_override: Some(WorldType::Wuxia),
            tone: Tone::Comedic,
            selected_files: vec!["a.ass".into()],
        };
        set_prefs(&mut c, "/x", p.clone());
        assert_eq!(get_prefs(&c, "/x"), Some(p.clone()));

        // saved prefs win outright
        assert_eq!(resolve_prefs(Some(p.clone()), Some("ja"), "zh", "en"), p);
        // unsaved: detection seeds source, global seeds target
        let r = resolve_prefs(None, Some("ja"), "zh", "en");
        assert_eq!(r.source_lang, "ja");
        assert_eq!(r.target_lang, "en");
        assert_eq!(r.world_override, None);
        assert_eq!(r.tone, Tone::Standard);
        // unsaved + no detection: global default source
        assert_eq!(resolve_prefs(None, None, "zh", "en").source_lang, "zh");
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::projects`
Expected: FAIL — missing types/functions.

- [ ] **Step 4: Implement**

Insert above the test module:

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;
use ts_rs::TS;

use crate::error::{AppError, AppResult};
use crate::glossary::world_detector::WorldType;

const STORE_FILE: &str = "projects.json";
const STORE_KEY: &str = "projects";
const RECENTS_CAP: usize = 10;

/// Translation tone (register). Persisted per folder; used by Translate (Step 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum Tone {
    Standard,
    Xianxia,
    Wuxia,
    Comedic,
    Funny,
}

impl Default for Tone {
    fn default() -> Self {
        Tone::Standard
    }
}

/// A folder's saved preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct FolderPrefs {
    pub source_lang: String,
    pub target_lang: String,
    pub world_override: Option<WorldType>,
    pub tone: Tone,
    /// Empty = all files selected.
    #[serde(default)]
    pub selected_files: Vec<String>,
}

/// One recent-folder entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct RecentFolder {
    pub path: String,
    pub file_count: u32,
    pub last_opened: i64,
}

/// The whole persisted projects document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectsConfig {
    #[serde(default)]
    pub recents: Vec<RecentFolder>,
    #[serde(default)]
    pub folders: BTreeMap<String, FolderPrefs>,
}

// ---- pure helpers ----------------------------------------------------------

/// Insert/refresh a recent entry at the front (MRU), capped at RECENTS_CAP.
pub fn record_recent(cfg: &mut ProjectsConfig, path: &str, file_count: u32, now: i64) {
    cfg.recents.retain(|r| r.path != path);
    cfg.recents.insert(
        0,
        RecentFolder {
            path: path.to_string(),
            file_count,
            last_opened: now,
        },
    );
    cfg.recents.truncate(RECENTS_CAP);
}

pub fn remove_recent(cfg: &mut ProjectsConfig, path: &str) {
    cfg.recents.retain(|r| r.path != path);
}

pub fn clear_recents(cfg: &mut ProjectsConfig) {
    cfg.recents.clear();
}

/// Drop recents whose path no longer satisfies `exists` (injected for testing).
pub fn prune_recents(cfg: &mut ProjectsConfig, exists: impl Fn(&str) -> bool) {
    cfg.recents.retain(|r| exists(&r.path));
}

pub fn get_prefs(cfg: &ProjectsConfig, path: &str) -> Option<FolderPrefs> {
    cfg.folders.get(path).cloned()
}

pub fn set_prefs(cfg: &mut ProjectsConfig, path: &str, prefs: FolderPrefs) {
    cfg.folders.insert(path.to_string(), prefs);
}

/// Resolve effective prefs for a freshly opened folder: saved prefs win; otherwise
/// seed source from detection → global default, target from global default.
pub fn resolve_prefs(
    saved: Option<FolderPrefs>,
    detected_source: Option<&str>,
    default_source: &str,
    default_target: &str,
) -> FolderPrefs {
    if let Some(p) = saved {
        return p;
    }
    FolderPrefs {
        source_lang: detected_source.unwrap_or(default_source).to_string(),
        target_lang: default_target.to_string(),
        world_override: None,
        tone: Tone::Standard,
        selected_files: Vec::new(),
    }
}

// ---- Tauri glue (thin; not unit-tested) ------------------------------------

pub fn load<R: Runtime>(app: &AppHandle<R>) -> AppResult<ProjectsConfig> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Other(e.to_string()))?;
    match store.get(STORE_KEY) {
        Some(value) => serde_json::from_value(value).map_err(AppError::from),
        None => Ok(ProjectsConfig::default()),
    }
}

pub fn save<R: Runtime>(app: &AppHandle<R>, cfg: &ProjectsConfig) -> AppResult<()> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set(STORE_KEY, serde_json::to_value(cfg)?);
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 5: Run to verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config::projects`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/config/mod.rs src-tauri/src/config/projects.rs
git commit -m "$(printf 'feat(config): project store — recents (MRU) + per-folder prefs\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 9: View DTOs + folder commands + registration + bindings

**Files:**
- Modify: `src-tauri/src/models/mod.rs` (add `SourceFile`, `ProjectView`)
- Create: `src-tauri/src/commands/project.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Add the view DTOs**

Append to `src-tauri/src/models/mod.rs`:

```rust

use crate::config::projects::FolderPrefs;
use crate::glossary::world_detector::WorldType;

/// One discovered source file.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct SourceFile {
    pub path: String,
    pub name: String,
    pub dialogue_count: u32,
    pub has_translation: bool,
}

/// Result of opening a folder (O6/O7/O8 bundled).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ProjectView {
    pub folder: String,
    pub files: Vec<SourceFile>,
    pub total_dialogue_lines: u32,
    pub detected_source_lang: Option<String>,
    pub detected_world: WorldType,
    pub prefs: FolderPrefs,
    pub supports_glossary: bool,
}
```

- [ ] **Step 2: Create the commands**

Create `src-tauri/src/commands/project.rs`:

```rust
//! Folder-pickup commands (O6/O7/O8) + per-folder persistence. Thin wrappers over
//! the tested engine modules; `open_folder` bundles discovery + counts + detection.

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::ass::{decode::decode_file, parse::parse_dialogues, tags::strip_for_text};
use crate::config::languages::{detect_source_language, languages, Language};
use crate::config::projects::{self, FolderPrefs, RecentFolder};
use crate::config::store as config_store;
use crate::error::{AppError, AppResult};
use crate::glossary::world_detector::detect;
use crate::models::language_pair::LanguagePair;
use crate::models::{ProjectView, SourceFile};
use crate::utils::discover::{discover_source_files, has_existing_translation};

#[tauri::command]
pub fn list_languages() -> Vec<Language> {
    languages()
}

#[tauri::command]
pub fn list_recents(app: AppHandle) -> AppResult<Vec<RecentFolder>> {
    let mut cfg = projects::load(&app)?;
    projects::prune_recents(&mut cfg, |p| Path::new(p).is_dir());
    projects::save(&app, &cfg)?;
    Ok(cfg.recents)
}

#[tauri::command]
pub fn remove_recent(app: AppHandle, path: String) -> AppResult<()> {
    let mut cfg = projects::load(&app)?;
    projects::remove_recent(&mut cfg, &path);
    projects::save(&app, &cfg)
}

#[tauri::command]
pub fn clear_recents(app: AppHandle) -> AppResult<()> {
    let mut cfg = projects::load(&app)?;
    projects::clear_recents(&mut cfg);
    projects::save(&app, &cfg)
}

#[tauri::command]
pub fn save_folder_prefs(app: AppHandle, path: String, prefs: FolderPrefs) -> AppResult<()> {
    let mut cfg = projects::load(&app)?;
    projects::set_prefs(&mut cfg, &path, prefs);
    projects::save(&app, &cfg)
}

#[tauri::command]
pub async fn open_folder(app: AppHandle, path: String, now: i64) -> AppResult<ProjectView> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(AppError::Other("please choose a folder, not a file".into()));
    }

    let app_cfg = config_store::load(&app)?;
    let projects_cfg = projects::load(&app)?;
    let saved = projects::get_prefs(&projects_cfg, &path);

    // Discovery uses the saved-or-default language pair (fall back to zh→en).
    let src_code = saved
        .as_ref()
        .map(|p| p.source_lang.clone())
        .unwrap_or_else(|| app_cfg.default_source.clone());
    let tgt_code = saved
        .as_ref()
        .map(|p| p.target_lang.clone())
        .unwrap_or_else(|| app_cfg.default_target.clone());
    let pair = LanguagePair::from_codes(&src_code, &tgt_code)
        .or_else(|_| LanguagePair::from_codes("zh", "en"))?;

    // Decode + parse every file off the async runtime thread (we're inside a
    // Tauri async command, which runs on the tokio runtime, so tokio's
    // spawn_blocking is available and its JoinError implements Display).
    let dir_for_blocking = dir.clone();
    let pair_for_blocking = pair.clone();
    let analyzed = tokio::task::spawn_blocking(move || {
        analyze_folder(&dir_for_blocking, &pair_for_blocking)
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))?;

    let detected_world = detect(&analyzed.combined_text, pair.supports_world_detection);
    let detected_source_lang = detect_source_language(&analyzed.combined_text);

    let prefs = projects::resolve_prefs(
        saved,
        detected_source_lang.as_deref(),
        &app_cfg.default_source,
        &app_cfg.default_target,
    );

    let mut projects_cfg = projects_cfg;
    projects::record_recent(&mut projects_cfg, &path, analyzed.files.len() as u32, now);
    projects::save(&app, &projects_cfg)?;

    Ok(ProjectView {
        folder: path,
        total_dialogue_lines: analyzed.files.iter().map(|f| f.dialogue_count).sum(),
        files: analyzed.files,
        detected_source_lang,
        detected_world,
        prefs,
        supports_glossary: pair.supports_glossary,
    })
}

struct Analyzed {
    files: Vec<SourceFile>,
    combined_text: String,
}

fn analyze_folder(dir: &Path, pair: &LanguagePair) -> Analyzed {
    let mut files = Vec::new();
    let mut combined = String::new();
    for src in discover_source_files(dir, pair) {
        let text = decode_file(&src).unwrap_or_default();
        let dialogues = parse_dialogues(&text);
        for d in &dialogues {
            combined.push_str(&strip_for_text(&d.text));
            combined.push('\n');
        }
        files.push(SourceFile {
            name: src
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            path: src.to_string_lossy().into_owned(),
            dialogue_count: dialogues.len() as u32,
            has_translation: has_existing_translation(&src, pair),
        });
    }
    Analyzed {
        files,
        combined_text: combined,
    }
}
```

- [ ] **Step 3: Wire the command module**

In `src-tauri/src/commands/mod.rs`, after the `connections` lines add:

```rust
pub mod project;
pub use project::*;
```

- [ ] **Step 4: Register the commands**

In `src-tauri/src/lib.rs`, add these inside `tauri::generate_handler![ ... ]` (after `commands::list_models,`):

```rust
            commands::list_languages,
            commands::list_recents,
            commands::remove_recent,
            commands::clear_recents,
            commands::save_folder_prefs,
            commands::open_folder,
```

- [ ] **Step 5: Build + run all backend tests**

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: compiles clean.
Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all tests PASS (Step-1 suite + the new Step-2 tests).

- [ ] **Step 6: Generate TS bindings**

Run: `bun gen:bindings`
Then confirm these files now exist:
Run: `ls src/types/generated/ | grep -E 'WorldType|Tone|Language|FolderPrefs|RecentFolder|SourceFile|ProjectView'`
Expected: `WorldType.ts`, `Tone.ts`, `Language.ts`, `FolderPrefs.ts`, `RecentFolder.ts`, `SourceFile.ts`, `ProjectView.ts` all listed.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/models/mod.rs src-tauri/src/commands/project.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src/types/generated/
git commit -m "$(printf 'feat(commands): open_folder (O6/O7/O8) + languages/recents/prefs commands\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 10: Frontend IPC + app-store + relative-time util

**Files:**
- Modify: `src/lib/ipc.ts`
- Modify: `src/stores/app-store.ts`
- Create: `src/lib/relative-time.ts`

- [ ] **Step 1: Add IPC wrappers**

In `src/lib/ipc.ts`, add these type imports after the existing ones:

```ts
import type { Language } from "@/types/generated/Language";
import type { ProjectView } from "@/types/generated/ProjectView";
import type { FolderPrefs } from "@/types/generated/FolderPrefs";
import type { RecentFolder } from "@/types/generated/RecentFolder";
```

Add these properties inside the `ipc` object (after `listModels`):

```ts
  /** O6/O7/O8 — open a folder: discover files, counts, detect world/language, load prefs. */
  openFolder: (path: string) =>
    invoke<ProjectView>("open_folder", { path, now: Math.floor(Date.now() / 1000) }),
  /** O8 — the supported-language table for the selects. */
  listLanguages: () => invoke<Language[]>("list_languages"),
  /** Persist per-folder preferences (languages, world override, tone, selection). */
  saveFolderPrefs: (path: string, prefs: FolderPrefs) =>
    invoke<void>("save_folder_prefs", { path, prefs }),
  /** Recent folders (MRU; missing folders pruned server-side). */
  listRecents: () => invoke<RecentFolder[]>("list_recents"),
  removeRecent: (path: string) => invoke<void>("remove_recent", { path }),
  clearRecents: () => invoke<void>("clear_recents"),
```

- [ ] **Step 2: Extend the app store**

Replace the entire contents of `src/stores/app-store.ts` with:

```ts
import { create } from "zustand";
import type { WorldType } from "@/types/generated/WorldType";
import type { Tone } from "@/types/generated/Tone";

interface ProjectState {
  workdir: string;
  sourceLang: string;
  targetLang: string;
  worldType: WorldType;
  tone: Tone;
  fileCount: number;
  dialogueLineCount: number;
  hasUntranslated: boolean;
  hasTranslated: boolean;
}

interface AppState {
  workdir: string | null;
  sourceLang: string;
  targetLang: string;
  worldType: WorldType | null;
  tone: Tone;
  fileCount: number;
  dialogueLineCount: number;
  hasUntranslated: boolean;
  hasTranslated: boolean;
  activeConnection: string | null;
  hasUsableConnection: boolean;
  setWorkdir: (dir: string | null) => void;
  setLanguages: (source: string, target: string) => void;
  setProject: (p: ProjectState) => void;
  clearProject: () => void;
  setActiveConnection: (name: string | null) => void;
  setHasUsableConnection: (v: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  workdir: null,
  sourceLang: "zh",
  targetLang: "en",
  worldType: null,
  tone: "standard",
  fileCount: 0,
  dialogueLineCount: 0,
  hasUntranslated: false,
  hasTranslated: false,
  activeConnection: null,
  hasUsableConnection: false,
  setWorkdir: (workdir) => set({ workdir }),
  setLanguages: (sourceLang, targetLang) => set({ sourceLang, targetLang }),
  setProject: (p) => set({ ...p }),
  clearProject: () =>
    set({
      workdir: null,
      worldType: null,
      fileCount: 0,
      dialogueLineCount: 0,
      hasUntranslated: false,
      hasTranslated: false,
    }),
  setActiveConnection: (activeConnection) => set({ activeConnection }),
  setHasUsableConnection: (hasUsableConnection) => set({ hasUsableConnection }),
}));
```

- [ ] **Step 3: Add the relative-time util**

Create `src/lib/relative-time.ts`:

```ts
/**
 * Human "time ago" for a unix-seconds timestamp. Port of the Python
 * `theme.py:format_relative_time` (just now / Nm ago / Nh ago / yesterday /
 * Nd ago / N wks / N mo / N yrs).
 */
export function formatRelativeTime(epochSeconds: number, nowMs = Date.now()): string {
  const diff = Math.max(0, Math.floor(nowMs / 1000) - epochSeconds);
  if (diff < 60) return "just now";
  const mins = Math.floor(diff / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days === 1) return "yesterday";
  if (days < 7) return `${days}d ago`;
  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks} wk${weeks > 1 ? "s" : ""}`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months} mo`;
  const years = Math.floor(days / 365);
  return `${years} yr${years > 1 ? "s" : ""}`;
}
```

- [ ] **Step 4: Verify the build**

Run: `bun run build`
Expected: PASS (route-gen → tsc → vite). No type errors. (The new store fields/util are not yet consumed; that's fine.)

- [ ] **Step 5: Commit**

```bash
git add src/lib/ipc.ts src/stores/app-store.ts src/lib/relative-time.ts
git commit -m "$(printf 'feat(ui): folder-pickup IPC wrappers, project store fields, relative-time\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 11: Welcome screen (recents, open-folder, drag-drop)

**Files:**
- Create: `src/components/recent-item.tsx`
- Create: `src/features/welcome/use-recents.ts`
- Create: `src/features/project/use-project.ts`
- Create: `src/features/welcome/welcome-page.tsx`
- Modify: `src/routes/index.tsx`

> **Capabilities note:** `src-tauri/capabilities/default.json` already grants `dialog:default` (the folder picker) and `core:default` (the webview drag-drop event is enabled by default). No capability change is needed; just confirm the picker + drop work in the manual smoke at the end.

- [ ] **Step 1: RecentItem component**

Create `src/components/recent-item.tsx`:

```tsx
import { Folder } from "@phosphor-icons/react";
import type { RecentFolder } from "@/types/generated/RecentFolder";
import { formatRelativeTime } from "@/lib/relative-time";

export function RecentItem({
  recent,
  onOpen,
  onRemove,
}: {
  recent: RecentFolder;
  onOpen: () => void;
  onRemove: () => void;
}) {
  const name = recent.path.split("/").pop() || recent.path;
  return (
    <div className="group flex items-center gap-3 rounded-md border border-border bg-[color:var(--card)] px-3 py-2 hover:bg-[color:var(--color-bg-hover)]">
      <button
        type="button"
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-3 text-left"
      >
        <Folder weight="fill" className="size-5 shrink-0 text-primary" />
        <span className="min-w-0">
          <span className="block truncate text-[13px] font-medium text-foreground">{name}</span>
          <span className="block truncate font-mono text-[11px] text-muted-foreground">
            {recent.path}
          </span>
        </span>
      </button>
      <span className="shrink-0 text-[11px] tabular-nums text-muted-foreground">
        {recent.file_count} files · {formatRelativeTime(recent.last_opened)}
      </span>
      <button
        type="button"
        onClick={onRemove}
        className="shrink-0 text-[11px] text-muted-foreground opacity-0 transition group-hover:opacity-100 hover:text-[color:var(--color-danger)]"
      >
        Remove
      </button>
    </div>
  );
}
```

- [ ] **Step 2: Recents hook**

Create `src/features/welcome/use-recents.ts`:

```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ipc } from "@/lib/ipc";

const KEY = ["recents"] as const;

export function useRecents() {
  return useQuery({ queryKey: KEY, queryFn: ipc.listRecents });
}

export function useRecentMutations() {
  const qc = useQueryClient();
  const invalidate = () => qc.invalidateQueries({ queryKey: KEY });
  return {
    remove: useMutation({ mutationFn: ipc.removeRecent, onSuccess: invalidate }),
    clear: useMutation({ mutationFn: ipc.clearRecents, onSuccess: invalidate }),
  };
}
```

- [ ] **Step 3: Project hooks (shared open-folder mutation + project query)**

Create `src/features/project/use-project.ts`:

```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";
import type { ProjectView } from "@/types/generated/ProjectView";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";

export function projectKey(path: string) {
  return ["project", path] as const;
}

/** Apply a ProjectView to global shell state (status bar + rail gating). */
export function syncProjectStore(view: ProjectView) {
  const untranslated = view.files.filter((f) => !f.has_translation).length;
  useAppStore.getState().setProject({
    workdir: view.folder,
    sourceLang: view.prefs.source_lang,
    targetLang: view.prefs.target_lang,
    worldType: view.prefs.world_override ?? view.detected_world,
    tone: view.prefs.tone,
    fileCount: view.files.length,
    dialogueLineCount: view.total_dialogue_lines,
    hasUntranslated: untranslated > 0,
    hasTranslated: view.files.length - untranslated > 0,
  });
}

/**
 * Open a folder: discover/analyze, seed the project cache, sync the shell, and
 * navigate to Project. A folder with zero `.ass` files stays on Welcome (the
 * caller renders an inline message from `mutation.data`).
 */
export function useOpenFolder() {
  const qc = useQueryClient();
  const navigate = useNavigate();
  return useMutation({
    mutationFn: ipc.openFolder,
    onSuccess: (view) => {
      qc.invalidateQueries({ queryKey: ["recents"] });
      if (view.files.length === 0) return; // empty: stay, show inline message
      qc.setQueryData(projectKey(view.folder), view);
      syncProjectStore(view);
      void navigate({ to: "/project" });
    },
    onError: (e) => toast.error(String(e)),
  });
}

/** The current folder's ProjectView (seeded by useOpenFolder; refetches on reload). */
export function useProject(path: string) {
  return useQuery({
    queryKey: projectKey(path),
    queryFn: () => ipc.openFolder(path),
    enabled: !!path,
    staleTime: Infinity,
  });
}
```

- [ ] **Step 4: Welcome page**

Create `src/features/welcome/welcome-page.tsx`:

```tsx
import { useEffect } from "react";
import { useNavigate } from "@tanstack/react-router";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Translate } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import { RecentItem } from "@/components/recent-item";
import { useAppStore } from "@/stores/app-store";
import { useRecents, useRecentMutations } from "./use-recents";
import { useOpenFolder } from "@/features/project/use-project";

export function WelcomePage() {
  const navigate = useNavigate();
  const hasConnection = useAppStore((s) => s.hasUsableConnection);
  const { data: recents } = useRecents();
  const recentM = useRecentMutations();
  const open = useOpenFolder();

  const pick = async () => {
    const path = await openDialog({ directory: true, multiple: false });
    if (typeof path === "string") open.mutate(path);
  };

  // Folder drag-and-drop onto the window → open the first dropped path.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop" && event.payload.paths.length > 0) {
        open.mutate(event.payload.paths[0]);
      }
    });
    return () => {
      void unlisten.then((u) => u());
    };
  }, [open]);

  const emptyResult = open.data?.files.length === 0;

  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 p-10">
      <div className="flex flex-col items-center gap-1 text-center">
        <Translate weight="fill" className="size-10 text-primary" />
        <h1 className="text-[22px] font-semibold text-foreground">Subs Translator</h1>
        <p className="text-sm text-muted-foreground">
          LLM-powered subtitle translation for donghua &amp; anime.
        </p>
      </div>

      {!hasConnection ? (
        <div className="w-full max-w-xl space-y-2">
          <Step n={1} title="Connect an AI provider" hint="OpenAI, Anthropic, Gemini, Z.AI, or a local model — needs an API key.">
            <Button variant="secondary" onClick={() => navigate({ to: "/connections" })}>
              Connect
            </Button>
          </Step>
          <Step n={2} title="Open a folder of subtitles" hint="Point at a folder of .ass files to begin.">
            <Button onClick={pick}>Open folder</Button>
          </Step>
        </div>
      ) : (
        <div className="w-full max-w-xl space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-[13px] font-semibold text-muted-foreground">Recent folders</h2>
            <Button size="sm" onClick={pick}>
              Open folder
            </Button>
          </div>
          {recents && recents.length > 0 ? (
            <div className="space-y-1.5">
              {recents.map((r) => (
                <RecentItem
                  key={r.path}
                  recent={r}
                  onOpen={() => open.mutate(r.path)}
                  onRemove={() => recentM.remove.mutate(r.path)}
                />
              ))}
              <button
                type="button"
                onClick={() => {
                  if (confirm("Clear all recent folders?")) recentM.clear.mutate();
                }}
                className="text-[11px] text-muted-foreground hover:text-foreground"
              >
                Clear all
              </button>
            </div>
          ) : (
            <p className="text-center text-[12px] text-muted-foreground">No recent folders yet.</p>
          )}
          <p className="text-center text-[11px] text-muted-foreground">
            ↓ or drag a folder onto the window
          </p>
          {emptyResult ? (
            <p className="text-center text-[12px] text-[color:var(--color-alert)]">
              No subtitle files found here.
            </p>
          ) : null}
        </div>
      )}
    </div>
  );
}

function Step({
  n,
  title,
  hint,
  children,
}: {
  n: number;
  title: string;
  hint: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-border bg-[color:var(--card)] px-4 py-3">
      <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-[color:var(--popover)] text-[12px] text-primary">
        {n}
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-[13px] font-medium text-foreground">{title}</div>
        <div className="text-[11px] text-muted-foreground">{hint}</div>
      </div>
      {children}
    </div>
  );
}
```

- [ ] **Step 5: Make `/` render Welcome (drop the redirect)**

Replace the entire contents of `src/routes/index.tsx` with:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { WelcomePage } from "@/features/welcome/welcome-page";

export const Route = createFileRoute("/")({
  beforeLoad: async () => {
    // Seed the rail badge before first render (Welcome shows the first-run steps
    // when there's no usable connection; no auto-redirect to Connections).
    const status = await ipc.firstRunStatus();
    useAppStore.getState().setHasUsableConnection(status.has_usable_connection);
  },
  component: WelcomePage,
});
```

- [ ] **Step 6: Verify build**

Run: `bun run build`
Expected: PASS. (If the `<a>`-in-Button approach errors, apply the `Link` swap from the note in Step 4.)

- [ ] **Step 7: Commit**

```bash
git add src/components/recent-item.tsx src/features/welcome/ src/features/project/use-project.ts src/routes/index.tsx
git commit -m "$(printf 'feat(ui): Welcome screen — recents, open-folder, drag-and-drop\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 12: Project view (setup fields, file list, forks)

**Files:**
- Create: `src/features/project/file-list.tsx`
- Create: `src/features/project/project-page.tsx`
- Modify: `src/routes/project.tsx`

- [ ] **Step 1: File list with selection checkboxes**

Create `src/features/project/file-list.tsx`:

```tsx
import { CheckCircle } from "@phosphor-icons/react";
import type { SourceFile } from "@/types/generated/SourceFile";
import { Checkbox } from "@/components/ui/checkbox";

/**
 * Source-file list with per-file selection. An empty `selected` array means
 * "all files selected" (the backend's convention); the header checkbox resets
 * to that state.
 */
export function FileList({
  files,
  selected,
  onChange,
}: {
  files: SourceFile[];
  selected: string[];
  onChange: (sel: string[]) => void;
}) {
  const allSelected = selected.length === 0;
  const isChecked = (name: string) => allSelected || selected.includes(name);

  const toggle = (name: string) => {
    const base = allSelected ? files.map((f) => f.name) : selected;
    const next = base.includes(name) ? base.filter((n) => n !== name) : [...base, name];
    // Re-selecting everything normalizes back to "all" (empty array).
    onChange(next.length === files.length ? [] : next);
  };

  return (
    <div className="mt-4 rounded-md border border-border">
      <div className="flex items-center gap-2 border-b border-border px-3 py-2 text-[11px] text-muted-foreground">
        <Checkbox checked={allSelected} onCheckedChange={() => onChange([])} />
        <span>
          {files.length} files{" "}
          {allSelected ? "· all selected" : `· ${selected.length} selected`}
        </span>
      </div>
      <ul className="max-h-64 overflow-auto">
        {files.map((f) => (
          <li key={f.path} className="flex items-center gap-2 px-3 py-1.5 text-[12px]">
            <Checkbox checked={isChecked(f.name)} onCheckedChange={() => toggle(f.name)} />
            <span className="flex-1 truncate">{f.name}</span>
            <span className="shrink-0 text-[10px] tabular-nums text-muted-foreground">
              {f.dialogue_count} lines
            </span>
            {f.has_translation ? (
              <span className="inline-flex shrink-0 items-center gap-0.5 text-[10px] text-[color:var(--color-success)]">
                <CheckCircle weight="fill" className="size-3" />
                translated
              </span>
            ) : null}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

- [ ] **Step 2: Project page**

Create `src/features/project/project-page.tsx`:

```tsx
import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { BookOpen, Play } from "@phosphor-icons/react";
import type { Language } from "@/types/generated/Language";
import type { FolderPrefs } from "@/types/generated/FolderPrefs";
import type { Tone } from "@/types/generated/Tone";
import type { WorldType } from "@/types/generated/WorldType";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useProject, syncProjectStore } from "./use-project";
import { FileList } from "./file-list";
import { PageHeader } from "@/components/page-header";
import { SetupField } from "@/components/setup-field";
import { HelpText } from "@/components/help-text";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/empty-state";

const TONES: Tone[] = ["standard", "xianxia", "wuxia", "comedic", "funny"];
const WORLDS: WorldType[] = ["xianxia", "wuxia", "historical", "modern"];
const SELECT_CLS =
  "h-9 w-full rounded-md border border-input bg-[color:var(--card)] px-2 text-sm";

export function ProjectPage() {
  const workdir = useAppStore((s) => s.workdir);
  const navigate = useNavigate();
  const { data: view } = useProject(workdir ?? "");
  const { data: languages } = useQuery({
    queryKey: ["languages"],
    queryFn: ipc.listLanguages,
    staleTime: Infinity,
  });
  const [prefs, setPrefs] = useState<FolderPrefs | null>(null);

  useEffect(() => {
    if (view) setPrefs(view.prefs);
  }, [view]);

  if (!workdir) return <EmptyState title="Project" description="Open a folder first." />;
  if (!view || !prefs || !languages) return null;

  const persist = (next: FolderPrefs) => {
    setPrefs(next);
    void ipc.saveFolderPrefs(view.folder, next);
    // Keep the shell (status bar + rail gating) consistent with edits.
    syncProjectStore({ ...view, prefs: next });
  };

  const sameLang = prefs.source_lang === prefs.target_lang;
  const sourceLang = languages.find((l) => l.code === prefs.source_lang);
  const showWorld = !!sourceLang?.supports_world_detection;
  const effectiveWorld: WorldType = prefs.world_override ?? view.detected_world;
  const folderName = view.folder.split("/").pop() || view.folder;

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title={folderName}
        description="Set up this folder, then build a glossary or jump to translating."
        actions={
          <button
            type="button"
            onClick={() => navigate({ to: "/" })}
            className="text-[11px] text-primary hover:underline"
          >
            change
          </button>
        }
      />
      <div className="flex-1 overflow-auto p-5">
        <p className="mb-3 text-[12.5px] text-muted-foreground tabular-nums">
          {view.files.length} subtitle files · {view.total_dialogue_lines} lines
        </p>

        <div className="grid grid-cols-2 gap-x-4">
          <SetupField
            label="Source language"
            help={<HelpText>Detected from the files; change if it&apos;s wrong.</HelpText>}
          >
            <LangSelect
              languages={languages}
              value={prefs.source_lang}
              onChange={(v) => persist({ ...prefs, source_lang: v })}
            />
          </SetupField>
          <SetupField
            label="Target language"
            help={
              sameLang ? (
                <p className="mt-1 text-[11px] text-[color:var(--color-danger)]">
                  Source and target must differ.
                </p>
              ) : undefined
            }
          >
            <LangSelect
              languages={languages}
              value={prefs.target_lang}
              onChange={(v) => persist({ ...prefs, target_lang: v })}
            />
          </SetupField>

          {showWorld ? (
            <SetupField
              label="World type"
              help={<HelpText>Tunes how names &amp; cultivation terms are extracted.</HelpText>}
            >
              <select
                className={SELECT_CLS}
                value={effectiveWorld}
                onChange={(e) =>
                  persist({ ...prefs, world_override: e.target.value as WorldType })
                }
              >
                {WORLDS.map((w) => (
                  <option key={w} value={w}>
                    {w}
                    {!prefs.world_override && w === view.detected_world ? " (auto-detected)" : ""}
                  </option>
                ))}
              </select>
            </SetupField>
          ) : null}

          <SetupField label="Tone" help={<HelpText>The register of the dialogue.</HelpText>}>
            <select
              className={SELECT_CLS}
              value={prefs.tone}
              onChange={(e) => persist({ ...prefs, tone: e.target.value as Tone })}
            >
              {TONES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </SetupField>
        </div>

        <FileList
          files={view.files}
          selected={prefs.selected_files}
          onChange={(sel) => persist({ ...prefs, selected_files: sel })}
        />
      </div>

      <div className="flex items-center gap-3 border-t border-border bg-[color:var(--popover)] px-5 py-3">
        <span className="text-[11px] text-muted-foreground">Next:</span>
        {view.supports_glossary ? (
          <Button variant="secondary" onClick={() => navigate({ to: "/glossary" })}>
            <BookOpen className="size-4" /> Build a glossary
          </Button>
        ) : null}
        <Button disabled={sameLang} onClick={() => navigate({ to: "/translate" })}>
          <Play className="size-4" /> Translate now
        </Button>
      </div>
    </div>
  );
}

function LangSelect({
  languages,
  value,
  onChange,
}: {
  languages: Language[];
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <select className={SELECT_CLS} value={value} onChange={(e) => onChange(e.target.value)}>
      {languages.map((l) => (
        <option key={l.code} value={l.code}>
          {l.name}
        </option>
      ))}
    </select>
  );
}
```

- [ ] **Step 3: Mount the route**

Replace the entire contents of `src/routes/project.tsx` with:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { ProjectPage } from "@/features/project/project-page";

export const Route = createFileRoute("/project")({
  component: ProjectPage,
});
```

- [ ] **Step 4: Verify build**

Run: `bun run build`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/features/project/file-list.tsx src/features/project/project-page.tsx src/routes/project.tsx
git commit -m "$(printf 'feat(ui): Project view — language/world/tone setup, file list, forks\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Task 13: Status bar fill + nav-rail gating

**Files:**
- Modify: `src/components/status-bar.tsx`
- Modify: `src/components/nav-rail.tsx`

- [ ] **Step 1: Status bar — real counts + world chip**

Replace the entire contents of `src/components/status-bar.tsx` with:

```tsx
import { useQuery } from "@tanstack/react-query";
import { Folder } from "@phosphor-icons/react";
import { StateChip } from "@/components/state-chip";
import { StatusChip } from "@/components/status-chip";
import { Separator } from "@/components/ui/separator";
import { ipc } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";

export function StatusBar() {
  const workdir = useAppStore((s) => s.workdir);
  const sourceLang = useAppStore((s) => s.sourceLang);
  const targetLang = useAppStore((s) => s.targetLang);
  const worldType = useAppStore((s) => s.worldType);
  const fileCount = useAppStore((s) => s.fileCount);
  const lineCount = useAppStore((s) => s.dialogueLineCount);
  const connection = useAppStore((s) => s.activeConnection);
  const { data: appInfo } = useQuery({ queryKey: ["app-info"], queryFn: ipc.appInfo });

  return (
    <footer className="col-start-2 flex h-8 items-center gap-3 border-t border-border bg-[color:var(--color-bg-deepest)] px-3 text-[11px] text-muted-foreground">
      <span className="flex min-w-0 items-center gap-1.5">
        <Folder className="size-3.5 shrink-0" />
        <span className="truncate">{workdir ?? "No folder selected"}</span>
      </span>
      <Separator orientation="vertical" className="h-4" />
      <span className="shrink-0 tabular-nums">
        {workdir ? `${fileCount} files · ${lineCount} lines` : "— files · — lines"}
      </span>
      <span className="flex-1" />
      {worldType ? <StatusChip variant="muted">{worldType}</StatusChip> : null}
      <span className="shrink-0">
        {sourceLang}→{targetLang}
      </span>
      <StatusChip variant="accent">{connection ?? "No connection"}</StatusChip>
      <StateChip state="idle" />
      <span className="shrink-0 opacity-60">core {appInfo?.version ?? "…"}</span>
    </footer>
  );
}
```

- [ ] **Step 2: Nav-rail — per-destination gating**

Replace the entire contents of `src/components/nav-rail.tsx` with:

```tsx
import {
  BookOpen,
  CheckCircle,
  Folder,
  Gear,
  Lightning,
  Play,
  Question,
  type Icon,
} from "@phosphor-icons/react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useAppStore } from "@/stores/app-store";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

interface RailItem {
  to: string;
  label: string;
  icon: Icon;
  group: "workflow" | "setup";
  needsFolder?: boolean;
}

const ITEMS: RailItem[] = [
  { to: "/project", label: "Project", icon: Folder, group: "workflow", needsFolder: true },
  { to: "/glossary", label: "Glossary", icon: BookOpen, group: "workflow", needsFolder: true },
  { to: "/translate", label: "Translate", icon: Play, group: "workflow", needsFolder: true },
  { to: "/verify", label: "Verify", icon: CheckCircle, group: "workflow", needsFolder: true },
  { to: "/connections", label: "Connections", icon: Lightning, group: "setup" },
  { to: "/settings", label: "Settings", icon: Gear, group: "setup" },
  { to: "/help", label: "Help", icon: Question, group: "setup" },
];

export function NavRail() {
  const workdir = useAppStore((s) => s.workdir);
  const hasUsableConnection = useAppStore((s) => s.hasUsableConnection);
  const hasUntranslated = useAppStore((s) => s.hasUntranslated);
  const hasTranslated = useAppStore((s) => s.hasTranslated);
  const pathname = useRouterState({ select: (s) => s.location.pathname });

  // Returns a gating hint when the destination is disabled, else null.
  const gateHint = (item: RailItem): string | null => {
    if (!item.needsFolder) return null;
    if (!workdir) return "Open a folder first";
    if (item.to === "/translate") {
      if (!hasUsableConnection) return "Connect an AI provider";
      if (!hasUntranslated) return "No untranslated files in this folder";
    }
    if (item.to === "/verify" && !hasTranslated) return "Translate something first";
    return null;
  };

  const workflow = ITEMS.filter((i) => i.group === "workflow");
  const setup = ITEMS.filter((i) => i.group === "setup");

  const render = (item: RailItem) => {
    const hint = gateHint(item);
    const disabled = hint !== null;
    const active = pathname.startsWith(item.to);
    const Icon = item.icon;
    const body = (
      <div
        className={cn(
          "flex w-16 flex-col items-center gap-1 rounded-md py-2 text-[10px]",
          active && "bg-[color:var(--popover)] text-primary",
          disabled
            ? "cursor-not-allowed text-muted-foreground/50"
            : "hover:bg-[color:var(--color-bg-hover)]",
        )}
      >
        <Icon weight={active ? "fill" : "regular"} className="size-5" />
        {item.label}
        {item.to === "/connections" ? (
          <span
            className={
              hasUsableConnection
                ? "text-[color:var(--color-success)]"
                : "text-[color:var(--color-alert)]"
            }
          >
            {hasUsableConnection ? "✓" : "⚠"}
          </span>
        ) : null}
      </div>
    );
    if (disabled) {
      return (
        <Tooltip key={item.to}>
          <TooltipTrigger asChild>
            <div>{body}</div>
          </TooltipTrigger>
          <TooltipContent side="right">{hint}</TooltipContent>
        </Tooltip>
      );
    }
    return (
      <Link key={item.to} to={item.to as never}>
        {body}
      </Link>
    );
  };

  return (
    <nav className="flex w-20 flex-col items-center gap-1 border-r border-border bg-[color:var(--sidebar)] py-3">
      {workflow.map(render)}
      <div className="my-1 h-px w-10 bg-border" />
      {render(setup[0])}
      <div className="flex-1" />
      {setup.slice(1).map(render)}
    </nav>
  );
}
```

- [ ] **Step 3: Verify build**

Run: `bun run build`
Expected: PASS.

- [ ] **Step 4: Full manual smoke (out-of-band, not CI)**

Run `bun tauri dev`. With a usable connection seeded:
- App opens on **Welcome** (not Connections). Recents list appears (empty initially).
- **Open folder** picks a directory; a folder with `.ass` files navigates to **Project** showing real file + line counts, a detected source language and world type (for Chinese content), and per-file translated badges.
- Editing language/world/tone/selection persists (re-open the same folder → choices restored).
- **Drag-and-drop** a folder onto the window opens it; dropping a *file* shows the not-a-directory error toast.
- A folder with **no** `.ass` files stays on Welcome with "No subtitle files found here."
- Status bar shows folder + counts + world + language pair. Rail: Translate enabled only with a connection + untranslated files; Verify enabled only when a translation exists.

- [ ] **Step 5: Commit**

```bash
git add src/components/status-bar.tsx src/components/nav-rail.tsx
git commit -m "$(printf 'feat(ui): status-bar counts/world chip + per-destination rail gating\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>')"
```

---

## Final verification (after all tasks)

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` — all green.
- [ ] `bun run build` — green.
- [ ] `git log --oneline` shows 13 task commits on `step2-welcome-and-project`.
- [ ] Manual smoke (Task 13 Step 4) passes.

Then hand off to **superpowers:finishing-a-development-branch**.

---

## Notes / known follow-ups (do NOT implement here)
- **Writer + positional tag reapply** → Step 3 (translation). Spec §2 records the design.
- `selected_files` empty = "all" — the Translate step must honor this convention.
- `open_folder` re-detects source/world each open until the user saves a pref; that's intentional (we only persist what the user touches).
- Glossary/Translate/Verify destinations are still placeholders; this step only wires their rail gating.
