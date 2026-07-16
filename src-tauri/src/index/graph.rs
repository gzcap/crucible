//! 链接图谱索引
//!
//! 维护笔记间的链接关系，支持：
//! - 正向链接（笔记引用的其他笔记）
//! - 反向链接（引用该笔记的其他笔记）
//! - 链接解析（将 wikilink 名称解析为文件路径）
//! - 图谱数据导出（用于可视化）
//!
//! 使用 `DashMap` 实现线程安全的并发访问。

use std::fs;
use std::path::Path;

use dashmap::DashMap;

use crate::error::Result;
use crate::parser::{extract_links, extract_stem, extract_tags, extract_title, replace_wikilink, LinkRef, NoteMeta};

/// 链接索引
///
/// 维护笔记间的链接关系图，包含四个核心数据结构：
/// - notes: 笔记元数据（路径 -> NoteMeta）
/// - forward: 正向链接（源路径 -> 链接列表）
/// - backrefs: 反向链接（目标路径 -> 源路径列表）
/// - name_index: 名称索引（文件名称 -> 路径列表，用于解析 wikilink）
pub struct LinkIndex {
    /// 笔记元数据映射
    pub notes: DashMap<String, NoteMeta>,
    /// 正向链接映射
    pub forward: DashMap<String, Vec<LinkRef>>,
    /// 反向链接映射
    pub backrefs: DashMap<String, Vec<String>>,
    /// 名称索引映射（用于解析 wikilink）
    pub name_index: DashMap<String, Vec<String>>,
}

impl LinkIndex {
    /// 创建新的链接索引
    pub fn new() -> Self {
        Self {
            notes: DashMap::new(),
            forward: DashMap::new(),
            backrefs: DashMap::new(),
            name_index: DashMap::new(),
        }
    }

    /// 从 Vault 重建索引
    ///
    /// 遍历 Vault 目录下所有 `.md` 文件，重新构建完整索引。
    /// 所有笔记索引完成后，统一重建反向链接（此时 name_index 完整）。
    pub fn rebuild_from_vault(&self, vault_path: &Path) -> Result<()> {
        self.notes.clear();
        self.forward.clear();
        self.backrefs.clear();
        self.name_index.clear();

        for entry in walkdir::WalkDir::new(vault_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let path = entry.path();
            let rel_path = path.strip_prefix(vault_path)?.to_string_lossy().to_string();

            let content = fs::read_to_string(path)?;
            let mtime = entry.metadata()?.modified()?.elapsed()?.as_millis() as i64;
            let size = entry.metadata()?.len();

            self.upsert_note(&rel_path, &content, mtime, size)?;
        }

        // 所有笔记索引完成后，统一重建反向链接（此时 name_index 完整）
        self.rebuild_backrefs();

        Ok(())
    }

    /// 更新或插入笔记
    ///
    /// 更新笔记元数据、正向链接和名称索引。
    /// 注意：不会自动重建反向链接，需要手动调用 `rebuild_backrefs()`。
    pub fn upsert_note(&self, rel_path: &str, content: &str, mtime: i64, size: u64) -> Result<()> {
        let stem = extract_stem(rel_path);
        let title = extract_title(content, rel_path);
        let tags = extract_tags(content);
        let links = extract_links(content);

        let meta = NoteMeta {
            title,
            path: rel_path.to_string(),
            tags,
            mtime,
            size,
        };
        self.notes.insert(rel_path.to_string(), meta);

        self.forward.insert(rel_path.to_string(), links.clone());

        self.name_index
            .entry(stem)
            .or_insert_with(Vec::new)
            .push(rel_path.to_string());

        Ok(())
    }

    /// 重建反向链接索引
    ///
    /// 在 name_index 完整后调用，确保 backrefs 准确。
    /// 遍历所有正向链接，根据 name_index 解析目标路径，建立反向引用。
    pub fn rebuild_backrefs(&self) {
        self.backrefs.clear();
        for entry in self.forward.iter() {
            let source_path = entry.key().clone();
            for link in entry.value().iter() {
                if let Some(target_path) = self.resolve_wikilink(&link.target) {
                    let mut backref_entry = self
                        .backrefs
                        .entry(target_path)
                        .or_insert_with(Vec::new);
                    if !backref_entry.contains(&source_path) {
                        backref_entry.push(source_path.clone());
                    }
                }
            }
        }
    }

