use std::fs;
use std::path::PathBuf;

use tauri::{Emitter, State};
use tauri_plugin_fs::FsExt;

use crate::error::{AppError, Result};
use crate::index::graph::{Backlink, GraphData};
use crate::index::search::{SearchMode, SearchResult};
use crate::parser::{extract_tags, LinkRef, NoteMeta};
use crate::state::AppState;
use crate::vault::{generate_vault_id, VaultInfo};
use crate::watcher::FileWatcher;

#[derive(serde::Serialize)]
pub struct CreateNoteResult {
    pub path: String,
    pub title: String,
}

#[derive(serde::Serialize)]
pub struct CreateFolderResult {
    pub path: String,
    pub name: String,
}

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

    let _ = app_handle.emit("roc://vault-opened", &vault_info);

    Ok(vault_info)
}

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

#[tauri::command]
pub fn list_vaults(state: State<'_, AppState>) -> Result<Vec<VaultInfo>> {
    Ok(state.vault_registry.read().list())
}

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

    let _ = state.app_handle.emit("roc://vault-closed", ());

    Ok(())
}

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

#[tauri::command]
pub fn get_current_vault(state: State<'_, AppState>) -> Result<Option<VaultInfo>> {
    Ok(state.current_vault.read().clone())
}

#[tauri::command]
pub fn get_watcher_status(state: State<'_, AppState>) -> Result<bool> {
    Ok(state.watcher.lock().is_some())
}

#[tauri::command]
pub fn list_all_notes(state: State<'_, AppState>) -> Result<Vec<NoteMeta>> {
    Ok(state.link_index.list_all_notes())
}

#[tauri::command]
pub fn get_note_meta(state: State<'_, AppState>, path: String) -> Result<Option<NoteMeta>> {
    Ok(state.link_index.get_note_meta(&path))
}

#[tauri::command]
pub fn get_backlinks(state: State<'_, AppState>, path: String) -> Result<Vec<Backlink>> {
    let mut backlinks = state.link_index.get_backlinks(&path);

    // 增强 snippet：从源文件中提取包含 wikilink 的上下文行，
    // 替代 LinkIndex 中仅返回 wikilink 文本的默认实现
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

#[tauri::command]
pub fn get_outlinks(state: State<'_, AppState>, path: String) -> Result<Vec<LinkRef>> {
    Ok(state.link_index.get_outlinks(&path))
}

#[tauri::command]
pub fn get_unresolved_links(state: State<'_, AppState>, path: String) -> Result<Vec<LinkRef>> {
    Ok(state.link_index.get_unresolved_links(&path))
}

#[tauri::command]
pub fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>> {
    Ok(state.link_index.get_all_tags())
}

#[derive(serde::Serialize)]
pub struct TagOccurrence {
    pub path: String,
    pub title: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(serde::Serialize)]
pub struct TagDetail {
    pub name: String,
    pub count: usize,
    pub occurrences: Vec<TagOccurrence>,
}

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

#[tauri::command]
pub fn resolve_wikilink(state: State<'_, AppState>, target: String) -> Result<Option<String>> {
    Ok(state.link_index.resolve_wikilink(&target))
}

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

    state.link_index.on_rename(&old_path, &new_path, &vault_path)?;

    let content = fs::read_to_string(&abs_new)?;
    let tags = extract_tags(&content);
    let meta = fs::metadata(&abs_new)?;
    let mtime = meta.modified()?.elapsed()?.as_millis() as i64;

    state.search_index.upsert_note(&new_path, &content, &tags, mtime)?;
    state.search_index.remove_note(&old_path)?;

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "rename".to_string(),
        path: old_path,
        new_path: Some(new_path),
    });

    Ok(())
}

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

    let _ = state.link_index.remove_note(&path);
    let _ = state.search_index.remove_note(&path);

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "delete".to_string(),
        path: path.clone(),
        new_path: None,
    });

    Ok(())
}

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

    let index_dir = state.get_index_dir();
    if let Some(vault_id) = state.current_vault.read().as_ref().map(|v| v.id.clone()) {
        let _ = state.link_index.rebuild_from_vault(&vault_path);
        let _ = state.search_index.open_or_create(&index_dir, &vault_id);
        let _ = state.search_index.build_from_vault(&vault_path);
    }

    let _ = state.app_handle.emit("roc://file-changed", crate::watcher::FileChangeEvent {
        kind: "delete".to_string(),
        path: path.clone(),
        new_path: None,
    });

    Ok(())
}

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

#[tauri::command]
pub fn get_graph(state: State<'_, AppState>) -> Result<GraphData> {
    Ok(state.link_index.export_graph())
}

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
