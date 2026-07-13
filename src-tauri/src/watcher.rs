use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use notify::{EventKind, RecommendedWatcher, RecursiveMode};

use tauri::{AppHandle, Emitter, Manager};

use crate::error::Result;
use crate::state::AppState;

pub struct FileWatcher {
    pub debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    pub app_handle: Arc<AppHandle>,
    pub vault_path: PathBuf,
    pending_renames: Mutex<HashMap<u64, (PathBuf, Instant)>>,
}

impl FileWatcher {
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
            pending_renames: Mutex::new(HashMap::new()),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.debouncer
            .watch(&self.vault_path, RecursiveMode::Recursive)?;
        Ok(())
    }

    fn handle_event(app_handle: &AppHandle, vault_path: &Path, event: notify_debouncer_full::DebouncedEvent) {
        let event = event.event;

        if !event.paths.iter().any(|p| {
            p.extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
        }) {
            return;
        }

        let state = app_handle.state::<AppState>();

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        // 更新后端索引
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

                        let kind = if matches!(event.kind, EventKind::Create(_)) { "create" } else { "modify" };
                        let _ = app_handle.emit("roc://file-changed", FileChangeEvent {
                            kind: kind.to_string(),
                            path: rel_path.to_string(),
                            new_path: None,
                        });
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    if let Some(rel_path) = path.strip_prefix(vault_path).ok().and_then(|p| p.to_str()) {
                        // 从索引中移除
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub kind: String,
    pub path: String,
    pub new_path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WatcherErrorEvent {
    pub message: String,
}