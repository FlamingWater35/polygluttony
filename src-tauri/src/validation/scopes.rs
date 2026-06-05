//! Minimal retranslation scopes from verification samples. Port of
//! `core/scope_calculator.py`: passing samples are trust boundaries; scopes
//! cover failed groups between them, padded for context, merged on overlap.

use std::collections::BTreeSet;

pub const FULL_RETRANSLATION_THRESHOLD: f64 = 0.8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub start_line: u32,
    pub end_line: u32,
    pub context_start: u32,
    pub context_end: u32,
}

/// Compute minimal retranslation ranges from verification results.
///
/// Algorithm:
/// 1. If all samples failed → return full-file scope (fallback).
/// 2. Walk the sorted sample list to find contiguous groups of failed samples
///    (contiguous = no passing sample between them).
/// 3. For each group: find the nearest passing sample before (prev) and after
///    (next); scope = (prev+1 .. next-1), defaulting to file boundaries when
///    there is no bounding passing sample.
/// 4. Apply context padding, clamped to the file.
/// 5. Merge scopes whose context regions touch or overlap
///    (`next.context_start <= current.context_end + 1`).
pub fn compute_scopes(
    sampled_ids: &[u32],
    failed_ids: &BTreeSet<u32>,
    all_line_ids: &[u32],
    padding: u32,
) -> Vec<Scope> {
    if all_line_ids.is_empty() || sampled_ids.is_empty() || failed_ids.is_empty() {
        return Vec::new();
    }

    let file_start = all_line_ids[0];
    let file_end = *all_line_ids.last().unwrap();

    // Passing samples = sampled minus failed.
    let passed: BTreeSet<u32> =
        sampled_ids.iter().copied().filter(|id| !failed_ids.contains(id)).collect();

    // Edge case: all samples failed → full-file scope.
    if passed.is_empty() {
        return vec![Scope {
            start_line: file_start,
            end_line: file_end,
            context_start: file_start,
            context_end: file_end,
        }];
    }

    // Walk sorted samples to find contiguous failed groups.
    let mut sorted_sampled: Vec<u32> = sampled_ids.to_vec();
    sorted_sampled.sort_unstable();

    let mut groups: Vec<(u32, u32)> = Vec::new();
    let mut current: Option<(u32, u32)> = None;
    for &s in &sorted_sampled {
        if failed_ids.contains(&s) {
            current = Some(match current {
                Some((start, _)) => (start, s),
                None => (s, s),
            });
        } else if let Some(g) = current.take() {
            groups.push(g);
        }
    }
    if let Some(g) = current {
        groups.push(g);
    }

    // For each group compute a padded scope.
    let mut scopes: Vec<Scope> = Vec::new();
    for (gstart, gend) in groups {
        // Nearest passing sample strictly before the group start.
        // If none exists, treat as file_start - 1 → scope starts at file_start.
        let scope_start = passed
            .iter()
            .copied()
            .filter(|&p| p < gstart)
            .max()
            .map(|p| p + 1)
            .unwrap_or(file_start);

        // Nearest passing sample strictly after the group end.
        // If none exists, treat as file_end + 1 → scope ends at file_end.
        let scope_end = passed
            .iter()
            .copied()
            .filter(|&p| p > gend)
            .min()
            .map(|p| p - 1)
            .unwrap_or(file_end);

        // Guard against degenerate inversion (shouldn't happen with well-formed
        // input, but mirror the Python safety net).
        let (scope_start, scope_end) = if scope_start > scope_end {
            (gstart, gend)
        } else {
            (scope_start, scope_end)
        };

        scopes.push(Scope {
            start_line: scope_start,
            end_line: scope_end,
            context_start: scope_start.saturating_sub(padding).max(file_start),
            context_end: (scope_end + padding).min(file_end),
        });
    }

    // Merge scopes whose context regions touch or overlap.
    scopes.sort_by_key(|s| s.start_line);
    let mut merged: Vec<Scope> = Vec::new();
    for s in scopes {
        match merged.last_mut() {
            Some(last) if s.context_start <= last.context_end + 1 => {
                last.end_line = last.end_line.max(s.end_line);
                last.context_end = last.context_end.max(s.context_end);
            }
            _ => merged.push(s),
        }
    }
    merged
}

/// Returns `true` if the scopes cover ≥ 80 % of `all_line_ids` — signal to
/// fall back to a full retranslation instead of targeted scope retranslation.
pub fn is_full_file(scopes: &[Scope], all_line_ids: &[u32]) -> bool {
    if all_line_ids.is_empty() || scopes.is_empty() {
        return false;
    }
    let covered = all_line_ids
        .iter()
        .filter(|id| scopes.iter().any(|s| s.start_line <= **id && **id <= s.end_line))
        .count();
    covered as f64 / all_line_ids.len() as f64 >= FULL_RETRANSLATION_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn ids(n: u32) -> Vec<u32> {
        (1..=n).collect()
    }

    #[test]
    fn no_failures_no_scopes() {
        let s = compute_scopes(&[10, 20, 30], &BTreeSet::new(), &ids(40), 5);
        assert!(s.is_empty());
    }

    #[test]
    fn failure_bounded_by_passing_neighbours() {
        // Samples 10,20,30,40; failures 20,30 → scope 11..39, padded 6..44.
        let failed: BTreeSet<u32> = [20, 30].into();
        let s = compute_scopes(&[10, 20, 30, 40], &failed, &ids(50), 5);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].start_line, s[0].end_line), (11, 39));
        assert_eq!((s[0].context_start, s[0].context_end), (6, 44));
    }

    #[test]
    fn all_samples_failed_means_full_file() {
        let failed: BTreeSet<u32> = [10, 20].into();
        let s = compute_scopes(&[10, 20], &failed, &ids(30), 5);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].start_line, s[0].end_line), (1, 30));
    }

    #[test]
    fn disjoint_failures_make_two_scopes_and_merge_when_overlapping() {
        // With padding=0 the two context regions do not touch (scope1 ends at
        // 49, scope2 starts at 51) so we get two distinct scopes.
        let failed: BTreeSet<u32> = [10, 90].into();
        let s = compute_scopes(&[10, 50, 90], &failed, &ids(100), 0);
        assert_eq!(s.len(), 2);
        // With padding ≥ 1 the context regions touch (ctx1_end = 49+p,
        // ctx2_start = 51-p; merge fires when 51-p <= 49+p+1 → p ≥ 0.5,
        // i.e. any padding ≥ 1), so a single merged scope results.
        let s = compute_scopes(&[10, 50, 90], &failed, &ids(100), 1);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn coverage_above_80_percent_is_full_file() {
        let scopes = vec![Scope {
            start_line: 1,
            end_line: 85,
            context_start: 1,
            context_end: 90,
        }];
        assert!(is_full_file(&scopes, &ids(100)));
        let scopes = vec![Scope {
            start_line: 1,
            end_line: 50,
            context_start: 1,
            context_end: 55,
        }];
        assert!(!is_full_file(&scopes, &ids(100)));
    }
}
