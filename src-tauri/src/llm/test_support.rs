//! Test-only scripted LLM driver shared by service/batch/verify/pipeline tests.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use super::error::LlmError;
use super::{LlmDriver, LlmRequest, LlmResponse, Usage};

/// Pops one canned outcome per stream() call; records requests.
pub struct ScriptedDriver {
    script: Mutex<VecDeque<Result<String, LlmError>>>,
    calls: AtomicU32,
    last_request: Mutex<Option<LlmRequest>>,
}

impl ScriptedDriver {
    pub fn new(script: Vec<Result<String, LlmError>>) -> Arc<Self> {
        Arc::new(Self {
            script: Mutex::new(script.into()),
            calls: AtomicU32::new(0),
            last_request: Mutex::new(None),
        })
    }

    pub fn call_count(&self) -> u32 {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn last_request(&self) -> Option<LlmRequest> {
        self.last_request.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmDriver for ScriptedDriver {
    async fn complete(&self, _s: &str, _u: &str) -> Result<String, LlmError> {
        unreachable!("service path uses stream()")
    }
    async fn stream(&self, req: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_request.lock().unwrap() = Some(req.clone());
        let next = self
            .script
            .lock()
            .unwrap()
            .pop_front()
            .expect("ScriptedDriver: script exhausted");
        next.map(|text| LlmResponse { text, usage: Usage::default() })
    }
    async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        Ok(vec![])
    }
    fn model(&self) -> &str {
        "scripted"
    }
}
