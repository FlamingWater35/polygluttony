//! Batch sizing math: clamp to [10, 260], halve on failure. Constants are the
//! Python pipeline's (`core/constants.py`).

pub const BATCH_LINE_LIMIT: u32 = 260;
pub const LAST_RESORT_LINES_PER_BATCH: u32 = 10;
pub const CONTEXT_CARRYOVER_LINES: usize = 7;
pub const MAX_RETRANSLATION_ATTEMPTS: u32 = 3;
pub const MAX_CLEANUP_LINES: usize = 10;
pub const MAX_CLEANUP_ITERATIONS: u32 = 3;

/// Initial size from the connection (`batch_dialogue_limit`), clamped.
pub fn initial_batch_size(connection_limit: Option<u32>) -> u32 {
    connection_limit
        .unwrap_or(BATCH_LINE_LIMIT)
        .clamp(LAST_RESORT_LINES_PER_BATCH, BATCH_LINE_LIMIT)
}

/// Halve toward the floor; `None` when already at the floor (give up signal).
pub fn halved(current: u32) -> Option<u32> {
    if current <= LAST_RESORT_LINES_PER_BATCH {
        return None;
    }
    Some((current / 2).max(LAST_RESORT_LINES_PER_BATCH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_initial_size() {
        assert_eq!(initial_batch_size(None), 260);
        assert_eq!(initial_batch_size(Some(100)), 100);
        assert_eq!(initial_batch_size(Some(5)), 10);
        assert_eq!(initial_batch_size(Some(1000)), 260);
    }

    #[test]
    fn halves_to_floor_then_gives_up() {
        assert_eq!(halved(100), Some(50));
        assert_eq!(halved(15), Some(10));
        assert_eq!(halved(10), None);
    }
}
