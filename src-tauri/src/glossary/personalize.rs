//! Personalization pass (build step 8): one call on the web-capable
//! personalization connection to align terms with established fan usage.
//! Port of `glossary_builder.py:535-563`.

use crate::glossary::model::Glossary;
use crate::glossary::prompts;
use crate::llm::service::LlmService;
use crate::llm::LlmRequest;
use crate::translation::parse_response;

/// `Err(reason)` ⇒ caller keeps the original glossary and records the reason.
/// A response with zero usable terms is an error — accepting it would replace
/// the whole glossary with nothing (latent Python hazard, guarded here).
///
/// **Deviation:** Python honoured the response's own `world_type` field
/// (`glossary.py:237`, defaulting to "xianxia" when absent). We always
/// preserve the original glossary's `world_type` — user overrides must win.
pub async fn personalize_pass(
    svc: &LlmService,
    glossary: &Glossary,
    context: &str,
) -> Result<Glossary, String> {
    let world =
        if glossary.world_type.is_empty() { "modern" } else { glossary.world_type.as_str() };
    let req = LlmRequest {
        system: prompts::personalize_prompt(world, context),
        user: prompts::personalize_user_prompt(glossary, context),
    };
    let resp = svc.request(req).await.map_err(|e| format!("personalize request failed: {e}"))?;
    let v = parse_response::extract_object(&resp.text)
        .map_err(|e| format!("personalize response unparseable: {e}"))?;
    let out = Glossary::from_terms_value(&v, &glossary.world_type);
    if out.is_empty() {
        return Err("personalize response contained no terms — keeping original".into());
    }
    Ok(out)
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

    fn svc(driver: Arc<ScriptedDriver>) -> LlmService {
        let (tx, _rx) = tokio::sync::mpsc::channel(64);
        LlmService::new(driver, 1, CancellationToken::new(), tx)
    }

    fn glossary() -> Glossary {
        let mut g = Glossary::new("xianxia");
        g.characters.insert("林动".into(), "Lin Dong".into());
        g
    }

    #[tokio::test(start_paused = true)]
    async fn replaces_glossary_on_success() {
        let d = ScriptedDriver::new(vec![Ok(
            r#"{"world_type":"xianxia","terms":{"characters":{"林动":"Lin Dong (MC)"}}}"#.into(),
        )]);
        let out = personalize_pass(&svc(d.clone()), &glossary(), "Martial Universe").await.unwrap();
        assert_eq!(out.characters.get("林动").unwrap(), "Lin Dong (MC)");
        // Prompt carried the title + glossary JSON.
        let req = d.last_request().unwrap();
        assert!(req.system.contains("Martial Universe"));
        assert!(req.user.contains("Lin Dong"));
    }

    #[tokio::test(start_paused = true)]
    async fn request_failure_returns_err() {
        let d = ScriptedDriver::new(vec![Err(LlmError::Http { status: 401, body: "no".into(), retry_after: None })]);
        assert!(personalize_pass(&svc(d), &glossary(), "").await.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn empty_or_unparseable_response_returns_err() {
        let d = ScriptedDriver::new(vec![Ok("sorry, no JSON".into())]);
        assert!(personalize_pass(&svc(d), &glossary(), "").await.is_err());
        // Parses but contains zero terms → would wipe the glossary → Err.
        let d = ScriptedDriver::new(vec![Ok(r#"{"terms":{}}"#.into())]);
        assert!(personalize_pass(&svc(d), &glossary(), "").await.is_err());
    }

    /// Pin the "modern" fallback: a glossary with an empty world_type should
    /// send "modern" in the system prompt.
    #[tokio::test(start_paused = true)]
    async fn empty_world_type_uses_modern_in_prompt() {
        let mut g = Glossary::new("");
        g.characters.insert("林动".into(), "Lin Dong".into());
        let d = ScriptedDriver::new(vec![Ok(
            r#"{"terms":{"characters":{"林动":"Lin Dong"}}}"#.into(),
        )]);
        personalize_pass(&svc(d.clone()), &g, "").await.unwrap();
        let req = d.last_request().unwrap();
        assert!(req.system.contains("modern"), "expected 'modern' in system prompt");
    }
}
