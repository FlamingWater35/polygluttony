//! Shared SSE plumbing: POST a JSON body, yield each `data:` payload as parsed
//! JSON. Built on `eventsource_stream` over reqwest's byte stream.

use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::HeaderMap;
use serde_json::Value;

use super::error::LlmError;

/// POST and fold every SSE `data:` JSON through `on_event`. Non-JSON data
/// payloads (e.g. OpenAI's `[DONE]`) are skipped. `on_event` may return
/// `Err` to abort the stream immediately; the error is propagated to the
/// caller.
pub(crate) async fn post_sse<F>(
    client: &reqwest::Client,
    url: &str,
    headers: HeaderMap,
    body: &Value,
    mut on_event: F,
) -> Result<(), LlmError>
where
    F: FnMut(Value) -> Result<(), LlmError>,
{
    let resp = client
        .post(url)
        .headers(headers)
        .json(body)
        .send()
        .await
        .map_err(|e| LlmError::Transport(e.to_string()))?;
    let status = resp.status();
    if !status.is_success() {
        let retry_after = super::retry_after_secs(resp.headers());
        let body = resp.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(500).collect();
        return Err(LlmError::Http { status: status.as_u16(), body: snippet, retry_after });
    }
    let mut stream = resp.bytes_stream().eventsource();
    while let Some(ev) = stream.next().await {
        let ev = ev.map_err(|e| LlmError::Transport(e.to_string()))?;
        if ev.data == "[DONE]" {
            break;
        }
        if let Ok(v) = serde_json::from_str::<Value>(&ev.data) {
            on_event(v)?;
        }
    }
    Ok(())
}
