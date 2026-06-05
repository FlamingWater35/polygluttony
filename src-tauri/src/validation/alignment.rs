//! Layer-0 structural validation: every requested id came back, nothing extra,
//! nothing empty. Port of `validation/alignment.py`.

use super::LinePair;

#[derive(Debug, Clone, Default)]
pub struct AlignmentCheck {
    pub is_valid: bool,
    pub first_problem_id: Option<u32>,
    pub missing_ids: Vec<u32>,
    pub extra_ids: Vec<u32>,
    pub empty_translations: Vec<u32>,
}

pub fn check(expected_ids: &[u32], output: &[LinePair]) -> AlignmentCheck {
    let mut r = AlignmentCheck { is_valid: true, ..Default::default() };
    let out_ids: Vec<u32> = output.iter().map(|p| p.id).collect();

    r.missing_ids = expected_ids.iter().copied().filter(|id| !out_ids.contains(id)).collect();
    r.extra_ids = out_ids.iter().copied().filter(|id| !expected_ids.contains(id)).collect();
    r.empty_translations =
        output.iter().filter(|p| p.tgt.trim().is_empty()).map(|p| p.id).collect();

    if !r.missing_ids.is_empty() || !r.extra_ids.is_empty() || !r.empty_translations.is_empty() {
        r.is_valid = false;
        r.first_problem_id = r
            .missing_ids
            .iter()
            .chain(r.extra_ids.iter())
            .chain(r.empty_translations.iter())
            .copied()
            .min();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::LinePair;

    fn pair(id: u32, tgt: &str) -> LinePair {
        LinePair { id, src: format!("src{id}"), tgt: tgt.into() }
    }

    #[test]
    fn valid_when_ids_match_and_targets_nonempty() {
        let r = check(&[1, 2], &[pair(1, "a"), pair(2, "b")]);
        assert!(r.is_valid);
        assert_eq!(r.first_problem_id, None);
    }

    #[test]
    fn missing_and_extra_ids() {
        let r = check(&[1, 2, 3], &[pair(1, "a"), pair(4, "d")]);
        assert!(!r.is_valid);
        assert_eq!(r.missing_ids, vec![2, 3]);
        assert_eq!(r.extra_ids, vec![4]);
        assert_eq!(r.first_problem_id, Some(2));
    }

    #[test]
    fn empty_translation_is_a_problem() {
        let r = check(&[1, 2], &[pair(1, "a"), pair(2, "   ")]);
        assert!(!r.is_valid);
        assert_eq!(r.empty_translations, vec![2]);
        assert_eq!(r.first_problem_id, Some(2));
    }
}
