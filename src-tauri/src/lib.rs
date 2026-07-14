mod commands;
mod error;
mod index;
mod parser;
mod state;
mod vault;
mod watcher;

use std::sync::Arc;

use tauri::Manager;

use state::AppState;
use vault::VaultRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let handle = app.handle().clone();
            let state = AppState::new(app_data_dir.clone(), Arc::new(handle));

            let vaults_path = app_data_dir.join("vaults.json");
            if vaults_path.exists() {
                if let Ok(registry) = VaultRegistry::load_from_file(&vaults_path) {
                    let mut state_registry = state.vault_registry.write();
                    *state_registry = registry;

                    if let Some(last_used) = state_registry.get_last_used().cloned() {
                        let mut current_vault = state.current_vault.write();
                        *current_vault = Some(last_used);
                    }
                }
            }

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Vault 管理
            commands::open_vault,
            commands::add_vault_by_path,
            commands::list_vaults,
            commands::switch_vault,
            commands::close_vault,
            commands::remove_vault,
            commands::get_current_vault,
            commands::get_watcher_status,
            // 笔记与索引
            commands::list_all_notes,
            commands::get_note_meta,
            commands::get_backlinks,
            commands::get_outlinks,
            commands::get_unresolved_links,
            commands::get_all_tags,
            commands::resolve_wikilink,
            commands::rename_note,
            commands::delete_note,
            commands::delete_folder,
            commands::create_note,
            commands::create_folder,
            commands::reindex_all,
            // 搜索与图谱
            commands::search,
            commands::get_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
