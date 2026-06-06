//! Glossary build pipeline (O10). The orchestrator arrives with the build
//! task; `glossary_batches` is shared with the reference extractor.

/// Slice a cross-file line stream into batches of `limit × 0.7` lines
/// (`glossary_builder.py:136-138,235-241`; `reference_extractor.py:65-67,124-137`).
/// The 30% headroom leaves room for prompt overhead.
// consumed by the reference extractor and the build orchestrator (next task)
#[allow(dead_code)]
pub fn glossary_batches(lines: &[String], batch_limit: Option<u32>) -> Vec<String> {
    let limit = batch_limit.unwrap_or(crate::translation::batching::BATCH_LINE_LIMIT);
    let per = (((limit as f64) * 0.7) as usize).max(1);
    lines.chunks(per).map(|c| c.join("\n")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("line {i}")).collect()
    }

    #[test]
    fn batches_slice_at_seventy_percent() {
        // limit 10 → 7 lines per batch → 15 lines = 7 + 7 + 1.
        let b = glossary_batches(&lines(15), Some(10));
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].lines().count(), 7);
        assert_eq!(b[2], "line 14");
    }

    #[test]
    fn batches_floor_at_one_line_and_default_limit() {
        let b = glossary_batches(&lines(3), Some(1)); // 0.7 → floor 1
        assert_eq!(b.len(), 3);
        // Default = BATCH_LINE_LIMIT (260) → 182 per batch.
        let b = glossary_batches(&lines(183), None);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].lines().count(), 182);
    }

    #[test]
    fn empty_lines_give_no_batches() {
        assert!(glossary_batches(&[], Some(10)).is_empty());
    }
}
