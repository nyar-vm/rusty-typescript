//! 补全匹配器模块
//!
//! 提供模糊匹配功能，提高补全的准确性和可用性。

use super::*;

/// 补全匹配器
pub struct CompletionMatcher;

impl CompletionMatcher {
    /// 执行模糊匹配
    /// Returns true if the prefix is a subsequence of the label
    pub fn fuzzy_match(label: &str, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        }

        let mut label_iter = label.chars();

        for prefix_char in prefix.chars() {
            match label_iter.find(|&c| c.eq_ignore_ascii_case(&prefix_char)) {
                Some(_) => continue,
                None => return false,
            }
        }

        true
    }

    /// 计算匹配分数
    pub fn calculate_match_score(label: &str, prefix: &str) -> f64 {
        if prefix.is_empty() {
            return 1.0;
        }

        if label.starts_with(prefix) {
            1.0
        }
        else if label.contains(prefix) {
            0.7
        }
        else if Self::fuzzy_match(label, prefix) {
            0.5
        }
        else {
            0.3
        }
    }
}
