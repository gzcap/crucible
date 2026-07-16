//! Tauri 命令定义
//!
//! 定义所有前端可通过 `invoke` 调用的 Rust 命令。
//! 每个命令都使用 `#[tauri::command]` 属性标记，并接收 `State<'_, AppState>` 参数以访问全局状态。
//!
//! ## 命令分类
//!
//! ### Vault 管理命令
//! - `open_vault`: 打开指定路径的笔记库
//! - `add_vault_by_path`: 通过路径添加笔记库（不打开）
//! - `list_vaults`: 获取所有已注册的笔记库列表
//! - `switch_vault`: 切换到指定的笔记库
//! - `close_vault`: 关闭当前笔记库
//! - `remove_vault`: 从注册表中移除笔记库
//! - `get_current_vault`: 获取当前打开的笔记库信息
//! - `get_watcher_status`: 获取文件监控器状态
//!
//! ### 笔记与索引命令
//! - `list_all_notes`: 获取所有笔记的元数据列表
//! - `get_note_meta`: 获取单篇笔记的元数据
//! - `get_backlinks`: 获取笔记的反向链接（引用该笔记的其他笔记）
//! - `get_outlinks`: 获取笔记的正向链接（该笔记引用的其他笔记）
//! - `get_unresolved_links`: 获取未解析的链接（指向不存在笔记的链接）
//! - `get_all_tags`: 获取所有标签
//! - `resolve_wikilink`: 解析 wikilink 目标路径
//! - `rename_note`: 重命名笔记（自动修复相关链接）
//! - `delete_note`: 删除笔记
//! - `delete_folder`: 删除文件夹
//! - `create_note`: 创建新笔记
//! - `create_folder`: 创建新文件夹
//! - `reindex_all`: 重新构建所有索引
//!
//! ### 搜索与图谱命令
//! - `search`: 执行搜索（支持全文、标签、链接三种模式）
//! - `get_graph`: 获取图谱数据（节点和边）

use std::fs;
use std::path::PathBuf;

use tauri::{Emitter, State};
use tauri_plugin_fs::FsExt;

use crate::error::{AppError, Result};
use crate::index::graph::{Backlink, GraphData};
use crate::index::search::{SearchMode, SearchResult};
use crate::parser::{extract_tags, LinkRef, NoteMeta};
use crate::plugin_runtime::{PluginManager, PluginManifest};
use crate::security::PermissionManager;
use crate::state::AppState;
use crate::vault::{generate_vault_id, VaultInfo};
use crate::watcher::FileWatcher;

// ========== 数据结构定义 ==========

/// 创建笔记的返回结果
#[derive(serde::Serialize)]
pub struct CreateNoteResult {
    pub path: String,
    pub title: String,
}

/// 创建文件夹的返回结果
#[derive(serde::Serialize)]
pub struct CreateFolderResult {
    pub path: String,
    pub name: String,
}

/// 标签出现位置信息
#[derive(serde::Serialize)]
pub struct TagOccurrence {
    pub path: String,
    pub title: String,
    pub line: usize,
    pub snippet: String,
}

/// 标签详情（包含出现次数和位置列表）
#[derive(serde::Serialize)]
pub struct TagDetail {
    pub name: String,
    pub count: usize,
    pub occurrences: Vec<TagOccurrence>,
}

// ========== Vault 管理命令 ==========

