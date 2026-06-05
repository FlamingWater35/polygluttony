//! Translation orchestration.
//!
//! Ports the Python `core/` and `handlers/` packages: token-aware batching,
//! concurrent per-file translation with retry/cleanup passes, scope calculation,
//! progress reporting, and LLM-based verification. Drives the glossary →
//! translate → verify pipeline and emits progress via [`crate::events`].
//!
//! Planned submodules: `translator`, `batch_manager`, `batch_translator`,
//! `scope_calculator`, `verifier`, `progress`.

pub mod parse_response;
