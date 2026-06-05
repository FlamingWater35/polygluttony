//! LLM driver error type. Mirrors `llm/anthropic.py:is_retryable_error`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    /// Non-2xx HTTP response, carrying the status + a body snippet.
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    /// Transport-level failure (timeout, connection reset, DNS).
    #[error("request error: {0}")]
    Transport(String),
    /// Response received but could not be parsed into the expected shape.
    #[error("failed to parse response: {0}")]
    Parse(String),
    /// 2xx response with no usable text content.
    #[error("empty response from LLM")]
    Empty,
}

impl LlmError {
    /// True for transient failures worth retrying (timeouts, 429, 5xx).
    pub fn is_retryable(&self) -> bool {
        match self {
            LlmError::Http { status, .. } => {
                matches!(status, 408 | 425 | 429 | 500 | 502 | 503 | 504 | 529)
            }
            LlmError::Transport(_) => true,
            LlmError::Empty => true,
            LlmError::Parse(_) => false,
        }
    }

    /// True for auth/endpoint errors that won't be fixed by retrying.
    pub fn is_auth(&self) -> bool {
        matches!(self, LlmError::Http { status, .. } if matches!(status, 401 | 403 | 404))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_errors_are_not_retryable() {
        for s in [401u16, 403, 404] {
            let e = LlmError::Http { status: s, body: "x".into() };
            assert!(!e.is_retryable(), "{s} should not retry");
            assert!(e.is_auth(), "{s} is auth");
        }
    }

    #[test]
    fn transient_errors_are_retryable() {
        for s in [429u16, 500, 502, 503, 504] {
            assert!(LlmError::Http { status: s, body: "x".into() }.is_retryable());
        }
        assert!(LlmError::Transport("timed out".into()).is_retryable());
    }

    #[test]
    fn parse_errors_are_not_retryable() {
        assert!(!LlmError::Parse("bad json".into()).is_retryable());
    }
}