/// 打开指定路径的笔记库
///
/// 执行以下操作：
/// 1. 验证路径有效性
/// 2. 在注册表中查找或创建 Vault 记录
/// 3. 更新最后打开时间
/// 4. 允许文件系统访问该目录
/// 5. 启动文件监控器
/// 6. 初始化搜索索引
/// 7. 重建链接索引
/// 8. 触发 `roc://vault-opened` 事件
#[tauri::command]
pub fn open_vault(state: State<'_, AppState>, path: String) -> Result<VaultInfo> {
    let vault_path = PathBuf::from(&path);
    if !vault_path.exists() || !vault_path.is_dir() {
        return Err(AppError::InvalidPath(path));
    }

    let vaults_path = state.get_vaults_path();

    let mut registry = state.vault_registry.write();
    let existing_id = registry.find_id_by_path(&vault_path);

    let (vault_id, mut vault_info) = if let Some(id) = existing_id {
        let mut info = registry.get(&id).cloned().ok_or(AppError::VaultNotFound(id.clone()))?;
        info.last_opened = Some(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap());
        (id, info)
    } else {
        let id = generate_vault_id(&vault_path);
        let name = vault_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Vault")
            .to_string();
        let info = VaultInfo {
            id: id.clone(),
            name,
            path: vault_path.clone(),
            last_opened: Some(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap()),
        };
        registry.add(info.clone())?;
        (id, info)
    };

    registry.set_last_used(&vault_id);
    registry.save_to_file(&vaults_path)?;

    let mut current_vault = state.current_vault.write();
    *current_vault = Some(vault_info.clone());

    let app_handle = state.app_handle.clone();
    let fs_scope = app_handle.fs_scope();
    let _ = fs_scope.allow_directory(&vault_path, true);

    {
        let mut watcher = state.watcher.lock();
        *watcher = None;
    }

    let mut watcher = FileWatcher::new(app_handle.clone(), &vault_path)?;
    watcher.start()?;

    {
        let mut state_watcher = state.watcher.lock();
        *state_watcher = Some(watcher.debouncer);
    }

    let index_dir = state.get_index_dir();
    state.search_index.open_or_create(&index_dir, &vault_id)?;

    state.link_index.rebuild_from_vault(&vault_path)?;
    state.search_index.build_from_vault(&vault_path)?;

    state.init_plugin_manager(vault_path.clone());

    let _ = app_handle.emit("roc://vault-opened", &vault_info);

    Ok(vault_info)
}

/// 通过路径添加笔记库（不打开）
///
/// 将指定路径注册到 Vault 注册表中，但不执行打开操作。
#[tauri::command]
pub fn add_vault_by_path(state: State<'_, AppState>, path: String) -> Result<VaultInfo> {
    let vault_path = PathBuf::from(&path);
    if !vault_path.exists() || !vault_path.is_dir() {
        return Err(AppError::InvalidPath(path));
    }

    let vaults_path = state.get_vaults_path();
    let mut registry = state.vault_registry.write();

    if registry.exists_by_path(&vault_path) {
        return Err(AppError::VaultAlreadyExists(path));
    }

    let id = generate_vault_id(&vault_path);
    let name = vault_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Vault")
        .to_string();

    let vault_info = VaultInfo {
        id: id.clone(),
        name,
        path: vault_path,
        last_opened: None,
    };

    registry.add(vault_info.clone())?;
    registry.save_to_file(&vaults_path)?;

    Ok(vault_info)
}

/// 获取所有已注册的笔记库列表
///
/// 返回按最后打开时间降序排序的 Vault 列表。
#[tauri::command]
pub fn list_vaults(state: State<'_, AppState>) -> Result<Vec<VaultInfo>> {
    Ok(state.vault_registry.read().list())
}

/// 切换到指定的笔记库
///
/// 通过 Vault ID 切换到已注册的笔记库，内部调用 `open_vault`。
#[tauri::command]
pub fn switch_vault(state: State<'_, AppState>, vault_id: String) -> Result<VaultInfo> {
    let vault_info = state
        .vault_registry
        .read()
        .get(&vault_id)
        .cloned()
        .ok_or(AppError::VaultNotFound(vault_id))?;

    open_vault(state, vault_info.path.to_string_lossy().to_string())
}

/// 关闭当前笔记库
///
/// 清理文件监控器和搜索索引，触发 `roc://vault-closed` 事件。
#[tauri::command]
pub fn close_vault(state: State<'_, AppState>) -> Result<()> {
    {
        let mut current_vault = state.current_vault.write();
        *current_vault = None;
    }

    {
        let mut watcher = state.watcher.lock();
        *watcher = None;
    }

    state.search_index.close();

    {
        let mut pm = state.plugin_manager.lock();
        if let Some(ref mut manager) = *pm {
            manager.unload_all_plugins();
        }
        *pm = None;
    }

    let _ = state.app_handle.emit("roc://vault-closed", ());

    Ok(())
}

/// 从注册表中移除笔记库
///
/// 删除 Vault 记录并清理相关索引目录。
#[tauri::command]
pub fn remove_vault(state: State<'_, AppState>, vault_id: String) -> Result<VaultInfo> {
    let vaults_path = state.get_vaults_path();
    let mut registry = state.vault_registry.write();

    let vault_info = registry.remove(&vault_id)?;
    registry.save_to_file(&vaults_path)?;

    let index_dir = state.get_index_dir().join(&vault_id);
    if index_dir.exists() {
        let _ = fs::remove_dir_all(&index_dir);
    }

    Ok(vault_info)
}

