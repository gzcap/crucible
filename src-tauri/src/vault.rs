//! Vault（笔记库）管理
//!
//! 定义 Vault 信息结构和注册表管理逻辑，负责多个笔记库的注册、存储和切换。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Vault 信息结构
///
/// 表示一个笔记库的基本信息，包含唯一标识、名称、路径和最后打开时间。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    /// Vault 唯一标识符（基于路径的 SHA256 哈希）
    pub id: String,
    /// Vault 名称（取自文件夹名）
    pub name: String,
    /// Vault 所在路径
    pub path: PathBuf,
    /// 最后打开时间（RFC3339 格式）
    pub last_opened: Option<String>,
}

/// Vault 注册表
///
/// 管理所有已注册的 Vault，支持增删改查操作，并持久化到 `vaults.json` 文件。
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct VaultRegistry {
    /// Vault 映射表：id -> VaultInfo
    vaults: HashMap<String, VaultInfo>,
    /// 上次使用的 Vault ID
    last_used_id: Option<String>,
}

impl VaultRegistry {
    /// 创建新的空注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 从文件加载注册表
    ///
    /// 如果文件不存在，返回空注册表。
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// 将注册表保存到文件
    ///
    /// 自动创建父目录。
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 添加 Vault 到注册表
    ///
    /// 如果 ID 已存在，返回 `VaultAlreadyExists` 错误。
    pub fn add(&mut self, info: VaultInfo) -> Result<()> {
        if self.vaults.contains_key(&info.id) {
            return Err(AppError::VaultAlreadyExists(info.id));
        }
        self.vaults.insert(info.id.clone(), info);
        Ok(())
    }

    /// 从注册表移除 Vault
    ///
    /// 如果 ID 不存在，返回 `VaultNotFound` 错误。
    pub fn remove(&mut self, id: &str) -> Result<VaultInfo> {
        self.vaults.remove(id).ok_or(AppError::VaultNotFound(id.to_string()))
    }

    /// 根据 ID 获取 Vault 信息
    pub fn get(&self, id: &str) -> Option<&VaultInfo> {
        self.vaults.get(id)
    }

    /// 获取所有 Vault 列表
    ///
    /// 按最后打开时间降序排序。
    pub fn list(&self) -> Vec<VaultInfo> {
        let mut vaults: Vec<VaultInfo> = self.vaults.values().cloned().collect();
        vaults.sort_by(|a, b| {
            match (a.last_opened.as_ref(), b.last_opened.as_ref()) {
                (Some(a), Some(b)) => b.cmp(a),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        vaults
    }

    /// 设置上次使用的 Vault ID
    pub fn set_last_used(&mut self, id: &str) {
        self.last_used_id = Some(id.to_string());
    }

    /// 获取上次使用的 Vault
    pub fn get_last_used(&self) -> Option<&VaultInfo> {
        self.last_used_id.as_ref().and_then(|id| self.vaults.get(id))
    }

    /// 检查路径是否已注册
    pub fn exists_by_path(&self, path: &Path) -> bool {
        self.vaults.values().any(|v| v.path == path)
    }

    /// 根据路径查找 Vault ID
    pub fn find_id_by_path(&self, path: &Path) -> Option<String> {
        self.vaults
            .iter()
            .find(|(_, v)| v.path == path)
            .map(|(id, _)| id.clone())
    }
}

/// 生成 Vault ID
///
/// 基于路径的 SHA256 哈希生成唯一标识符，确保同一路径始终生成相同的 ID。
pub fn generate_vault_id(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}