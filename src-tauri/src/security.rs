//! 插件安全沙箱模块
//!
//! 负责插件权限管理、文件访问白名单、隔离恶意插件。
//! 所有插件的文件系统访问都必须经过此模块的权限校验。

use std::collections::HashSet;
use std::path::{Path, PathBuf};


use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionType {
    FileRead,
    FileWrite,
    FileCreate,
    FileDelete,
    NetworkAccess,
    SystemCommand,
    Export,
}

impl PermissionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "file-read" => Some(Self::FileRead),
            "file-write" => Some(Self::FileWrite),
            "file-create" => Some(Self::FileCreate),
            "file-delete" => Some(Self::FileDelete),
            "network-access" => Some(Self::NetworkAccess),
            "system-command" => Some(Self::SystemCommand),
            "export" => Some(Self::Export),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::FileRead => "file-read",
            Self::FileWrite => "file-write",
            Self::FileCreate => "file-create",
            Self::FileDelete => "file-delete",
            Self::NetworkAccess => "network-access",
            Self::SystemCommand => "system-command",
            Self::Export => "export",
        }
    }
}

pub struct PermissionManager {
    granted_permissions: HashMap<String, HashSet<PermissionType>>,
    file_whitelist: HashSet<PathBuf>,
    vault_path: PathBuf,
}

impl PermissionManager {
    pub fn new(vault_path: PathBuf) -> Self {
        let mut file_whitelist = HashSet::new();
        file_whitelist.insert(vault_path.clone());
        file_whitelist.insert(vault_path.join(".roc"));
        file_whitelist.insert(vault_path.join(".roc/plugins"));

        Self {
            granted_permissions: HashMap::new(),
            file_whitelist,
            vault_path,
        }
    }

    pub fn grant_permissions(&mut self, plugin_id: &str, permissions: &[PermissionType]) {
        let set = self.granted_permissions.entry(plugin_id.to_string()).or_insert_with(HashSet::new);
        set.extend(permissions.iter().cloned());
    }

    pub fn revoke_permissions(&mut self, plugin_id: &str) {
        self.granted_permissions.remove(plugin_id);
    }

    pub fn has_permission(&self, plugin_id: &str, permission: PermissionType) -> bool {
        self.granted_permissions
            .get(plugin_id)
            .map(|set| set.contains(&permission))
            .unwrap_or(false)
    }

    pub fn check_file_access(&self, plugin_id: &str, path: &Path) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::FileRead) &&
           !self.has_permission(plugin_id, PermissionType::FileWrite) {
            return Err(AppError::PluginPermissionDenied(plugin_id.to_string()));
        }

        let abs_path = path.canonicalize()?;
        let vault_abs = self.vault_path.canonicalize()?;

        if !abs_path.starts_with(&vault_abs) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} cannot access path outside vault: {}", plugin_id, path.display())));
        }

        Ok(())
    }

    pub fn can_read_file(&self, plugin_id: &str, path: &Path) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::FileRead) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires file-read permission", plugin_id)));
        }
        self.check_file_access(plugin_id, path)
    }

    pub fn can_write_file(&self, plugin_id: &str, path: &Path) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::FileWrite) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires file-write permission", plugin_id)));
        }
        self.check_file_access(plugin_id, path)
    }

    pub fn can_create_file(&self, plugin_id: &str, path: &Path) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::FileCreate) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires file-create permission", plugin_id)));
        }
        self.check_file_access(plugin_id, path)
    }

    pub fn can_delete_file(&self, plugin_id: &str, path: &Path) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::FileDelete) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires file-delete permission", plugin_id)));
        }
        self.check_file_access(plugin_id, path)
    }

    pub fn can_access_network(&self, plugin_id: &str) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::NetworkAccess) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires network-access permission", plugin_id)));
        }
        Ok(())
    }

    pub fn can_execute_command(&self, plugin_id: &str) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::SystemCommand) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires system-command permission", plugin_id)));
        }
        Ok(())
    }

    pub fn can_export(&self, plugin_id: &str) -> Result<()> {
        if !self.has_permission(plugin_id, PermissionType::Export) {
            return Err(AppError::PluginPermissionDenied(format!("plugin {} requires export permission", plugin_id)));
        }
        Ok(())
    }

    pub fn get_granted_permissions(&self, plugin_id: &str) -> Vec<String> {
        self.granted_permissions
            .get(plugin_id)
            .map(|set| set.iter().map(|p| p.to_str().to_string()).collect())
            .unwrap_or_default()
    }

    pub fn get_all_permissions() -> Vec<PluginPermission> {
        vec![
            PluginPermission {
                id: "file-read".to_string(),
                name: "文件读取".to_string(),
                description: "读取知识库中的文件".to_string(),
            },
            PluginPermission {
                id: "file-write".to_string(),
                name: "文件写入".to_string(),
                description: "修改知识库中的文件".to_string(),
            },
            PluginPermission {
                id: "file-create".to_string(),
                name: "文件创建".to_string(),
                description: "在知识库中创建文件和文件夹".to_string(),
            },
            PluginPermission {
                id: "file-delete".to_string(),
                name: "文件删除".to_string(),
                description: "删除知识库中的文件和文件夹".to_string(),
            },
            PluginPermission {
                id: "network-access".to_string(),
                name: "网络访问".to_string(),
                description: "访问网络资源".to_string(),
            },
            PluginPermission {
                id: "system-command".to_string(),
                name: "系统命令".to_string(),
                description: "执行系统命令".to_string(),
            },
            PluginPermission {
                id: "export".to_string(),
                name: "导出".to_string(),
                description: "导出文档为其他格式".to_string(),
            },
        ]
    }
}

use std::collections::HashMap;
