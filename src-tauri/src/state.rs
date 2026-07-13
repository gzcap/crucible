use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tauri::AppHandle;

use crate::index::graph::LinkIndex;
use crate::index::search::SearchIndex;
use crate::vault::{VaultInfo, VaultRegistry};

use notify_debouncer_full::{Debouncer, FileIdMap};
use notify::RecommendedWatcher;

pub struct AppState {
    pub current_vault: RwLock<Option<VaultInfo>>,
    pub vault_registry: RwLock<VaultRegistry>,
    pub link_index: LinkIndex,
    pub search_index: SearchIndex,
    pub watcher: Mutex<Option<Debouncer<RecommendedWatcher, FileIdMap>>>,
    pub app_data_dir: RwLock<PathBuf>,
    pub app_handle: Arc<AppHandle>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf, app_handle: Arc<AppHandle>) -> Self {
        Self {
            current_vault: RwLock::new(None),
            vault_registry: RwLock::new(VaultRegistry::new()),
            link_index: LinkIndex::new(),
            search_index: SearchIndex::new(),
            watcher: Mutex::new(None),
            app_data_dir: RwLock::new(app_data_dir),
            app_handle,
        }
    }

    pub fn get_vaults_path(&self) -> PathBuf {
        self.app_data_dir.read().join("vaults.json")
    }
    pub fn get_index_dir(&self) -> PathBuf {
        self.app_data_dir.read().join("indexes")
    }
}
