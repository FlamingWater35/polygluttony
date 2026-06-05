//! Validation layers guarding LLM output: structural alignment (layer 0),
//! line markers (layer 1), drift signals (layer 2), and retranslation scopes.

pub mod alignment;
pub mod drift;
pub mod markers;
pub mod scopes;

/// One source line + its candidate translation, as parsed from an LLM response.
#[derive(Debug, Clone, PartialEq)]
pub struct LinePair {
    pub id: u32,
    pub src: String,
    pub tgt: String,
}
