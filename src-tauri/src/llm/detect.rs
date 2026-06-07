//! Custom-connection API-format detection (spec §5.7). Probes both wire formats
//! and disambiguates by HTTP status: any non-404 error response means the route
//! exists (so a wrong key — 401/403 — still reveals the format).
//!
//! One status is NOT trusted on its own: HTTP 200. Some gateways (e.g. Z.AI's
//! `api.z.ai/api/anthropic`) answer unknown routes with a "soft 404" — HTTP 200
//! plus an error JSON like `{"code":500,"msg":"404 NOT_FOUND"}`. A nonsense
//! probe ("model":"probe", throwaway key) returning 200 therefore only counts
//! as the route existing when the body carries the protocol's own keys.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::config::Driver;
use crate::llm::error::LlmError;

enum Probe {
    Exists,
    NotHere,
    Unreachable,
}

async fn probe(
    client: &reqwest::Client,
    url: &str,
    headers: HeaderMap,
    body: serde_json::Value,
    protocol_keys: &[&str],
) -> Probe {
    match client
        .post(url)
        .headers(headers)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => match resp.status().as_u16() {
            404 => Probe::NotHere,
            // A 200 to a nonsense probe is suspicious (soft-404 gateways, see
            // module doc): trust it only when the body speaks the protocol.
            200 => match resp.json::<serde_json::Value>().await {
                Ok(v) if protocol_keys.iter().any(|k| v.get(k).is_some()) => Probe::Exists,
                _ => Probe::NotHere,
            },
            // Auth/validation errors (401/403/400/...) mean a real handler saw
            // the request — the route exists even though the key is wrong.
            _ => Probe::Exists,
        },
        Err(_) => Probe::Unreachable,
    }
}

/// Determine whether a base URL speaks OpenAI- or Anthropic-style.
pub async fn detect_format(base_url: &str, api_key: &str) -> Result<Driver, LlmError> {
    let client = reqwest::Client::new();
    let base = base_url.trim_end_matches('/');

    // Probe 1: OpenAI chat completions.
    let mut oai = HeaderMap::new();
    oai.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(v) = HeaderValue::from_str(&format!("Bearer {api_key}")) {
        oai.insert(AUTHORIZATION, v);
    }
    let oai_body = json!({"model":"probe","max_tokens":1,
        "messages":[{"role":"user","content":"ping"}]});
    if let Probe::Exists = probe(
        &client,
        &format!("{base}/chat/completions"),
        oai,
        oai_body,
        &["choices", "object", "error"],
    )
    .await
    {
        return Ok(Driver::Openai);
    }

    // Probe 2: Anthropic messages.
    let mut ant = HeaderMap::new();
    ant.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Ok(v) = HeaderValue::from_str(api_key) {
        ant.insert("x-api-key", v);
    }
    ant.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    let ant_body = json!({"model":"probe","max_tokens":1,
        "messages":[{"role":"user","content":"ping"}]});
    if let Probe::Exists = probe(
        &client,
        &format!("{base}/v1/messages"),
        ant,
        ant_body,
        &["content", "type", "error"],
    )
    .await
    {
        return Ok(Driver::Anthropic);
    }

    Err(LlmError::Transport(
        "couldn't determine the API format at this URL".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Driver;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn detects_openai_when_chat_route_exists() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401)) // exists but bad key
            .mount(&server).await;
        // No /v1/messages mounted -> wiremock returns 404 for it.
        let d = detect_format(&server.uri(), "k").await.unwrap();
        assert_eq!(d, Driver::Openai);
    }

    #[tokio::test]
    async fn detects_anthropic_when_only_messages_route_exists() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server).await;
        let d = detect_format(&server.uri(), "k").await.unwrap();
        assert_eq!(d, Driver::Anthropic);
    }

    #[tokio::test]
    async fn undetermined_when_both_404() {
        let server = MockServer::start().await; // nothing mounted -> all 404
        assert!(detect_format(&server.uri(), "k").await.is_err());
    }

    /// Z.AI's gateway answers unknown routes with HTTP 200 + an error JSON
    /// ("soft 404": `{"code":500,"msg":"404 NOT_FOUND","success":false}`).
    /// The OpenAI probe must not mistake that for a real chat/completions
    /// route — the Anthropic probe must still run and win.
    #[tokio::test]
    async fn soft_404_on_openai_route_still_detects_anthropic() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 500, "msg": "404 NOT_FOUND", "success": false
            })))
            .mount(&server).await;
        Mock::given(method("POST")).and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(401)) // exists, bad key
            .mount(&server).await;
        let d = detect_format(&server.uri(), "k").await.unwrap();
        assert_eq!(d, Driver::Anthropic);
    }

    /// A genuine 200 carrying OpenAI protocol keys still detects as OpenAI
    /// (e.g. a keyless local server that answers any model name).
    #[tokio::test]
    async fn genuine_openai_200_still_detects_openai() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "chat.completion", "choices": []
            })))
            .mount(&server).await;
        let d = detect_format(&server.uri(), "k").await.unwrap();
        assert_eq!(d, Driver::Openai);
    }

    /// Soft-404s on BOTH routes -> undetermined, not a false positive.
    #[tokio::test]
    async fn soft_404_on_both_routes_is_undetermined() {
        let server = MockServer::start().await;
        let soft = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 500, "msg": "404 NOT_FOUND", "success": false
        }));
        Mock::given(method("POST")).and(path("/chat/completions"))
            .respond_with(soft.clone()).mount(&server).await;
        Mock::given(method("POST")).and(path("/v1/messages"))
            .respond_with(soft).mount(&server).await;
        assert!(detect_format(&server.uri(), "k").await.is_err());
    }
}
