use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRef {
    pub target: String,
    pub alias: Option<String>,
    pub heading: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMeta {
    pub title: String,
    pub path: String,
    pub tags: Vec<String>,
    pub mtime: i64,
    pub size: u64,
}

lazy_static::lazy_static! {
    static ref WIKILINK_REGEX: Regex = Regex::new(r"\\?\[\\?\[([^\]|#]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\\?\]\\?\]").unwrap();
    static ref TAG_REGEX: Regex = Regex::new(r"#(\w[\w\-_]*)").unwrap();
    static ref EMBED_REGEX: Regex = Regex::new(r"!\\?\[\\?\[([^\]|#]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\\?\]\\?\]").unwrap();
    static ref HEADING_REGEX: Regex = Regex::new(r"^#\s+(.+)$").unwrap();
}

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

pub fn extract_tags(content: &str) -> Vec<String> {
    TAG_REGEX
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

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

pub fn extract_title(content: &str, filename: &str) -> String {
    for line in content.lines() {
        if let Some(cap) = HEADING_REGEX.captures(line) {
            return cap[1].trim().to_string();
        }
    }
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename)
        .to_string()
}

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

use std::path::Path;

pub fn extract_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}