//! 全文搜索索引
//!
//! 基于 Tantivy 搜索引擎实现全文搜索功能，支持：
//! - 全文搜索（标题和正文）
//! - 标签搜索
//! - 链接搜索
//! - 搜索结果高亮片段提取
//!
//! 使用 `Mutex` 和 `RwLock` 确保线程安全的并发访问。

use std::fs;
use std::path::Path;
use std::sync::RwLock;

use tantivy::collector::TopDocs;
use tantivy::query::{Query, TermQuery, QueryParser};
use tantivy::schema::{Field, IndexRecordOption, Schema, STORED, TEXT, Value};
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument, Term};

use crate::error::{AppError, Result};
use crate::parser::{extract_tags, extract_title};

/// 搜索索引
///
/// 基于 Tantivy 构建的全文搜索索引，包含以下字段：
/// - path: 文件路径（唯一标识）
/// - title: 笔记标题
/// - body: 笔记正文
/// - tags: 标签（空格分隔）
/// - mtime: 修改时间
pub struct SearchIndex {
    /// Tantivy Index 实例
    index: std::sync::Mutex<Option<Index>>,
    /// IndexWriter（用于写入和更新索引）
    writer: std::sync::Mutex<Option<IndexWriter>>,
    /// IndexReader（用于读取和搜索索引）
    reader: RwLock<Option<IndexReader>>,
    /// Schema 定义
    schema: Schema,
    /// 路径字段
    path_field: Field,
    /// 标题字段
    title_field: Field,
    /// 正文字段
    body_field: Field,
    /// 标签字段
    tags_field: Field,
    /// 修改时间字段
    mtime_field: Field,
}

impl SearchIndex {
    /// 创建新的搜索索引
    ///
    /// 定义 Schema 结构，包含 path、title、body、tags、mtime 五个字段。
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let mtime_field = schema_builder.add_i64_field("mtime", STORED);
        let schema = schema_builder.build();