    /// 从索引中移除笔记
    ///
    /// 移除笔记元数据、正向链接和名称索引，然后重建反向链接。
    pub fn remove_note(&self, rel_path: &str) -> Result<()> {
        self.notes.remove(rel_path);
        self.forward.remove(rel_path);

        let stem = extract_stem(rel_path);
        if let Some(mut paths) = self.name_index.get_mut(&stem) {
            paths.retain(|p| p != rel_path);
            if paths.is_empty() {
                self.name_index.remove(&stem);
            }
        }

        // 重建反向链接（确保删除后 backrefs 一致）
        self.rebuild_backrefs();

        Ok(())
    }

    /// 处理笔记重命名
    ///
    /// 执行以下操作：
    /// 1. 解析旧 stem 和新 stem
    /// 2. 遍历 forward 找到所有引用旧 stem 的笔记
    /// 3. 正则替换 `[[old…]]` → `[[new…]]`（保留 heading/alias）
    /// 4. 写回文件并更新索引
    /// 5. 从索引中移除旧路径，插入新路径
    /// 6. 返回修改的文件列表供 emit modify 事件
    pub fn on_rename(&self, old_path: &str, new_path: &str, vault_path: &Path) -> Result<Vec<String>> {
        let old_stem = extract_stem(old_path);
        let new_stem = extract_stem(new_path);

        let mut modified_files = Vec::new();

        // 遍历所有笔记的正向链接，找到引用旧 stem 的笔记
        for entry in self.forward.iter() {
            let source_path = entry.key().clone();
            let links = entry.value();

            // 检查该笔记是否引用了旧 stem
            let has_ref = links.iter().any(|link| link.target == old_stem);

            if has_ref {
                let abs_path = vault_path.join(&source_path);
                let content = fs::read_to_string(&abs_path)?;
                let new_content = replace_wikilink(&content, &old_stem, &new_stem);

                if content != new_content {
                    fs::write(&abs_path, &new_content)?;

                    let meta = fs::metadata(&abs_path)?;
                    let mtime = meta.modified()?.elapsed()?.as_millis() as i64;
                    let size = meta.len();
                    let _ = self.upsert_note(&source_path, &new_content, mtime, size);

                    modified_files.push(source_path);
                }
            }
        }

        // 从索引中移除旧路径
        self.remove_note(old_path)?;

        // 重新索引新路径的笔记
        if let Some(content) = fs::read_to_string(vault_path.join(new_path)).ok() {
            let meta = fs::metadata(vault_path.join(new_path))?;
            let mtime = meta.modified()?.elapsed()?.as_millis() as i64;
            let size = meta.len();
            let _ = self.upsert_note(new_path, &content, mtime, size);
        }

        // 重建反向链接（确保所有修改后的引用源都已更新）
        self.rebuild_backrefs();

        Ok(modified_files)
    }

    /// 解析 wikilink 目标路径
    ///
    /// 根据 wikilink 目标名称查找对应的笔记路径。
    /// 如果存在多个同名笔记，返回第一个匹配的路径。
    pub fn resolve_wikilink(&self, target: &str) -> Option<String> {
        self.name_index.get(target).and_then(|paths| {
            if paths.is_empty() {
                None
            } else {
                Some(paths[0].clone())
            }
        })
    }

    /// 获取笔记的反向链接
    ///
    /// 返回引用该笔记的其他笔记列表，包含来源路径、标题和上下文片段。
    pub fn get_backlinks(&self, rel_path: &str) -> Vec<Backlink> {
        let mut result = Vec::new();
        if let Some(sources) = self.backrefs.get(rel_path) {
            for source in sources.iter() {
                if let Some(meta) = self.notes.get(source) {
                    let snippet = self.extract_snippet(source, rel_path);
                    result.push(Backlink {
                        source: source.clone(),
                        source_title: meta.title.clone(),
                        snippet,
                    });
                }
            }
        }
        result
    }

