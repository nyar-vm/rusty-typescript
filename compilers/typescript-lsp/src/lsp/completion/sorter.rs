//! 补全排序器模块
//! 
//! 提供智能排序功能，根据相关性和使用频率排序补全项。

use super::*;
use oak_lsp::types::CompletionItem;
use std::collections::HashMap;

/// 补全排序器
pub struct CompletionSorter {
    /// 跟踪补全项的使用频率
    usage_frequency: HashMap<String, u32>,
}

impl CompletionSorter {
    /// 创建新的补全排序器
    pub fn new() -> Self {
        Self {
            usage_frequency: HashMap::new(),
        }
    }
    
    /// 更新使用频率
    pub fn update_usage(&mut self, item: &str) {
        *self.usage_frequency.entry(item.to_string()).or_insert(0) += 1;
    }
    
    /// 计算相关性分数
    pub fn calculate_relevance(&self, item: &CompletionItem, prefix: &str) -> f64 {
        let label = item.label.as_str();
        
        // Base score based on prefix match
        let mut score = if label.starts_with(prefix) {
            1.0
        } else if label.contains(prefix) {
            0.7
        } else if super::matcher::CompletionMatcher::fuzzy_match(label, prefix) {
            0.5
        } else {
            0.3
        };
        
        // Adjust score based on item kind
        if let Some(kind) = item.kind {
            match kind {
                oak_lsp::types::CompletionItemKind::Function => score += 0.2,
                oak_lsp::types::CompletionItemKind::Variable => score += 0.15,
                oak_lsp::types::CompletionItemKind::Class => score += 0.1,
                _ => {}
            }
        }
        
        // Adjust score based on usage frequency
        if let Some(freq) = self.usage_frequency.get(label) {
            score += (*freq as f64) * 0.01;
        }
        
        score
    }
    
    /// 排序补全项
    pub fn sort(&self, completions: &mut Vec<CompletionItem>, prefix: &str) {
        completions.sort_by(|a, b| {
            let score_a = self.calculate_relevance(a, prefix);
            let score_b = self.calculate_relevance(b, prefix);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}
