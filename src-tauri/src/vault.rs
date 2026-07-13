use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub last_opened: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VaultRegistry {
    vaults: HashMap<String, VaultInfo>,
    last_used_id: Option<String>,
}

impl VaultRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, info: VaultInfo) -> Result<()> {
        if self.vaults.contains_key(&info.id) {
            return Err(AppError::VaultAlreadyExists(info.id));
        }
        self.vaults.insert(info.id.clone(), info);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<VaultInfo> {
        self.vaults.remove(id).ok_or(AppError::VaultNotFound(id.to_string()))
    }

    pub fn get(&self, id: &str) -> Option<&VaultInfo> {
        self.vaults.get(id)
    }

    pub fn list(&self) -> Vec<VaultInfo> {
        self.vaults.values().cloned().collect()
    }

    pub fn set_last_used(&mut self, id: &str) {
        self.last_used_id = Some(id.to_string());
    }

    pub fn get_last_used(&self) -> Option<&VaultInfo> {
        self.last_used_id.as_ref().and_then(|id| self.vaults.get(id))
    }

    pub fn exists_by_path(&self, path: &Path) -> bool {
        self.vaults.values().any(|v| v.path == path)
    }

    pub fn find_id_by_path(&self, path: &Path) -> Option<String> {
        self.vaults
            .iter()
            .find(|(_, v)| v.path == path)
            .map(|(id, _)| id.clone())
    }
}

pub fn generate_vault_id(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}
