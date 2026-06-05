//! Parse the `[Events]` section of an `.ass` file into dialogue lines. The ASS
//! `Dialogue:` format is `Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text`
//! — exactly nine commas before the free-form text, so we split on the first nine.

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

/// Centiseconds → `H:MM:SS.cc` (ASS timestamp shape).
pub fn format_timestamp_cs(cs: i64) -> String {
    let cs = cs.max(0);
    format!(
        "{}:{:02}:{:02}.{:02}",
        cs / 360_000,
        (cs / 6_000) % 60,
        (cs / 100) % 60,
        cs % 100
    )
}

impl DialogueLine {
    /// Render back to a `Dialogue:` line (10 fields; text keeps its commas).
    pub fn render(&self) -> String {
        format!(
            "Dialogue: {},{},{},{},{},{},{},{},{},{}",
            self.layer,
            format_timestamp_cs(self.start_cs),
            format_timestamp_cs(self.end_cs),
            self.style,
            self.name,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.effect,
            self.text
        )
    }
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
// Step 3+ (Translate view): used to display line counts before pipeline starts.
#[allow(dead_code)]
pub fn dialogue_count(text: &str) -> usize {
    parse_dialogues(text).len()
}

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
        assert_eq!(parse_timestamp_cs("1:02:03.04"), (3600 + 2 * 60 + 3) * 100 + 4);
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

    #[test]
    fn formats_timestamps_back() {
        assert_eq!(format_timestamp_cs(0), "0:00:00.00");
        assert_eq!(format_timestamp_cs(parse_timestamp_cs("1:02:03.45")), "1:02:03.45");
    }

    #[test]
    fn renders_dialogue_line() {
        let d = DialogueLine {
            layer: 0,
            start_cs: 150,
            end_cs: 320,
            style: "Default".into(),
            name: "".into(),
            margin_l: 0,
            margin_r: 0,
            margin_v: 0,
            effect: "".into(),
            text: r"{\an8}Hello, world".into(),
        };
        assert_eq!(
            d.render(),
            r"Dialogue: 0,0:00:01.50,0:00:03.20,Default,,0,0,0,,{\an8}Hello, world"
        );
    }
}