        Self {
            index: std::sync::Mutex::new(None),
            writer: std::sync::Mutex::new(None),
            reader: RwLock::new(None),
            schema,
            path_field,
            title_field,
            body_field,
            tags_field,
            mtime_field,
        }
    }

    /// 打开或创建搜索索引
    ///
    /// 在指定目录下打开已有的索引，或创建新索引。
    /// 每个 Vault 有独立的索引目录（`{index_dir}/{vault_id}/`）。
    pub fn open_or_create(&self, index_dir: &Path, vault_id: &str) -> Result<()> {
        let vault_index_dir = index_dir.join(vault_id);

        let index = if vault_index_dir.exists() {
            Index::open_in_dir(&vault_index_dir)?
        } else {
            fs::create_dir_all(&vault_index_dir)?;
            Index::create_in_dir(&vault_index_dir, self.schema.clone())?
        };

        *self.index.lock().unwrap() = Some(index.clone());
        *self.writer.lock().unwrap() = Some(index.writer(50_000_000)?);
        *self.reader.write().unwrap() = Some(index.reader()?);

        Ok(())
    }

    /// 关闭搜索索引
    ///
    /// 提交所有待写入的文档，然后清理资源。
    pub fn close(&self) {
        if let Some(mut writer) = self.writer.lock().unwrap().take() {
            let _ = writer.commit();
        }
        *self.index.lock().unwrap() = None;
        *self.writer.lock().unwrap() = None;
        *self.reader.write().unwrap() = None;
    }

    /// 从 Vault 构建索引
    ///
    /// 遍历 Vault 目录下所有 `.md` 文件，重建完整搜索索引。
    /// 先删除所有旧文档，然后逐个添加新文档。
    pub fn build_from_vault(&self, vault_path: &Path) -> Result<()> {
        let mut writer_guard = self.writer.lock().map_err(|e| AppError::SearchError(e.to_string()))?;
        if let Some(ref mut w) = *writer_guard {
            w.delete_all_documents()?;

            for entry in walkdir::WalkDir::new(vault_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                let path = entry.path();
                let rel_path = path.strip_prefix(vault_path)?.to_string_lossy().to_string();
                let content = fs::read_to_string(path)?;
                let title = extract_title(&content, &rel_path);
                let tags = extract_tags(&content);
                let mtime = entry.metadata()?.modified()?.elapsed()?.as_millis() as i64;

                let doc = doc!(
                    self.path_field => rel_path.clone(),
                    self.title_field => title,
                    self.body_field => content.clone(),
                    self.tags_field => tags.join(" "),
                    self.mtime_field => mtime,
                );

                w.add_document(doc)?;
            }

            w.commit()?;
        }

        if let Some(ref mut reader) = *self.reader.write().map_err(|e| AppError::SearchError(e.to_string()))? {
            reader.reload()?;
        }

        Ok(())
    }

    /// 更新或插入笔记到索引
    ///
    /// 先删除旧文档（通过 path 字段匹配），然后添加新文档。
    pub fn upsert_note(&self, rel_path: &str, content: &str, tags: &[String], mtime: i64) -> Result<()> {
        let mut writer_guard = self.writer.lock().map_err(|e| AppError::SearchError(e.to_string()))?;
        if let Some(ref mut w) = *writer_guard {
            let term = Term::from_field_text(self.path_field, rel_path);
            w.delete_term(term);

            let title = extract_title(content, rel_path);

            let doc = doc!(
                self.path_field => rel_path,
                self.title_field => title,
                self.body_field => content,
                self.tags_field => tags.join(" "),
                self.mtime_field => mtime,
            );

            w.add_document(doc)?;
            w.commit()?;
        }

        if let Some(ref mut reader) = *self.reader.write().map_err(|e| AppError::SearchError(e.to_string()))? {
            reader.reload()?;
        }

        Ok(())
    }

    /// 从索引中移除笔记
    ///
    /// 通过 path 字段删除对应的文档。
    pub fn remove_note(&self, rel_path: &str) -> Result<()> {
        let mut writer = self.writer.lock().map_err(|e| AppError::SearchError(e.to_string()))?;
        if let Some(ref mut w) = *writer {
            let term = Term::from_field_text(self.path_field, rel_path);
            w.delete_term(term);
            let _ = w.commit();
        }

        let mut reader = self.reader.write().map_err(|e| AppError::SearchError(e.to_string()))?;
        if let Some(ref mut r) = *reader {
            let _ = r.reload();
        }

        Ok(())
    }

    /// 执行搜索
    ///
    /// 支持三种搜索模式：
    /// - Fulltext: 在标题和正文中搜索
    /// - Tag: 在标签字段中搜索
    /// - Link: 在正文中搜索 wikilink 格式
    pub fn search(&self, query: &str, mode: SearchMode, limit: usize) -> Result<Vec<SearchResult>> {
        let reader_guard = self.reader.read().map_err(|e| AppError::SearchError(e.to_string()))?;
        let reader = reader_guard.as_ref().ok_or(AppError::SearchError("index not open".to_string()))?;
        let searcher = reader.searcher();

        let mut results = Vec::new();

        let query: Box<dyn Query> = match mode {
            SearchMode::Fulltext => {
                let qp = QueryParser::for_index(&searcher.index(), vec![self.title_field, self.body_field]);
                qp.parse_query(query)?
            }
            SearchMode::Tag => {
                let term = Term::from_field_text(self.tags_field, query);
                Box::new(TermQuery::new(term, IndexRecordOption::Basic))
            }
            SearchMode::Link => {
                let qp = QueryParser::for_index(&searcher.index(), vec![self.body_field]);
                qp.parse_query(&format!("[[{}]]", query))?
            }
        };

        let hits = searcher.search(&query, &TopDocs::with_limit(limit))?;

        for (score, doc_addr) in hits {
            let doc = searcher.doc::<TantivyDocument>(doc_addr)?;

            let path = doc.get_first(self.path_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = doc.get_first(self.title_field).and_then(|v| v.as_str()).unwrap_or(&path).to_string();
            let snippet = self.extract_snippet(&doc, &*query);

            results.push(SearchResult {
                score: score as f32,
                path,
                title,
                snippet,
            });
        }

        Ok(results)
    }

    /// 提取搜索结果的高亮片段
    ///
    /// 从正文中提取包含搜索关键词的上下文片段，最多 100 个字符。
    fn extract_snippet(&self, doc: &TantivyDocument, query: &dyn Query) -> String {
        if let Some(body_val) = doc.get_first(self.body_field) {
            if let Some(body) = body_val.as_str() {
                let query_str = format!("{:?}", query);
                let lower_body = body.to_lowercase();
                let lower_query = query_str.to_lowercase();

                let search_terms: Vec<&str> = query_str.split_whitespace().filter(|s| s.len() > 2).collect();

                for term in search_terms {
                    let lower_term = term.to_lowercase();
                    if let Some(start) = lower_body.find(&lower_term) {
                        let snippet_start = start.saturating_sub(50);
                        let snippet_end = (start + term.len() + 50).min(body.len());
                        let mut snippet = body[snippet_start..snippet_end].to_string();
                        if snippet_start > 0 {
                            snippet.insert(0, '…');
                        }
                        if snippet_end < body.len() {
                            snippet.push('…');
                        }
                        return snippet;
                    }
                }

                if let Some(start) = lower_body.find(&lower_query) {
                    let snippet_start = start.saturating_sub(50);
                    let snippet_end = (start + query_str.len() + 50).min(body.len());
                    let mut snippet = body[snippet_start..snippet_end].to_string();
                    if snippet_start > 0 {
                        snippet.insert(0, '…');
                    }
                    if snippet_end < body.len() {
                        snippet.push('…');
                    }
                    return snippet;
                }
            }
        }
        String::new()
    }
}

/// 搜索模式枚举
///
/// 定义三种搜索模式：全文搜索、标签搜索、链接搜索。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// 全文搜索（在标题和正文中搜索）
    Fulltext,
    /// 标签搜索（在标签字段中搜索）
    Tag,
    /// 链接搜索（搜索 wikilink 格式）
    Link,
}

/// 搜索结果结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    /// 搜索得分
    pub score: f32,
    /// 笔记路径
    pub path: String,
    /// 笔记标题
    pub title: String,
    /// 高亮片段
    pub snippet: String,
}