use std::fs;
use std::path::Path;

use dashmap::DashMap;

use crate::error::{AppError, Result};
use crate::parser::{extract_links, extract_stem, extract_tags, extract_title, replace_wikilink, LinkRef, NoteMeta};

pub struct LinkIndex {
    pub notes: DashMap<String, NoteMeta>,
    pub forward: DashMap<String, Vec<LinkRef>>,
    pub backrefs: DashMap<String, Vec<String>>,
    pub name_index: DashMap<String, Vec<String>>,
}

impl LinkIndex {
    pub fn new() -> Self {
        Self {
            notes: DashMap::new(),
            forward: DashMap::new(),
            backrefs: DashMap::new(),
            name_index: DashMap::new(),
        }
    }

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

    /// 重建反向链接索引（在 name_index 完整后调用，确保 backrefs 准确）
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

    pub fn on_rename(&self, old_path: &str, new_path: &str, vault_path: &Path) -> Result<()> {
        let old_stem = extract_stem(old_path);
        let new_stem = extract_stem(new_path);

        let _forward_links: Vec<LinkRef> = self
            .forward
            .get(old_path)
            .map(|v| v.clone())
            .unwrap_or_default();

        for source_path in self.backrefs.get(old_path).map(|v| v.clone()).unwrap_or_default() {
            let abs_path = vault_path.join(&source_path);
            let content = fs::read_to_string(&abs_path)?;
            let new_content = replace_wikilink(&content, &old_stem, &new_stem);

            if content != new_content {
                fs::write(&abs_path, new_content)?;
            }
        }

        self.remove_note(old_path)?;

        if let Some(content) = fs::read_to_string(vault_path.join(new_path)).ok() {
            let meta = fs::metadata(vault_path.join(new_path))?;
            let mtime = meta.modified()?.elapsed()?.as_millis() as i64;
            let size = meta.len();
            self.upsert_note(new_path, &content, mtime, size)?;
        }

        self.rebuild_backrefs();

        Ok(())
    }

    pub fn resolve_wikilink(&self, target: &str) -> Option<String> {
        self.name_index.get(target).and_then(|paths| {
            if paths.is_empty() {
                None
            } else {
                Some(paths[0].clone())
            }
        })
    }

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

    pub fn get_outlinks(&self, rel_path: &str) -> Vec<LinkRef> {
        self.forward.get(rel_path).map(|v| v.clone()).unwrap_or_default()
    }

    pub fn get_unresolved_links(&self, rel_path: &str) -> Vec<LinkRef> {
        let links = self.get_outlinks(rel_path);
        links
            .into_iter()
            .filter(|link| self.resolve_wikilink(&link.target).is_none())
            .collect()
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = std::collections::BTreeSet::new();
        for note in self.notes.iter() {
            for tag in &note.tags {
                tags.insert(tag.clone());
            }
        }
        tags.into_iter().collect()
    }

    pub fn list_all_notes(&self) -> Vec<NoteMeta> {
        self.notes.iter().map(|n| n.clone()).collect()
    }

    pub fn get_note_meta(&self, rel_path: &str) -> Option<NoteMeta> {
        self.notes.get(rel_path).map(|n| n.clone())
    }

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

#[derive(Debug, Clone, serde::Serialize)]
pub struct Backlink {
    pub source: String,
    pub source_title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub tag_count: u32,
    pub link_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}