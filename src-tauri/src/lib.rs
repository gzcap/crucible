//! ROC 应用核心库
//!
//! 这是 ROC 笔记应用的核心逻辑模块，基于 Tauri 框架构建。
//! 负责处理文件系统操作、索引构建、搜索功能和插件系统。
//!
//! ## 模块结构
//!
//! ```
//! src-tauri/src/
//! ├── main.rs          # 应用入口，调用 lib.rs 的 run() 函数
//! ├── lib.rs           # 核心库入口，Tauri 应用配置和命令注册
//! ├── commands.rs      # Tauri 命令定义（前端调用的 API）
//! ├── error.rs         # 错误类型定义和转换
//! ├── vault.rs         # Vault（笔记库）管理和注册表
//! ├── state.rs         # 全局应用状态管理
//! ├── parser.rs        # Markdown 解析（链接、标签、标题提取）
//! ├── watcher.rs       # 文件系统监控（监听笔记变化）
//! └── index/           # 索引模块
//!     ├── mod.rs       # 索引模块入口
//!     ├── graph.rs     # 链接图谱索引（正向链接、反向链接）
//!     └── search.rs    # 全文搜索索引（基于 Tantivy）
//! ```
//!
//! ## 核心组件
//!
//! - **VaultRegistry**: 管理多个笔记库的注册和切换
//! - **LinkIndex**: 维护笔记间的链接关系（正向/反向链接）
//! - **SearchIndex**: 基于 Tantivy 的全文搜索索引
//! - **FileWatcher**: 使用 notify 库监听文件系统变化
//! - **AppState**: 全局状态容器，持有所有共享资源

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

/// 启动 Tauri 应用
///
/// 配置应用窗口、插件、全局状态，并注册所有前端可调用的命令。
///
/// # 初始化流程
///
/// 1. 创建应用数据目录（`~/.roc/`）
/// 2. 初始化全局状态 `AppState`
/// 3. 从 `vaults.json` 加载已注册的 Vault 列表
/// 4. 恢复上次使用的 Vault（如果存在）
/// 5. 注册 Tauri 命令处理函数
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 注册 Tauri 插件
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        // 应用初始化
        .setup(|app| {
            // 获取应用数据目录路径
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            // 创建应用全局状态
            let handle = app.handle().clone();
            let state = AppState::new(app_data_dir.clone(), Arc::new(handle));

            // 从文件加载 Vault 注册表
            let vaults_path = app_data_dir.join("vaults.json");
            if vaults_path.exists() {
                if let Ok(registry) = VaultRegistry::load_from_file(&vaults_path) {
                    let mut state_registry = state.vault_registry.write();
                    *state_registry = registry;

                    // 恢复上次使用的 Vault
                    if let Some(last_used) = state_registry.get_last_used().cloned() {
                        let mut current_vault = state.current_vault.write();
                        *current_vault = Some(last_used);
                    }
                }
            }

            // 将状态注入 Tauri 应用上下文
            app.manage(state);

            Ok(())
        })
        // 注册前端可调用的命令
        .invoke_handler(tauri::generate_handler![
            // ========== Vault 管理命令 ==========
            commands::open_vault,
            commands::add_vault_by_path,
            commands::list_vaults,
            commands::switch_vault,
            commands::close_vault,
            commands::remove_vault,
            commands::get_current_vault,
            commands::get_watcher_status,
            // ========== 笔记与索引命令 ==========
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
            // ========== 搜索与图谱命令 ==========
            commands::search,
            commands::get_graph,
        ])
        // 启动应用
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}