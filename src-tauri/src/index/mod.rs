//! 索引模块
//!
//! 包含应用的核心索引功能：
//! - `graph.rs`: 链接图谱索引（正向链接、反向链接、图谱数据导出）
//! - `search.rs`: 全文搜索索引（基于 Tantivy 搜索引擎）

pub mod graph;
pub mod search;