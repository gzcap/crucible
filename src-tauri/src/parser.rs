//! Markdown 解析模块
//!
//! 提供 Markdown 内容的解析功能，包括：
//! - Wikilink 提取（[[目标]] 或 [[目标|别名]]）
//! - 标签提取（#tag）
//! - 嵌入提取（![[目标]]）
//! - 标题提取（# 标题）
//! - Wikilink 替换（重命名笔记时自动修复链接）

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 链接引用结构
///
/// 表示一个 wikilink 的解析结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRef {
    /// 链接目标（笔记名称）
    pub target: String,
    /// 链接别名（显示文本）
    pub alias: Option<String>,
    /// 锚点标题（可选）
    pub heading: Option<String>,
}

/// 笔记元数据结构
///
/// 表示一篇笔记的基本信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMeta {
    /// 笔记标题（取自一级标题或文件名）
    pub title: String,
    /// 笔记相对路径
    pub path: String,
    /// 标签列表
    pub tags: Vec<String>,
    /// 修改时间（毫秒时间戳）
    pub mtime: i64,
    /// 文件大小（字节）
    pub size: u64,
}

// ========== 正则表达式定义 ==========

lazy_static::lazy_static! {
    /// Wikilink 正则表达式
    ///
    /// 匹配 `[[目标]]`、`[[目标|别名]]`、`[[目标#标题]]`、`[[目标#标题|别名]]`
    /// 支持转义 `\[\[` 不匹配
    static ref WIKILINK_REGEX: Regex = Regex::new(r"\\?\[\\?\[([^\]|#]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\\?\]\\?\]").unwrap();

    /// 标签正则表达式
    ///
    /// 匹配 `#tag`、`#tag-name`、`#tag_name`
    static ref TAG_REGEX: Regex = Regex::new(r"#(\w[\w\-_]*)").unwrap();

    /// 嵌入正则表达式
    ///
    /// 匹配 `![[目标]]` 格式的嵌入链接
    static ref EMBED_REGEX: Regex = Regex::new(r"!\\?\[\\?\[([^\]|#]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\\?\]\\?\]").unwrap();

    /// 标题正则表达式
    ///
    /// 匹配一级标题 `# 标题`
    static ref HEADING_REGEX: Regex = Regex::new(r"^#\s+(.+)$").unwrap();
}

// ========== 解析函数 ==========

/// 提取文本中的所有 wikilink
///
/// 返回 `LinkRef` 结构体列表，包含目标、别名和标题信息。
pub fn extract_links(content: &str) -> Vec<LinkRef> {
    let mut links = Vec::new();
    for cap in WIKILINK_REGEX.captures_iter(content) {
        let target = cap[1].trim().to_string();
        let heading = cap.get(2).map(|m| m.as_str().trim().to_string());
        let alias = cap.get(3).map(|m| m.as_str().trim().to_string());
        links.push(LinkRef { target, heading, alias });
    }
    links
}

/// 提取文本中的所有标签
///
/// 返回标签名称列表（不含 `#` 前缀）。
pub fn extract_tags(content: &str) -> Vec<String> {
    TAG_REGEX
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// 提取文本中的所有嵌入链接
///
/// 返回 `LinkRef` 结构体列表，格式与 wikilink 相同。
pub fn extract_embeds(content: &str) -> Vec<LinkRef> {
    let mut embeds = Vec::new();
    for cap in EMBED_REGEX.captures_iter(content) {
        let target = cap[1].trim().to_string();
        let heading = cap.get(2).map(|m| m.as_str().trim().to_string());
        let alias = cap.get(3).map(|m| m.as_str().trim().to_string());
        embeds.push(LinkRef { target, heading, alias });
    }
    embeds
}

/// 提取笔记标题
///
/// 使用文件名（不含扩展名）作为标题。
pub fn extract_title(content: &str, filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename)
        .to_string()
}

/// 替换文本中的 wikilink
///
/// 将旧的 wikilink 目标替换为新目标，保留别名和标题信息。
/// 用于重命名笔记时自动修复相关链接。
pub fn replace_wikilink(content: &str, old_stem: &str, new_stem: &str) -> String {
    WIKILINK_REGEX.replace_all(content, |caps: &regex::Captures| {
        let target = &caps[1];
        if target == old_stem {
            let heading = caps.get(2).map(|m| format!("#{}", m.as_str())).unwrap_or_default();
            let alias = caps.get(3).map(|m| format!("|{}", m.as_str())).unwrap_or_default();
            format!("[[{}{}{}]]", new_stem, heading, alias)
        } else {
            caps[0].to_string()
        }
    }).to_string()
}

/// 提取路径的文件名（不含扩展名）
///
/// 用于从文件路径中获取 wikilink 的目标名称。
pub fn extract_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}