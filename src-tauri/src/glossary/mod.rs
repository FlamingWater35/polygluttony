//! Glossary extraction and management.
//!
//! Ports the Python `glossary/` package: collect dialogue across files, detect
//! world type (xianxia / wuxia / historical / modern), extract the six-category
//! glossary in parallel via the LLM, dedupe/merge, and optionally normalize and
//! personalize. Supports injecting reference terminology.
//!
//! Current submodules:
//! - `model`   — `GlossaryDoc` IPC shape, term ops (merge, dedupe, parse)
//! - `io`      — atomic pretty-printed save + load for the glossary JSON file
//! - `diff`    — pure diff between two glossary snapshots (`GlossaryDiff`)
//! - `world_detector` — keyword-heuristic world-type detection (no LLM)
//! - `reference` — reference terminology types, cache, ref/ discovery (O11 pure half)

pub mod diff;
pub mod io;
pub mod model;
pub mod reference;
pub mod world_detector;