//! 插件 API 绑定模块
//!
//! 桥接前端 Vue 与 Rust 插件系统，提供统一的插件 API。
//! 复刻 Obsidian 的 App、Workspace、Editor 等接口。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::plugin_runtime::PluginManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPosition {
    pub line: usize,
    pub ch: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSelection {
    pub start: EditorPosition,
    pub end: EditorPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub ext: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    pub source: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarPanelConfig {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub render: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub id: String,
    pub name: String,
    pub callback: String,
    pub hotkeys: Option<Vec<HotkeyConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub key: String,
    pub ctrl: Option<bool>,
    pub alt: Option<bool>,
    pub shift: Option<bool>,
    pub meta: Option<bool>,
}

pub struct AppApi {
    vault_path: PathBuf,
    plugin_manager: Arc<Mutex<PluginManager>>,
}

impl AppApi {
    pub fn new(vault_path: PathBuf, plugin_manager: Arc<Mutex<PluginManager>>) -> Self {
        Self {
            vault_path,
            plugin_manager,
        }
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub async fn read_file(&self, path: &str) -> Result<String> {
        let full_path = self.vault_path.join(path);
        Ok(std::fs::read_to_string(full_path)?)
    }

    pub async fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        std::fs::write(full_path, content)?;
        Ok(())
    }

    pub async fn list_files(&self, dir: &str) -> Result<Vec<FileInfo>> {
        let full_path = self.vault_path.join(dir);
        let mut files = Vec::new();

        for entry in std::fs::read_dir(full_path)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;

            if meta.is_file() {
                files.push(FileInfo {
                    path: path.strip_prefix(&self.vault_path)?.to_string_lossy().to_string(),
                    name: path.file_name().unwrap().to_string_lossy().to_string(),
                    ext: path.extension().unwrap_or_default().to_string_lossy().to_string(),
                    size: meta.len(),
                    mtime: meta.modified()?.elapsed()?.as_millis() as u64,
                });
            }
        }

        Ok(files)
    }

    pub async fn create_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full_path, content)?;
        Ok(())
    }

    pub async fn delete_file(&self, path: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        std::fs::remove_file(full_path)?;
        Ok(())
    }

    pub async fn create_folder(&self, path: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        std::fs::create_dir_all(full_path)?;
        Ok(())
    }

    pub async fn delete_folder(&self, path: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        std::fs::remove_dir_all(full_path)?;
        Ok(())
    }

    pub async fn search(&self, query: &str, mode: &str, limit: usize) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }

    pub async fn get_backlinks(&self, path: &str) -> Result<Vec<Backlink>> {
        Ok(Vec::new())
    }

    pub async fn get_tags(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    pub async fn resolve_wikilink(&self, target: &str) -> Result<Option<String>> {
        Ok(None)
    }

    pub async fn register_sidebar_panel(&self, config: SidebarPanelConfig) -> Result<()> {
        Ok(())
    }

    pub async fn register_command(&self, config: CommandConfig) -> Result<()> {
        Ok(())
    }

    pub async fn read_plugin_data(&self, plugin_id: &str) -> Result<String> {
        let manager = self.plugin_manager.lock().unwrap();
        manager.read_plugin_data(plugin_id)
    }

    pub async fn write_plugin_data(&self, plugin_id: &str, data: &str) -> Result<()> {
        let manager = self.plugin_manager.lock().unwrap();
        manager.write_plugin_data(plugin_id, data)
    }

    pub async fn show_notification(&self, title: &str, message: &str) -> Result<()> {
        Ok(())
    }

    pub async fn open_url(&self, url: &str) -> Result<()> {
        Ok(())
    }
}

pub struct WorkspaceApi {
    app_api: Arc<AppApi>,
}

impl WorkspaceApi {
    pub fn new(app_api: Arc<AppApi>) -> Self {
        Self { app_api }
    }

    pub async fn open_file(&self, path: &str) -> Result<()> {
        Ok(())
    }

    pub async fn close_file(&self, path: &str) -> Result<()> {
        Ok(())
    }

    pub async fn get_active_file(&self) -> Result<Option<String>> {
        Ok(None)
    }

    pub async fn register_sidebar_panel(&self, config: SidebarPanelConfig) -> Result<()> {
        self.app_api.register_sidebar_panel(config).await
    }

    pub async fn open_sidebar_panel(&self, panel_id: &str) -> Result<()> {
        Ok(())
    }

    pub async fn create_popover(&self, options: PopoverOptions) -> Result<String> {
        Ok("popover-id".to_string())
    }

    pub async fn create_modal(&self, options: ModalOptions) -> Result<String> {
        Ok("modal-id".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopoverOptions {
    pub title: Option<String>,
    pub content: String,
    pub target: Option<String>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalOptions {
    pub title: Option<String>,
    pub content: String,
    pub width: Option<String>,
    pub confirm_text: Option<String>,
    pub cancel_text: Option<String>,
}

pub struct EditorApi {
    app_api: Arc<AppApi>,
}

impl EditorApi {
    pub fn new(app_api: Arc<AppApi>) -> Self {
        Self { app_api }
    }

    pub async fn get_selection(&self) -> Result<Option<EditorSelection>> {
        Ok(None)
    }

    pub async fn set_selection(&self, selection: EditorSelection) -> Result<()> {
        Ok(())
    }

    pub async fn get_cursor(&self) -> Result<Option<EditorPosition>> {
        Ok(None)
    }

    pub async fn set_cursor(&self, position: EditorPosition) -> Result<()> {
        Ok(())
    }

    pub async fn get_content(&self) -> Result<String> {
        Ok(String::new())
    }

    pub async fn set_content(&self, content: &str) -> Result<()> {
        Ok(())
    }

    pub async fn replace_range(&self, from: EditorPosition, to: EditorPosition, text: &str) -> Result<()> {
        Ok(())
    }

    pub async fn get_active_file(&self) -> Result<Option<String>> {
        Ok(None)
    }
}
