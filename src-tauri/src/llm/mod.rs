//! LLM client layer.
//!
//! Ports the Python `llm/` package: a driver abstraction over the Anthropic,
//! OpenAI Chat Completions, and OpenAI Responses APIs, built on `reqwest`.
//!
//! Step 1 implements the one-shot [`LlmDriver::complete`] path plus a
//! `/models` listing — enough to power connection testing, model autocomplete,
//! and Custom API-format detection. Step 3 adds [`LlmDriver::stream`] for the
//! translation pipeline (avoids idle-timeouts on long batches).

pub mod anthropic;
pub mod detect;
pub mod error;
pub mod openai;
pub mod openai_responses;
pub mod sse;
pub mod service;

#[cfg(test)]
pub mod test_support;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::time::Duration;

use crate::config::{Connection, Driver};
use error::LlmError;

/// One streaming-capable request.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub usage: Usage,
}

/// A provider driver: one-shot completion, streamed completion, and a model
/// list.
#[async_trait]
pub trait LlmDriver: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> Result<String, LlmError>;
    /// Streamed completion, accumulated to a full response. Used by the whole
    /// translation pipeline — avoids provider idle-timeouts on long batches.
    async fn stream(&self, req: &LlmRequest) -> Result<LlmResponse, LlmError>;
    async fn list_models(&self) -> Result<Vec<String>, LlmError>;
    fn model(&self) -> &str;
}

/// Build the driver for a connection. For Custom, callers must resolve the
/// driver via `detect::detect_format` first; this trusts `conn.driver`.
pub fn create_driver(conn: Connection) -> Box<dyn LlmDriver> {
    match conn.driver {
        Driver::Anthropic => Box::new(anthropic::AnthropicDriver::new(conn)),
        Driver::Openai => Box::new(openai::OpenAiDriver::new(conn)),
        Driver::OpenaiResponses => Box::new(openai_responses::OpenAiResponsesDriver::new(conn)),
    }
}

/// POST JSON and return the parsed body, classifying failures into `LlmError`.
pub(crate) async fn post_json(
    client: &reqwest::Client,
    url: &str,
    headers: HeaderMap,
    body: &Value,
    timeout_secs: u64,
) -> Result<Value, LlmError> {
    let resp = client
        .post(url)
        .headers(headers)
        .json(body)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| LlmError::Transport(e.to_string()))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(500).collect();
        return Err(LlmError::Http { status: status.as_u16(), body: snippet });
    }
    resp.json::<Value>()
        .await
        .map_err(|e| LlmError::Parse(e.to_string()))
}

/// GET JSON (used by `list_models`).
pub(crate) async fn get_json(
    client: &reqwest::Client,
    url: &str,
    headers: HeaderMap,
    timeout_secs: u64,
) -> Result<Value, LlmError> {
    let resp = client
        .get(url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| LlmError::Transport(e.to_string()))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(500).collect();
        return Err(LlmError::Http { status: status.as_u16(), body: snippet });
    }
    resp.json::<Value>()
        .await
        .map_err(|e| LlmError::Parse(e.to_string()))
}

/// Parse `{ "data": [ { "id": "..." } ] }` model lists (OpenAI + Anthropic shape).
pub(crate) fn parse_model_ids(v: &Value) -> Vec<String> {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn base_of(conn: &Connection) -> String {
    conn.base_url.trim_end_matches('/').to_string()
}

pub(crate) fn timeout_of(conn: &Connection) -> u64 {
    conn.timeout.unwrap_or(120) as u64
}

/// Build a `reqwest::Client` for a driver connection.
///
/// - `connect_timeout` bounds the TCP handshake.
/// - `read_timeout` is an **idle read timeout** — it resets on every chunk
///   received, so a healthy long-running generation is never killed.  This
///   mirrors httpx `read` timeout semantics and replaces the old per-request
///   `.timeout()` that cut off streams after the total deadline.
pub(crate) fn client_for(conn: &Connection) -> reqwest::Client {
    let connect = Duration::from_secs(conn.connect_timeout.unwrap_or(10) as u64);
    let read = Duration::from_secs(timeout_of(conn));
    reqwest::Client::builder()
        .connect_timeout(connect)
        .read_timeout(read)
        .build()
        .expect("reqwest client")
}
