//! 全局应用状态管理
//!
//! 定义应用的全局状态容器 `AppState`，持有所有共享资源，包括：
//! - 当前打开的 Vault
//! - Vault 注册表
//! - 链接索引
//! - 搜索索引
//! - 文件监控器
//! - 应用数据目录
//! - Tauri 应用句柄

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use tauri::AppHandle;

use crate::index::graph::LinkIndex;
use crate::index::search::SearchIndex;
use crate::plugin_runtime::PluginManager;
use crate::security::PermissionManager;
use crate::vault::{VaultInfo, VaultRegistry};

use notify_debouncer_full::{Debouncer, FileIdMap};
use notify::RecommendedWatcher;

/// 应用全局状态
///
/// 使用 `parking_lot` 的锁类型确保高效并发访问：
/// - `RwLock`：适用于多读少写场景（如 current_vault, vault_registry）
/// - `Mutex`：适用于独占访问场景（如 watcher）
/// - 无锁：适用于线程安全的数据结构（如 link_index, search_index）
pub struct AppState {
    /// 当前打开的 Vault（读写锁保护）
    pub current_vault: RwLock<Option<VaultInfo>>,
    /// Vault 注册表（读写锁保护）
    pub vault_registry: RwLock<VaultRegistry>,
    /// 链接索引（线程安全，内部使用 DashMap）
    pub link_index: LinkIndex,
    /// 搜索索引（线程安全，内部使用 Mutex/RwLock）
    pub search_index: SearchIndex,
    /// 文件监控器（互斥锁保护）
    pub watcher: Mutex<Option<Debouncer<RecommendedWatcher, FileIdMap>>>,
    /// 待处理的删除事件（用于检测 Finder 重命名：先删后建）
    /// key: 文件 stem，value: (旧路径, 时间戳)
    pub pending_deletes: Mutex<HashMap<String, (String, Instant)>>,
    /// 应用数据目录路径（读写锁保护）
    pub app_data_dir: RwLock<PathBuf>,
    /// Tauri 应用句柄（原子引用计数）
    pub app_handle: Arc<AppHandle>,
    /// 插件管理器（互斥锁保护）
    pub plugin_manager: Mutex<Option<PluginManager>>,
    /// 权限管理器（互斥锁保护）
    pub permission_manager: Mutex<Option<PermissionManager>>,
}

impl AppState {
    /// 创建新的应用状态
    ///
    /// # 参数
    ///
    /// - `app_data_dir`: 应用数据目录路径（通常为 `~/.roc/`）
    /// - `app_handle`: Tauri 应用句柄，用于发送事件和访问系统资源
    pub fn new(app_data_dir: PathBuf, app_handle: Arc<AppHandle>) -> Self {
        Self {
            current_vault: RwLock::new(None),
            vault_registry: RwLock::new(VaultRegistry::new()),
            link_index: LinkIndex::new(),
            search_index: SearchIndex::new(),
            watcher: Mutex::new(None),
            pending_deletes: Mutex::new(HashMap::new()),
            app_data_dir: RwLock::new(app_data_dir),
            app_handle,
            plugin_manager: Mutex::new(None),
            permission_manager: Mutex::new(None),
        }
    }

    pub fn init_plugin_manager(&self, vault_path: PathBuf) {
        let mut pm = self.plugin_manager.lock();
        *pm = Some(PluginManager::new(vault_path.clone()));

        let mut perm_mgr = self.permission_manager.lock();
        *perm_mgr = Some(PermissionManager::new(vault_path));
    }

    /// 获取 Vault 注册表文件路径
    ///
    /// 返回 `{app_data_dir}/vaults.json`
    pub fn get_vaults_path(&self) -> PathBuf {
        self.app_data_dir.read().join("vaults.json")
    }

    /// 获取索引目录路径
    ///
    /// 返回 `{app_data_dir}/indexes/`
    pub fn get_index_dir(&self) -> PathBuf {
        self.app_data_dir.read().join("indexes")
    }
}