/// 获取当前打开的笔记库信息
#[tauri::command]
pub fn get_current_vault(state: State<'_, AppState>) -> Result<Option<VaultInfo>> {
    Ok(state.current_vault.read().clone())
}

/// 获取文件监控器状态
///
/// 返回 `true` 表示监控器正在运行，`false` 表示未运行。
#[tauri::command]
pub fn get_watcher_status(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.watcher.lock().is_some())
}

// ========== 笔记与索引命令 ==========

/// 获取所有笔记的元数据列表
#[tauri::command]
pub fn list_all_notes(state: State<'_, AppState>) -> Result<Vec<NoteMeta>> {
    Ok(state.link_index.list_all_notes())
}

/// 获取单篇笔记的元数据
#[tauri::command]
pub fn get_note_meta(state: State<'_, AppState>, path: String) -> Result<Option<NoteMeta>> {
    Ok(state.link_index.get_note_meta(&path))
}

/// 获取笔记的反向链接
///
/// 返回引用该笔记的其他笔记列表，包含来源路径、标题和上下文片段。
#[tauri::command]
pub fn get_backlinks(state: State<'_, AppState>, path: String) -> Result<Vec<Backlink>> {
    let mut backlinks = state.link_index.get_backlinks(&path);

    // 增强 snippet：从源文件中提取包含 wikilink 的上下文行
    if let Some(vault) = state.current_vault.read().as_ref() {
        let target_pathbuf = PathBuf::from(&path);
        let target_stem = target_pathbuf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for backlink in backlinks.iter_mut() {
            let abs_path = vault.path.join(&backlink.source);
            if let Ok(content) = fs::read_to_string(&abs_path) {
                for line in content.lines() {
                    if line.contains(&format!("[[{}", target_stem)) {
                        let snippet = line.trim();
                        backlink.snippet = if snippet.chars().count() > 200 {
                            let truncated: String = snippet.chars().take(200).collect();
                            format!("{}…", truncated)
                        } else {
                            snippet.to_string()
                        };
                        break;
                    }
                }
            }
        }
    }

    Ok(backlinks)
}

/// 获取笔记的正向链接
///
/// 返回该笔记引用的其他笔记列表。
#[tauri::command]
pub fn get_outlinks(state: State<'_, AppState>, path: String) -> Result<Vec<LinkRef>> {
    Ok(state.link_index.get_outlinks(&path))
}

/// 获取未解析的链接
///
/// 返回指向不存在笔记的链接列表。
#[tauri::command]
pub fn get_unresolved_links(state: State<'_, AppState>, path: String) -> Result<Vec<LinkRef>> {
    Ok(state.link_index.get_unresolved_links(&path))
}

/// 获取所有标签
///
/// 返回去重后的标签列表，按字母顺序排序。
#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>> {
    Ok(state.link_index.get_all_tags())
}

