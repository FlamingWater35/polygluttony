//! Weighted multi-signal drift detection (layer 2). Port of
//! `validation/drift_detector.py`. Threshold 0.7; weights are calibrated —
//! keep in sync with the Python reference.

pub mod signals;

use std::collections::{BTreeMap, BTreeSet};

use super::LinePair;

pub const THRESHOLD: f64 = 0.7;

const WEIGHTS: [(&str, f64); 5] = [
    ("punctuation", 0.3),
    ("glossary_position", 0.4),
    ("sentence_type", 0.3),
    ("last_line", 0.4),
    ("length_ratio", 0.2),
];

#[derive(Debug, Clone, Default)]
pub struct DriftReport {
    pub has_suspected_drift: bool,
    /// Weighted sum; may exceed 1.0 when multiple signals fire (weights sum to 1.6).
    pub score: f64,
    /// signal name → weighted score
    pub signals: BTreeMap<&'static str, f64>,
    pub suspected_drift_start_id: Option<u32>,
    pub flagged_line_ids: BTreeSet<u32>,
}

pub fn detect(pairs: &[LinePair], terms: &BTreeMap<String, String>) -> DriftReport {
    let raw: [(&'static str, signals::Signal); 5] = [
        ("punctuation", signals::punctuation(pairs)),
        ("glossary_position", signals::glossary_position(pairs, terms)),
        ("sentence_type", signals::sentence_type(pairs)),
        ("last_line", signals::last_line(pairs, terms)),
        ("length_ratio", signals::length_ratio(pairs)),
    ];

    let mut report = DriftReport::default();

    for (name, (score, first, flagged)) in raw {
        let weight = WEIGHTS.iter().find(|(n, _)| *n == name).unwrap().1;
        let weighted = score * weight;
        report.signals.insert(name, weighted);
        report.score += weighted;

        if let Some(id) = first {
            report.suspected_drift_start_id = Some(match report.suspected_drift_start_id {
                Some(cur) => cur.min(id),
                None => id,
            });
        }

        report.flagged_line_ids.extend(flagged);
    }

    report.has_suspected_drift = report.score > THRESHOLD;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn pairs(v: &[(u32, &str, &str)]) -> Vec<crate::validation::LinePair> {
        v.iter()
            .map(|(id, s, t)| crate::validation::LinePair {
                id: *id,
                src: s.to_string(),
                tgt: t.to_string(),
            })
            .collect()
    }

    #[test]
    fn empty_pairs_returns_no_drift() {
        let r = detect(&[], &BTreeMap::new());
        assert!(!r.has_suspected_drift);
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn clean_translation_scores_low() {
        let p = pairs(&[
            (1, "你要去哪里？", "Where are you going?"),
            (2, "我去市场。", "I am going to the market."),
        ]);
        let r = detect(&p, &BTreeMap::new());
        assert!(!r.has_suspected_drift, "score {} signals {:?}", r.score, r.signals);
    }

    #[test]
    fn garbage_translation_trips_threshold() {
        // Question→statement violations, missing glossary terms, absurd length
        // ratios, empty last line — every signal fires.
        let mut terms = BTreeMap::new();
        terms.insert("星汉".to_string(), "Xinghan".to_string());
        let p = pairs(&[
            (1, "星汉你要去哪里？", "Banana."),
            (2, "星汉在吗？", "The weather is extraordinarily pleasant today indeed truly."),
            (3, "好。", ""),
        ]);
        let r = detect(&p, &terms);
        assert!(r.has_suspected_drift);
        assert_eq!(r.suspected_drift_start_id, Some(1));
        assert!(r.flagged_line_ids.contains(&1));
    }

    #[test]
    fn signal_punctuation_counts_question_mismatches() {
        let p = pairs(&[(1, "去哪？", "Going somewhere.")]);
        let (score, first, flagged) = signals::punctuation(&p);
        assert!(score > 0.0);
        assert_eq!(first, Some(1));
        assert!(flagged.contains(&1));
    }

    #[test]
    fn signal_length_ratio_flags_outliers() {
        let p = pairs(&[
            (1, "这是一个非常长的中文句子需要翻译", "No."), // ratio « 0.8
        ]);
        let (score, first, _) = signals::length_ratio(&p);
        assert!(score > 0.0);
        assert_eq!(first, Some(1));
    }

    #[test]
    fn signal_glossary_position_detects_missing_terms() {
        let mut terms = BTreeMap::new();
        terms.insert("凌天门".to_string(), "Lingtian Sect".to_string());
        let p = pairs(&[(1, "凌天门有救了", "The sect is saved")]); // translation absent
        let (score, first, _) = signals::glossary_position(&p, &terms);
        assert!(score > 0.0);
        assert_eq!(first, Some(1));
    }

    #[test]
    fn signal_last_line_penalizes_empty_tail() {
        let p = pairs(&[(1, "好", "OK"), (2, "走吧", "")]);
        let (score, _, flagged) = signals::last_line(&p, &BTreeMap::new());
        assert!(score >= 1.0);
        assert!(flagged.contains(&2));
    }
}
