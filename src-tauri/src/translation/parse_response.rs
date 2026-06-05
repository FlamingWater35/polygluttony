//! Parse LLM responses: strip fences, repair common JSON damage, extract the
//! line array. Port of `utils/json_repairer.py` minus the external subprocess.

use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use crate::validation::LinePair;

#[derive(Debug, thiserror::Error)]
#[error("could not extract JSON from response: {0}")]
pub struct ResponseParseError(pub String);

static FENCE_OPEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^```(?:json)?\s*\n?").unwrap());
static FENCE_CLOSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n?```\s*$").unwrap());
static TRAILING_COMMA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r",(\s*[}\]])").unwrap());
static ARRAY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\[.*\]").unwrap());
static OBJECT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\{.*\}").unwrap());

fn repair(raw: &str) -> Result<Value, ResponseParseError> {
    let s = FENCE_OPEN.replace(raw.trim(), "");
    let s = FENCE_CLOSE.replace(&s, "");
    let s = s.trim();

    if let Ok(v) = serde_json::from_str::<Value>(s) {
        return Ok(v);
    }
    let no_trailing = TRAILING_COMMA.replace_all(s, "$1");
    if let Ok(v) = serde_json::from_str::<Value>(&no_trailing) {
        return Ok(v);
    }
    for re in [&*ARRAY, &*OBJECT] {
        if let Some(m) = re.find(&no_trailing) {
            if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
                return Ok(v);
            }
        }
    }
    Err(ResponseParseError(s.chars().take(120).collect()))
}

/// The translation-batch shape: array of `{id, src?, tgt}`.
pub fn extract_pairs(raw: &str) -> Result<Vec<LinePair>, ResponseParseError> {
    let v = repair(raw)?;
    let arr = v
        .as_array()
        .ok_or_else(|| ResponseParseError("repaired JSON is not an array".into()))?;
    Ok(arr
        .iter()
        .filter_map(|item| {
            let id = match item.get("id") {
                Some(Value::Number(n)) => n.as_u64()? as u32,
                Some(Value::String(s)) => s.trim().parse().ok()?,
                _ => return None,
            };
            Some(LinePair {
                id,
                src: item.get("src").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
                tgt: item.get("tgt").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
            })
        })
        .collect())
}

/// The verify shape: an object (`{"issues": [...]}`).
pub fn extract_object(raw: &str) -> Result<Value, ResponseParseError> {
    let v = repair(raw)?;
    if v.is_object() {
        Ok(v)
    } else {
        Err(ResponseParseError("repaired JSON is not an object".into()))
    }
}

/// A bare JSON array of arbitrary objects (the verify fallback shape).
///
/// Used when the model returns `[{"id":5,"reason":"x"}]` directly instead of
/// the expected `{"issues":[...]}` wrapper — e.g. when the response is
/// truncated mid-object. Port of `utils/json_repairer.py` `extract_array`.
pub fn extract_array(raw: &str) -> Result<Vec<Value>, ResponseParseError> {
    let v = repair(raw)?;
    v.as_array()
        .cloned()
        .ok_or_else(|| ResponseParseError("repaired JSON is not an array".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_json_array() {
        let pairs =
            extract_pairs(r#"[{"id": 1, "src": "你好", "tgt": "<0001:D> Hello"}]"#).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].id, 1);
        assert_eq!(pairs[0].tgt, "<0001:D> Hello");
    }

    #[test]
    fn strips_markdown_fences() {
        let raw = "```json\n[{\"id\": 1, \"tgt\": \"Hi\"}]\n```";
        assert_eq!(extract_pairs(raw).unwrap().len(), 1);
    }

    #[test]
    fn removes_trailing_commas() {
        let raw = r#"[{"id": 1, "tgt": "Hi",},]"#;
        assert_eq!(extract_pairs(raw).unwrap().len(), 1);
    }

    #[test]
    fn extracts_array_embedded_in_prose() {
        let raw = r#"Here you go: [{"id": 1, "tgt": "Hi"}] Hope that helps!"#;
        assert_eq!(extract_pairs(raw).unwrap().len(), 1);
    }

    #[test]
    fn rejects_hopeless_input() {
        assert!(extract_pairs("I cannot translate this").is_err());
    }

    #[test]
    fn skips_entries_without_id_and_accepts_string_ids() {
        let raw = r#"[{"id": "2", "tgt": "Hi"}, {"tgt": "orphan"}]"#;
        let pairs = extract_pairs(raw).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].id, 2);
    }

    #[test]
    fn extract_object_for_verify_responses() {
        let raw = "```json\n{\"issues\": [{\"id\": 5, \"reason\": \"x\"}]}\n```";
        let v = extract_object(raw).unwrap();
        assert_eq!(v["issues"][0]["id"], 5);
    }

    #[test]
    fn extract_array_bare_returns_items() {
        let arr = extract_array(r#"[{"id":5,"reason":"x"}]"#).unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], 5);
    }

    #[test]
    fn extract_array_rejects_object() {
        assert!(extract_array(r#"{"a":1}"#).is_err());
    }
}