/// 获取标签详情（包含出现位置）
#[tauri::command]
pub fn get_tag_details(state: State<'_, AppState>, tag: String) -> Result<TagDetail> {
    let mut occurrences = Vec::new();

    if let Some(vault) = state.current_vault.read().as_ref() {
        for note in state.link_index.notes.iter() {
            if note.tags.contains(&tag) {
                let abs_path = vault.path.join(&note.path);
                if let Ok(content) = fs::read_to_string(&abs_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains(&format!("#{}", tag)) {
                            let snippet = if line.chars().count() > 100 {
                                let truncated: String = line.chars().take(100).collect();
                                format!("{}…", truncated)
                            } else {
                                line.to_string()
                            };
                            occurrences.push(TagOccurrence {
                                path: note.path.clone(),
                                title: note.title.clone(),
                                line: line_num + 1,
                                snippet,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(TagDetail {
        name: tag,
        count: occurrences.len(),
        occurrences,
    })
}

/// 解析 wikilink 目标路径
///
/// 根据 wikilink 目标名称查找对应的笔记路径。
#[tauri::command]
pub fn resolve_wikilink(state: State<'_, AppState>, target: String) -> Result<Option<String>> {
    Ok(state.link_index.resolve_wikilink(&target))
}

/// 重命名笔记
///
/// 执行以下操作：
/// 1. 重命名文件
/// 2. 更新链接索引（自动修复所有引用该笔记的 wikilink）
/// 3. 更新搜索索引
/// 4. 触发 `roc://file-changed` 事件
#[tauri::command]
pub fn rename_note(
    state: State<'_, AppState>,
    old_path: String,
    new_path: String,
) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let abs_old = vault_path.join(&old_path);
    let abs_new = vault_path.join(&new_path);

    fs::rename(&abs_old, &abs_new)?;

    let modified_files = state.link_index.on_rename(&old_path, &new_path, &vault_path)?;

    let content = fs::read_to_string(&abs_new)?;
    let tags = extract_tags(&content);
    let meta = fs::metadata(&abs_new)?;
    let mtime = meta.modified()?.elapsed()?.as_millis() as i64;

    state.search_index.upsert_note(&new_path, &content, &tags, mtime)?;
    state.search_index.remove_note(&old_path)?;

    // 为每个被修改的引用源 emit modify 事件
    for modified_path in modified_files {
        let modified_content = fs::read_to_string(vault_path.join(&modified_path)).unwrap_or_default();
        let modified_tags = extract_tags(&modified_content);
        let modified_mtime = fs::metadata(vault_path.join(&modified_path))
            .ok()
            .and_then(|meta| meta.modified().ok())
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        state.search_index.upsert_note(&modified_path, &modified_content, &modified_tags, modified_mtime)?;

        let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
            kind: "modify".to_string(),
            path: modified_path,
            new_path: None,
        });
    }

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "rename".to_string(),
        path: old_path,
        new_path: Some(new_path),
    });

    Ok(())
}

/// 删除笔记
///
/// 从文件系统删除文件，并从索引中移除相关数据。
#[tauri::command]
pub fn delete_note(state: State<'_, AppState>, path: String) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let abs_path = vault_path.join(&path);
    if abs_path.exists() {
        fs::remove_file(&abs_path)?;
    }

    // 索引更新和事件发送由文件监控器处理，避免重复操作导致卡死
    Ok(())
}

/// 删除文件夹
///
/// 删除文件夹及其所有内容，并重建索引。
#[tauri::command]
pub fn delete_folder(state: State<'_, AppState>, path: String) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let abs_path = vault_path.join(&path);
    if abs_path.exists() {
        fs::remove_dir_all(&abs_path)?;
    }

    // 文件夹删除后，监控器会为每个 .md 文件触发删除事件
    // 这里发送一个事件通知前端文件夹被删除，前端会调用 loadNotes() 重新加载
    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "delete".to_string(),
        path: path.clone(),
        new_path: None,
    });

    Ok(())
}

/// 创建新笔记
///
/// 创建一个新的 Markdown 文件，命名为"未命名文件"或"未命名文件N"（避免重复）。
#[tauri::command]
pub fn create_note(state: State<'_, AppState>) -> Result<CreateNoteResult> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let mut name = "未命名文件".to_string();
    let mut counter = 2;

    while vault_path.join(format!("{}.md", name)).exists() {
        name = format!("未命名文件{}", counter);
        counter += 1;
    }

    let title = format!("{}.md", name);
    let path = title.clone();
    let abs_path = vault_path.join(&path);

    fs::write(&abs_path, "# New Note\n\n")?;

    let tags = vec![];
    let meta = fs::metadata(&abs_path)?;
    let mtime = meta.modified()?.elapsed()?.as_millis() as i64;

    state.link_index.upsert_note(&path, "# New Note\n\n", mtime, 0)?;
    state.link_index.rebuild_backrefs();
    state.search_index.upsert_note(&path, "# New Note\n\n", &tags, mtime)?;

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "create".to_string(),
        path: path.clone(),
        new_path: None,
    });

    Ok(CreateNoteResult { path, title })
}

/// 创建新文件夹
///
/// 创建一个新文件夹，命名为"未命名文件夹"或"未命名文件夹N"（避免重复）。
#[tauri::command]
pub fn create_folder(state: State<'_, AppState>) -> Result<CreateFolderResult> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let mut name = "未命名文件夹".to_string();
    let mut counter = 2;

    while vault_path.join(&name).exists() {
        name = format!("未命名文件夹{}", counter);
        counter += 1;
    }

    let path = name.clone();
    let abs_path = vault_path.join(&path);
    fs::create_dir_all(&abs_path)?;

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "create".to_string(),
        path: path.clone(),
        new_path: None,
    });

    Ok(CreateFolderResult { path, name })
}

