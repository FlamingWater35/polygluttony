//! Glossary extraction and management.
//!
//! Ports the Python `glossary/` package: collect dialogue across files, detect
//! world type (xianxia / wuxia / historical / modern), extract the six-category
//! glossary in parallel via the LLM, dedupe/merge, and optionally normalize and
//! personalize. Supports injecting reference terminology.
//!
//! Planned submodules: `glossary`, `world_detector`, `build_result`, `diff`,
//! `reference_loader`, `reference_terminology`, `reference_extractor`.

pub mod io;
pub mod model;
pub mod world_detector;