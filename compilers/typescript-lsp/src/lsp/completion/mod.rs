//! 代码补全模块
//! 
//! 提供 TypeScript 语言服务器的代码补全功能，包括上下文感知、智能排序和模糊匹配等特性。

mod context;
mod matcher;
mod sorter;
mod provider;

pub use context::CompletionContext;
pub use matcher::CompletionMatcher;
pub use sorter::CompletionSorter;
pub use provider::CompletionProvider;