// ========== 搜索与图谱命令 ==========

/// 执行搜索
///
/// 支持三种搜索模式：
/// - `fulltext`: 全文搜索（默认）
/// - `tag`: 标签搜索
/// - `link`: 链接搜索
#[tauri::command]
pub fn search(
    state: State<'_, AppState>,
    query: String,
    mode: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>> {
    let search_mode = match mode.as_str() {
        "tag" => SearchMode::Tag,
        "link" => SearchMode::Link,
        _ => SearchMode::Fulltext,
    };

    state.search_index.search(&query, search_mode, limit.unwrap_or(20))
}

/// 重新构建所有索引
///
/// 重建链接索引和搜索索引。
#[tauri::command]
pub fn reindex_all(state: State<'_, AppState>) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    state.link_index.rebuild_from_vault(&vault_path)?;
    state.search_index.build_from_vault(&vault_path)?;

    Ok(())
}

/// 获取图谱数据
///
/// 返回笔记间的链接关系图，包含节点（笔记）和边（链接）。
#[tauri::command]
pub fn get_graph(state: State<'_, AppState>) -> Result<GraphData> {
    Ok(state.link_index.export_graph())
}

// ========== 插件管理命令 ==========

/// 读取插件配置
///
/// 从 `{vault}/.roc/plugins/{plugin-id}/data.json` 读取插件的持久化配置。
#[tauri::command]
pub fn read_plugin_config(state: State<'_, AppState>, plugin_id: String) -> Result<String> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let plugin_dir = vault_path.join(".roc").join("plugins").join(&plugin_id);
    let data_path = plugin_dir.join("data.json");

    if data_path.exists() {
        Ok(fs::read_to_string(data_path)?)
    } else {
        Ok("{}".to_string())
    }
}

/// 写入插件配置
///
/// 将插件配置持久化到 `{vault}/.roc/plugins/{plugin-id}/data.json`。
#[tauri::command]
pub fn write_plugin_config(state: State<'_, AppState>, plugin_id: String, data: String) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let plugin_dir = vault_path.join(".roc").join("plugins").join(&plugin_id);
    fs::create_dir_all(&plugin_dir)?;

    let data_path = plugin_dir.join("data.json");
    fs::write(data_path, data)?;

    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub repo: Option<String>,
    pub min_app_version: Option<String>,
    pub is_desktop_only: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub enabled: bool,
}

/// 获取已安装的插件列表
///
/// 扫描 `{vault}/.roc/plugins/` 目录，读取每个插件的 `manifest.json` 文件。
#[tauri::command]
pub fn list_installed_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let plugins_dir = vault_path.join(".roc").join("plugins");

    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugin_infos = Vec::new();

    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref();

    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                        let plugin_id = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&manifest.id)
                            .to_string();

                        let enabled = manager.map(|m| m.is_enabled(&plugin_id)).unwrap_or(false);

                        plugin_infos.push(PluginInfo {
                            id: plugin_id,
                            name: manifest.name,
                            version: manifest.version,
                            description: manifest.description,
                            author: manifest.author,
                            author_url: manifest.author_url,
                            repo: manifest.repo,
                            min_app_version: manifest.min_app_version,
                            is_desktop_only: manifest.is_desktop_only,
                            permissions: manifest.permissions,
                            enabled,
                        });
                    }
                }
            }
        }
    }

    Ok(plugin_infos)
}

#[tauri::command]
pub fn load_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.load_plugin(&plugin_id)
}

#[tauri::command]
pub fn unload_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.unload_plugin(&plugin_id)
}

#[tauri::command]
pub fn load_all_plugins(state: State<'_, AppState>) -> Result<Vec<String>> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.load_all_plugins()
}

#[tauri::command]
pub fn unload_all_plugins(state: State<'_, AppState>) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.unload_all_plugins();
    Ok(())
}

#[tauri::command]
pub fn init_plugin_system(state: State<'_, AppState>) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    state.init_plugin_manager(vault_path);

    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::InternalError("plugin manager not initialized".to_string()))?;

    manager.scan_plugins()?;

    Ok(())
}

