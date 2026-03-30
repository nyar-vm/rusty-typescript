//! 代码补全模块
//!
//! 提供 TypeScript 语言服务器的代码补全功能，包括上下文感知、智能排序和模糊匹配等特性。

mod context;
mod matcher;
mod provider;
mod sorter;

pub use context::{CompletionContext, CompletionContextType};
pub use matcher::CompletionMatcher;
pub use provider::CompletionProvider;
pub use sorter::CompletionSorter;
