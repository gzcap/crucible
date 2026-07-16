//! 文件系统监控模块
//!
//! 使用 `notify` 库监控 Vault 目录下的文件变化，支持：
//! - 文件创建/修改/删除事件
//! - 300ms 防抖（避免频繁事件）
//! - 自动更新链接索引和搜索索引
//! - 通过 Tauri 事件通知前端

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::ModifyKind;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::Result;
use crate::state::AppState;

/// 文件监控器
///
/// 使用 debouncer 机制避免频繁触发事件，仅处理 `.md` 文件的变化。
pub struct FileWatcher {
    /// Debouncer 实例，管理监控器和防抖逻辑
    pub debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    /// Tauri 应用句柄，用于发送事件
    pub app_handle: Arc<AppHandle>,
    /// 监控的 Vault 路径
    pub vault_path: PathBuf,
}

impl FileWatcher {
    /// 创建新的文件监控器
    ///
    /// # 参数
    ///
    /// - `app_handle`: Tauri 应用句柄
    /// - `vault_path`: 需要监控的目录路径
    pub fn new(app_handle: Arc<AppHandle>, vault_path: &Path) -> Result<Self> {
        let vault_path_clone = vault_path.to_path_buf();
        let app_handle_clone = app_handle.clone();
        let debouncer = new_debouncer(
            Duration::from_millis(300),
            None,
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        for event in events {
                            Self::handle_event(&app_handle_clone, &vault_path_clone, event);
                        }
                    }
                    Err(e) => {
                        let _ = app_handle_clone.emit("roc://watcher-error", WatcherErrorEvent {
                            message: format!("{:?}", e),
                        });
                    }
                }
            },
        )?;

        Ok(Self {
            debouncer,
            app_handle,
            vault_path: vault_path.to_path_buf(),
        })
    }

    /// 启动文件监控
    ///
    /// 开始递归监控指定目录下的所有文件变化。
    pub fn start(&mut self) -> Result<()> {
        self.debouncer
            .watcher()
            .watch(&self.vault_path, RecursiveMode::Recursive)?;
        Ok(())
    }

    /// 计算文件内容的 hash 值（用于检测重命名）
    fn hash_content(content: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// 处理文件系统事件
    ///
    /// 根据事件类型执行相应操作：
    /// - 创建：检查是否为重命名（通过 content hash 匹配 pending_deletes），否则正常创建
    /// - 修改：更新索引并通知前端
    /// - 删除：记录到 pending_deletes（用于检测重命名），稍后清理
    /// - 其他：忽略
    fn handle_event(app_handle: &AppHandle, vault_path: &Path, event: notify_debouncer_full::DebouncedEvent) {
        let event = event.event;

        // 只处理 Markdown 文件
        if !event.paths.iter().any(|p| {
            p.extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
        }) {
            return;
        }

        let state = app_handle.state::<AppState>();

        match event.kind {
            EventKind::Create(_) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        let content = std::fs::read_to_string(path).unwrap_or_default();
                        let content_hash = Self::hash_content(&content);

                        let mut pending_deletes = state.pending_deletes.lock();

                        // 检查是否存在匹配的删除事件（content hash 相同且时间间隔 < 1秒）
                        if let Some((old_path, delete_time)) = pending_deletes.remove(&content_hash.to_string()) {
                            if delete_time.elapsed().as_millis() < 1000 {
                                // 视为重命名，触发完整的 on_rename 流程
                                let modified_files = state.link_index.on_rename(&old_path, rel_path, vault_path);
                                if let Ok(files) = modified_files {
                                    for modified_path in files {
                                        let modified_content = std::fs::read_to_string(vault_path.join(&modified_path)).unwrap_or_default();
                                        let modified_tags = crate::parser::extract_tags(&modified_content);
                                        let modified_mtime = std::fs::metadata(vault_path.join(&modified_path))
                                            .ok()
                                            .and_then(|meta| meta.modified().ok())
                                            .and_then(|t| t.elapsed().ok())
                                            .map(|d| d.as_millis() as i64)
                                            .unwrap_or(0);
                                        let _ = state.search_index.upsert_note(&modified_path, &modified_content, &modified_tags, modified_mtime);

                                        let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                                            kind: "modify".to_string(),
                                            path: modified_path,
                                            new_path: None,
                                        });
                                    }
                                }

                                let _ = state.search_index.remove_note(&old_path);
                                if let Ok(meta) = std::fs::metadata(path) {
                                    let mtime = meta.modified()
                                        .map(|t| t.elapsed().map(|d| d.as_millis() as i64).unwrap_or(0))
                                        .unwrap_or(0);
                                    let tags = crate::parser::extract_tags(&content);
                                    let _ = state.search_index.upsert_note(rel_path, &content, &tags, mtime);
                                }

                                let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                                    kind: "rename".to_string(),
                                    path: old_path,
                                    new_path: Some(rel_path.to_string()),
                                });

                                continue;
                            }
                        }

                        // 正常创建事件
                        if let Ok(meta) = std::fs::metadata(path) {
                            let mtime = meta.modified()
                                .map(|t| t.elapsed().map(|d| d.as_millis() as i64).unwrap_or(0))
                                .unwrap_or(0);
                            let size = meta.len();
                            let tags = crate::parser::extract_tags(&content);
                            let _ = state.link_index.upsert_note(rel_path, &content, mtime, size);
                            state.link_index.rebuild_backrefs();
                            let _ = state.search_index.upsert_note(rel_path, &content, &tags, mtime);
                        }

                        let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                            kind: "create".to_string(),
                            path: rel_path.to_string(),
                            new_path: None,
                        });
                    }
                }
            }
            EventKind::Modify(ModifyKind::Name(_)) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        if path.exists() {
                            if let Ok(content) = std::fs::read_to_string(path) {
                                if let Ok(meta) = std::fs::metadata(path) {
                                    let mtime = meta.modified()
                                        .map(|t| t.elapsed().map(|d| d.as_millis() as i64).unwrap_or(0))
                                        .unwrap_or(0);
                                    let size = meta.len();
                                    let tags = crate::parser::extract_tags(&content);
                                    let _ = state.link_index.upsert_note(rel_path, &content, mtime, size);
                                    state.link_index.rebuild_backrefs();
                                    let _ = state.search_index.upsert_note(rel_path, &content, &tags, mtime);
                                }
                            }

                            let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                                kind: "create".to_string(),
                                path: rel_path.to_string(),
                                new_path: None,
                            });
                        } else {
                            let _ = state.link_index.remove_note(rel_path);
                            let _ = state.search_index.remove_note(rel_path);

                            let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                                kind: "delete".to_string(),
                                path: rel_path.to_string(),
                                new_path: None,
                            });
                        }
                    }
                }
            }
            EventKind::Modify(_) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if let Ok(meta) = std::fs::metadata(path) {
                                let mtime = meta.modified()
                                    .map(|t| t.elapsed().map(|d| d.as_millis() as i64).unwrap_or(0))
                                    .unwrap_or(0);
                                let size = meta.len();
                                let tags = crate::parser::extract_tags(&content);
                                let _ = state.link_index.upsert_note(rel_path, &content, mtime, size);
                                state.link_index.rebuild_backrefs();
                                let _ = state.search_index.upsert_note(rel_path, &content, &tags, mtime);
                            }
                        }

                        let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                            kind: "modify".to_string(),
                            path: rel_path.to_string(),
                            new_path: None,
                        });
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        // 读取删除前的内容用于 hash 匹配（重命名检测）
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let content_hash = Self::hash_content(&content);
                            let mut pending_deletes = state.pending_deletes.lock();
                            pending_deletes.insert(content_hash.to_string(), (rel_path.to_string(), Instant::now()));

                            // 清理过期的 pending_deletes（超过 1 秒）
                            pending_deletes.retain(|_, (_, time)| time.elapsed().as_millis() < 1000);
                        }

                        let _ = state.link_index.remove_note(rel_path);
                        let _ = state.search_index.remove_note(rel_path);

                        let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                            kind: "delete".to_string(),
                            path: rel_path.to_string(),
                            new_path: None,
                        });
                    }
                }
            }
            EventKind::Other | EventKind::Access(_) | EventKind::Any => {}
        }
    }
}

/// 文件变化事件
///
/// 用于通过 Tauri 事件系统通知前端文件变化。
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    /// 事件类型：`create`、`modify`、`delete`、`rename`
    pub kind: String,
    /// 变化文件的相对路径
    pub path: String,
    /// 重命名时的新路径（可选）
    pub new_path: Option<String>,
}

/// 监控器错误事件
///
/// 用于通过 Tauri 事件系统通知前端监控器错误。
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatcherErrorEvent {
    /// 错误消息
    pub message: String,
}