#[tauri::command]
pub fn reload_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;

    let was_enabled = manager.is_enabled(&plugin_id);

    if was_enabled {
        manager.unload_plugin(&plugin_id)?;
    }

    manager.scan_plugins()?;

    if was_enabled {
        manager.load_plugin(&plugin_id)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_plugin_status(state: State<'_, AppState>, plugin_id: String) -> Result<bool> {
    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref().ok_or(AppError::NoVaultOpen)?;
    Ok(manager.is_enabled(&plugin_id))
}

#[tauri::command]
pub fn enable_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.load_plugin(&plugin_id)
}

#[tauri::command]
pub fn disable_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.unload_plugin(&plugin_id)
}

#[tauri::command]
pub fn get_plugin_manifest(state: State<'_, AppState>, plugin_id: String) -> Result<PluginManifest> {
    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref().ok_or(AppError::NoVaultOpen)?;
    let instance = manager.get_plugin(&plugin_id)
        .ok_or(AppError::PluginNotFound(plugin_id))?;
    Ok(instance.manifest.clone())
}

#[tauri::command]
pub fn read_plugin_main(state: State<'_, AppState>, plugin_id: String) -> Result<String> {
    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref().ok_or(AppError::NoVaultOpen)?;
    let instance = manager.get_plugin(&plugin_id)
        .ok_or(AppError::PluginNotFound(plugin_id.clone()))?;
    
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;
    
    let main_path = vault_path.join(".roc").join("plugins").join(&plugin_id).join("main.js");
    fs::read_to_string(main_path).map_err(|e| AppError::PluginLoadError(plugin_id, e.to_string()))
}

#[tauri::command]
pub fn get_plugin_data(state: State<'_, AppState>, plugin_id: String) -> Result<String> {
    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref().ok_or(AppError::NoVaultOpen)?;
    manager.read_plugin_data(&plugin_id)
}

#[tauri::command]
pub fn set_plugin_data(state: State<'_, AppState>, plugin_id: String, data: String) -> Result<()> {
    let pm = state.plugin_manager.lock();
    let manager = pm.as_ref().ok_or(AppError::NoVaultOpen)?;
    manager.write_plugin_data(&plugin_id, &data)
}

#[tauri::command]
pub fn reload_all_plugins(state: State<'_, AppState>) -> Result<()> {
    let mut pm = state.plugin_manager.lock();
    let manager = pm.as_mut().ok_or(AppError::NoVaultOpen)?;
    manager.unload_all_plugins();
    manager.scan_plugins()?;
    manager.load_all_plugins()?;
    Ok(())
}

#[tauri::command]
pub fn list_dir(state: State<'_, AppState>, dir: String) -> Result<Vec<serde_json::Value>> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        let mut obj = serde_json::Map::new();
        obj.insert("name".to_string(), serde_json::Value::String(
            entry.file_name().to_string_lossy().to_string()
        ));
        obj.insert("isDirectory".to_string(), serde_json::Value::Bool(path.is_dir()));
        obj.insert("isFile".to_string(), serde_json::Value::Bool(path.is_file()));
        obj.insert("path".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
        
        entries.push(serde_json::Value::Object(obj));
    }
    
    Ok(entries)
}

#[tauri::command]
pub fn read_text_file(state: State<'_, AppState>, file_path: String) -> Result<String> {
    Ok(fs::read_to_string(file_path)?)
}

#[tauri::command]
pub fn copy_plugin(state: State<'_, AppState>, source_path: String, target_dir: String) -> Result<()> {
    let source = PathBuf::from(&source_path);
    let target = PathBuf::from(&target_dir).join(source.file_name().unwrap());
    
    if source.is_dir() {
        fs_extra::dir::copy(&source, &target_dir, &fs_extra::dir::CopyOptions::new())?;
    } else {
        fs_extra::file::copy(&source, &target, &fs_extra::file::CopyOptions::new())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn uninstall_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<()> {
    let vault_path = state
        .current_vault
        .read()
        .as_ref()
        .map(|v| v.path.clone())
        .ok_or(AppError::NoVaultOpen)?;

    let plugin_dir = vault_path.join(".roc").join("plugins").join(&plugin_id);

    if !plugin_dir.exists() {
        return Err(AppError::PluginNotFound(plugin_id));
    }

    let mut pm = state.plugin_manager.lock();
    if let Some(ref mut manager) = *pm {
        let _ = manager.unload_plugin(&plugin_id);
    }

    fs_extra::dir::remove(&plugin_dir)?;

    Ok(())
}