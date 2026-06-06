//! Instant, keyword-only world-type detection (no LLM). Ported from
//! `glossary/world_detector.py`. Counts Chinese keyword occurrences per category;
//! the highest wins (ties: xianxia > wuxia > historical); none → modern.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Detected story world. Tunes glossary extraction + tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub enum WorldType {
    Xianxia,
    Wuxia,
    Historical,
    Modern,
}

impl WorldType {
    /// Lowercase string for prompts and the glossary.json `world_type` field
    /// (matches the serde `lowercase` rename).
    pub fn as_str(self) -> &'static str {
        match self {
            WorldType::Xianxia => "xianxia",
            WorldType::Wuxia => "wuxia",
            WorldType::Historical => "historical",
            WorldType::Modern => "modern",
        }
    }
}

const XIANXIA: &[&str] = &[
    "修仙", "筑基", "金丹", "元婴", "渡劫", "灵气", "仙人", "修炼", "灵石", "丹药",
    "法宝", "飞剑", "结丹", "化神", "修真", "仙界", "魔界", "灵根", "天劫", "飞升",
];
const WUXIA: &[&str] = &[
    "武林", "江湖", "门派", "内力", "轻功", "武功", "剑法", "掌法", "拳法", "气功",
    "真气", "武者", "侠客", "大侠", "盟主", "帮派",
];
const HISTORICAL: &[&str] = &[
    "皇帝", "朝廷", "太监", "皇后", "大臣", "科举", "宰相", "王爷", "公主", "皇宫",
    "后宫", "朝代", "太子", "皇上", "圣旨",
];

fn count(content: &str, keywords: &[&str]) -> usize {
    keywords.iter().map(|kw| content.matches(kw).count()).sum()
}

/// Detect the world type from combined dialogue `content`. When the source
/// language doesn't support detection, returns `Modern` without scanning.
pub fn detect(content: &str, supports_world_detection: bool) -> WorldType {
    if !supports_world_detection {
        return WorldType::Modern;
    }
    let x = count(content, XIANXIA);
    let w = count(content, WUXIA);
    let h = count(content, HISTORICAL);
    let max = x.max(w).max(h);
    if max == 0 {
        WorldType::Modern
    } else if x == max {
        WorldType::Xianxia
    } else if w == max {
        WorldType::Wuxia
    } else {
        WorldType::Historical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_world() {
        assert_eq!(detect("修仙者突破了金丹期，灵气充沛，准备渡劫飞升", true), WorldType::Xianxia);
        assert_eq!(detect("武林盟主召集江湖各大门派，讨论轻功和内力修炼", true), WorldType::Wuxia);
        assert_eq!(detect("皇帝在朝廷上接见了宰相和大臣，商议科举事宜", true), WorldType::Historical);
        assert_eq!(detect("今天天气不错，我们去公园散步吧", true), WorldType::Modern);
    }

    #[test]
    fn no_detection_when_unsupported() {
        assert_eq!(detect("修仙金丹渡劫", false), WorldType::Modern);
    }

    #[test]
    fn ties_break_in_priority_order() {
        // one xianxia + one wuxia keyword → xianxia wins the tie.
        assert_eq!(detect("修仙 江湖", true), WorldType::Xianxia);
    }

    #[test]
    fn as_str_is_lowercase_and_total() {
        assert_eq!(WorldType::Xianxia.as_str(), "xianxia");
        assert_eq!(WorldType::Wuxia.as_str(), "wuxia");
        assert_eq!(WorldType::Historical.as_str(), "historical");
        assert_eq!(WorldType::Modern.as_str(), "modern");
    }
}