    /// 提取反向链接的上下文片段
    ///
    /// 从源文件中提取包含 wikilink 的行作为上下文。
    fn extract_snippet(&self, source_path: &str, target_path: &str) -> String {
        use crate::parser::extract_stem;
        let target_stem = extract_stem(target_path);
        if let Some(links) = self.forward.get(source_path) {
            for link in links.iter() {
                if link.target == target_stem {
                    return link
                        .alias
                        .clone()
                        .unwrap_or_else(|| format!("[[{}]]", link.target));
                }
            }
        }
        String::new()
    }

    /// 获取笔记的正向链接
    ///
    /// 返回该笔记引用的其他笔记列表。
    pub fn get_outlinks(&self, rel_path: &str) -> Vec<LinkRef> {
        self.forward.get(rel_path).map(|v| v.clone()).unwrap_or_default()
    }

    /// 获取未解析的链接
    ///
    /// 返回指向不存在笔记的链接列表。
    pub fn get_unresolved_links(&self, rel_path: &str) -> Vec<LinkRef> {
        let links = self.get_outlinks(rel_path);
        links
            .into_iter()
            .filter(|link| self.resolve_wikilink(&link.target).is_none())
            .collect()
    }

    /// 获取所有标签
    ///
    /// 返回去重后的标签列表，按字母顺序排序。
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = std::collections::BTreeSet::new();
        for note in self.notes.iter() {
            for tag in &note.tags {
                tags.insert(tag.clone());
            }
        }
        tags.into_iter().collect()
    }

    /// 获取所有笔记列表
    pub fn list_all_notes(&self) -> Vec<NoteMeta> {
        self.notes.iter().map(|n| n.clone()).collect()
    }

    /// 获取笔记元数据
    pub fn get_note_meta(&self, rel_path: &str) -> Option<NoteMeta> {
        self.notes.get(rel_path).map(|n| n.clone())
    }

    /// 导出图谱数据
    ///
    /// 返回笔记间的链接关系图，包含节点（笔记）和边（链接）。
    /// 边会去重：A→B 和 B→A 视为同一条边。
    pub fn export_graph(&self) -> GraphData {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut seen_edges = std::collections::HashSet::new();

        for note in self.notes.iter() {
            let outlink_count = self.get_outlinks(&note.path).len() as u32;
            let backlink_count = self
                .backrefs
                .get(&note.path)
                .map(|v| v.len() as u32)
                .unwrap_or(0);

            nodes.push(GraphNode {
                id: note.path.clone(),
                name: note.title.clone(),
                tag_count: note.tags.len() as u32,
                link_count: outlink_count + backlink_count,
            });

            let outlinks = self.get_outlinks(&note.path);
            for outlink in &outlinks {
                let resolved = self.resolve_wikilink(&outlink.target);
                if let Some(target_path) = resolved {
                    // 无向边去重：A→B 和 B→A 视为同一条边
                    let edge_key = if note.path < target_path {
                        format!("{}|{}", note.path, target_path)
                    } else {
                        format!("{}|{}", target_path, note.path)
                    };
                    if seen_edges.insert(edge_key) {
                        links.push(GraphLink {
                            source: note.path.clone(),
                            target: target_path,
                        });
                    }
                }
            }
        }

        GraphData { nodes, links }
    }
}

/// 反向链接结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct Backlink {
    /// 来源笔记路径
    pub source: String,
    /// 来源笔记标题
    pub source_title: String,
    /// 上下文片段
    pub snippet: String,
}

/// 图谱节点结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNode {
    /// 节点 ID（笔记路径）
    pub id: String,
    /// 节点名称（笔记标题）
    pub name: String,
    /// 标签数量
    pub tag_count: u32,
    /// 链接数量（正向+反向）
    pub link_count: u32,
}

/// 图谱边结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphLink {
    /// 源节点 ID
    pub source: String,
    /// 目标节点 ID
    pub target: String,
}

/// 图谱数据结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphData {
    /// 节点列表
    pub nodes: Vec<GraphNode>,
    /// 边列表
    pub links: Vec<GraphLink>,
}