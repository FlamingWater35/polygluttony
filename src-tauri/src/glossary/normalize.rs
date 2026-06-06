//! Per-category glossary normalization (O12 + build step 6). Port of
//! `glossary_builder.py:441-533`.

use std::collections::BTreeMap;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::events::{GlossaryEvent, LogLevel};
use crate::glossary::diff::GlossaryDiff;
use crate::glossary::model::{Glossary, GlossaryDoc, CATEGORIES};
use crate::glossary::prompts;
use crate::llm::service::LlmService;
use crate::llm::LlmRequest;
use crate::translation::parse_response;

/// O12 result: the normalized glossary + diff, NOT yet saved — the UI shows a
/// review and saves on accept.
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct NormalizeReview {
    pub normalized: GlossaryDoc,
    pub diff: GlossaryDiff,
}

/// Parse one category response into a validated term map. `None` = keep the
/// original category. Values trimmed + re-validated (deviation 8 — Python
/// trusted the response wholesale, `glossary_builder.py:470-473`).
fn parse_category_response(text: &str) -> Option<BTreeMap<String, String>> {
    let v = parse_response::extract_object(text).ok()?;
    let obj = v.as_object()?;
    let mut out = BTreeMap::new();
    for (k, val) in obj {
        if let Some(s) = val.as_str() {
            let t = s.trim();
            if !t.is_empty() && t.chars().count() <= 200 {
                out.insert(k.clone(), t.to_string());
            }
        }
    }
    Some(out)
}

/// Normalize every non-empty category concurrently (the service bounds
/// concurrency). World type comes from the glossary itself
/// (`glossary_builder.py:504`: `glossary.world_type or "modern"`). A failed
/// category keeps its original terms (ONE warning log per failure).
// consumed by commands/glossary (later step-4 task)
#[allow(dead_code)]
pub async fn normalize_pass(
    svc: &LlmService,
    glossary: &Glossary,
    tx: &mpsc::Sender<GlossaryEvent>,
) -> Glossary {
    let world =
        if glossary.world_type.is_empty() { "modern" } else { glossary.world_type.as_str() };

    let jobs: Vec<&str> = CATEGORIES
        .iter()
        .copied()
        .filter(|c| !glossary.category(c).is_empty())
        .collect();
    let futures = jobs.iter().map(|c| {
        let req = LlmRequest {
            system: prompts::normalize_prompt(c, world),
            user: prompts::normalize_user_prompt(glossary.category(c)),
        };
        svc.request(req)
    });
    let results = join_all(futures).await;

    let mut out = glossary.clone();
    for (c, result) in jobs.iter().zip(results) {
        let replacement = match result {
            Ok(resp) => parse_category_response(&resp.text),
            Err(e) => {
                let _ = tx
                    .send(GlossaryEvent::Log {
                        level: LogLevel::Warning,
                        message: format!("normalize {c}: request failed, keeping original ({e})"),
                    })
                    .await;
                continue; // original kept; one log only
            }
        };
        match replacement {
            Some(map) => *out.category_mut(c) = map,
            None => {
                let _ = tx
                    .send(GlossaryEvent::Log {
                        level: LogLevel::Warning,
                        message: format!("normalize {c}: unusable response, keeping original"),
                    })
                    .await;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::model::Glossary;
    use crate::llm::error::LlmError;
    use crate::llm::service::LlmService;
    use crate::llm::test_support::ScriptedDriver;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    fn svc(driver: Arc<ScriptedDriver>, cap: u32) -> LlmService {
        let (tx, _rx) = tokio::sync::mpsc::channel(64);
        LlmService::new(driver, cap, CancellationToken::new(), tx)
    }

    fn gtx() -> tokio::sync::mpsc::Sender<crate::events::GlossaryEvent> {
        tokio::sync::mpsc::channel(64).0
    }

    #[tokio::test(start_paused = true)]
    async fn normalizes_each_nonempty_category() {
        let mut g = Glossary::new("xianxia");
        g.characters.insert("林动".into(), "lin dong".into());
        // Only one non-empty category → exactly one LLM call.
        let d = ScriptedDriver::new(vec![Ok(r#"{"林动":"Lin Dong"}"#.into())]);
        let out = normalize_pass(&svc(d.clone(), 2), &g, &gtx()).await;
        assert_eq!(out.characters.get("林动").unwrap(), "Lin Dong");
        assert_eq!(d.call_count(), 1);
        assert_eq!(out.world_type, "xianxia");
    }

    #[tokio::test(start_paused = true)]
    async fn failed_category_keeps_original_terms() {
        let mut g = Glossary::new("wuxia");
        g.characters.insert("张三".into(), "zhang san".into());
        g.locations.insert("华山".into(), "Mount Hua".into());
        // cap 1 + non-retryable error → one driver call per category, in
        // CATEGORIES order (characters before locations) — deterministic.
        let d = ScriptedDriver::new(vec![
            Err(LlmError::Http { status: 400, body: "bad".into() }), // characters fails
            Ok(r#"{"华山":"Mt. Hua"}"#.into()),                       // locations succeeds
        ]);
        let out = normalize_pass(&svc(d, 1), &g, &gtx()).await;
        assert_eq!(out.characters.get("张三").unwrap(), "zhang san"); // kept
        assert_eq!(out.locations.get("华山").unwrap(), "Mt. Hua"); // replaced
    }

    #[tokio::test(start_paused = true)]
    async fn unparseable_response_keeps_original() {
        let mut g = Glossary::new("modern");
        g.items.insert("a".into(), "A".into());
        let d = ScriptedDriver::new(vec![Ok("I refuse to answer with JSON".into())]);
        let out = normalize_pass(&svc(d, 2), &g, &gtx()).await;
        assert_eq!(out.items.get("a").unwrap(), "A");
    }

    #[tokio::test(start_paused = true)]
    async fn normalized_values_are_revalidated() {
        let mut g = Glossary::new("modern");
        g.skills.insert("k1".into(), "V1".into());
        g.skills.insert("k2".into(), "V2".into());
        // LLM merges k2 away, empties k1's value (invalid → dropped), adds k3.
        let d = ScriptedDriver::new(vec![Ok(r#"{"k1":"   ","k3":"  V3  "}"#.into())]);
        let out = normalize_pass(&svc(d, 2), &g, &gtx()).await;
        assert!(!out.skills.contains_key("k1")); // empty value dropped
        assert!(!out.skills.contains_key("k2")); // legitimately merged away
        assert_eq!(out.skills.get("k3").unwrap(), "V3"); // trimmed
    }

    #[tokio::test(start_paused = true)]
    async fn empty_glossary_makes_no_calls() {
        let d = ScriptedDriver::new(vec![]); // would panic if called
        let g = Glossary::new("modern");
        let out = normalize_pass(&svc(d, 2), &g, &gtx()).await;
        assert!(out.is_empty());
    }
}
