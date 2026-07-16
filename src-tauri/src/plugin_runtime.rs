//! 插件运行时模块
//!
//! 负责插件的加载、执行、生命周期管理和安全沙箱。
//! 支持两种插件类型：
//! - JS/TS 插件：运行在 QuickJS 沙箱中，兼容 Obsidian 插件生态
//! - Native Rust 插件：编译为动态链接库，高性能执行

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginManifest {
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
    #[serde(rename = "type")]
    pub plugin_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PluginType {
    Js,
    Native,
}

#[derive(Debug)]
pub struct PluginInstance {
    pub manifest: PluginManifest,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub data: Arc<RwLock<serde_json::Value>>,
}

pub trait NativePlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn onload(&mut self, api: &PluginApi);
    fn onunload(&mut self);
}

pub struct PluginApi {
    vault_path: PathBuf,
}

impl PluginApi {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    pub fn read_file(&self, path: &str) -> Result<String> {
        let full_path = self.vault_path.join(path);
        Ok(fs::read_to_string(full_path)?)
    }

    pub fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.vault_path.join(path);
        fs::write(full_path, content)?;
        Ok(())
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<String>> {
        let full_path = self.vault_path.join(path);
        let mut entries = Vec::new();
        for entry in fs::read_dir(full_path)? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(entries)
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginState {
    enabled_plugins: Vec<String>,
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInstance>,
    api: Arc<PluginApi>,
    plugin_dir: PathBuf,
    state_path: PathBuf,
}

impl PluginManager {
    pub fn new(vault_path: PathBuf) -> Self {
        let plugin_dir = vault_path.join(".roc").join("plugins");
        let state_path = vault_path.join(".roc").join("plugins.json");
        let api = Arc::new(PluginApi::new(vault_path));

        Self {
            plugins: HashMap::new(),
            api,
            plugin_dir,
            state_path,
        }
    }

    pub fn scan_plugins(&mut self) -> Result<Vec<PluginManifest>> {
        let mut manifests = Vec::new();

        let enabled_plugins: Vec<String> = if self.state_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.state_path) {
                if let Ok(state) = serde_json::from_str::<PluginState>(&content) {
                    state.enabled_plugins
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if !self.plugin_dir.exists() {
            return Ok(manifests);
        }

        for entry in fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(mut manifest) = serde_json::from_str::<PluginManifest>(&content) {
                            let plugin_id = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&manifest.id)
                                .to_string();
                            manifest.id = plugin_id.clone();
                            manifests.push(manifest.clone());

                            let plugin_type = if manifest.plugin_type.as_deref() == Some("native") {
                                PluginType::Native
                            } else {
                                PluginType::Js
                            };

                            let is_enabled = enabled_plugins.contains(&plugin_id);

                            self.plugins.insert(
                                plugin_id,
                                PluginInstance {
                                    manifest,
                                    plugin_type,
                                    enabled: is_enabled,
                                    data: Arc::new(RwLock::new(serde_json::Value::Null)),
                                },
                            );
                        }
                    }
                }
            }
        }

        if enabled_plugins.is_empty() && !manifests.is_empty() {
            self.save_state()?;
        }

        Ok(manifests)
    }

    pub fn save_state(&self) -> Result<()> {
        let enabled_plugins: Vec<String> = self.plugins
            .values()
            .filter(|p| p.enabled)
            .map(|p| p.manifest.id.clone())
            .collect();

        let state = PluginState { enabled_plugins };
        let content = serde_json::to_string_pretty(&state)?;

        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.state_path, content)?;

        Ok(())
    }

    pub fn load_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let instance = self.plugins.get_mut(plugin_id).ok_or(AppError::PluginNotFound(plugin_id.to_string()))?;

        if instance.enabled {
            return Ok(());
        }

        let plugin_type = instance.plugin_type.clone();
        let plugin_dir = self.plugin_dir.clone();

        match plugin_type {
            PluginType::Js => Self::load_js_plugin_impl(instance, &plugin_dir)?,
            PluginType::Native => Self::load_native_plugin_impl(instance, &plugin_dir)?,
        }

        instance.enabled = true;
        self.save_state()?;
        Ok(())
    }

    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let instance = self.plugins.get_mut(plugin_id).ok_or(AppError::PluginNotFound(plugin_id.to_string()))?;

        if !instance.enabled {
            return Ok(());
        }

        let plugin_type = instance.plugin_type.clone();

        match plugin_type {
            PluginType::Js => Self::unload_js_plugin_impl(instance),
            PluginType::Native => Self::unload_native_plugin_impl(instance),
        }

        instance.enabled = false;
        self.save_state()?;
        Ok(())
    }

    pub fn load_all_plugins(&mut self) -> Result<Vec<String>> {
        let mut loaded = Vec::new();
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();
        for plugin_id in plugin_ids {
            if let Ok(_) = self.load_plugin(&plugin_id) {
                loaded.push(plugin_id);
            }
        }
        Ok(loaded)
    }

    pub fn unload_all_plugins(&mut self) {
        for plugin_id in self.plugins.keys().cloned().collect::<Vec<_>>() {
            let _ = self.unload_plugin(&plugin_id);
        }
    }

    fn load_js_plugin_impl(instance: &mut PluginInstance, plugin_dir: &Path) -> Result<()> {
        let main_path = plugin_dir.join(&instance.manifest.id).join("main.js");
        if !main_path.exists() {
            return Err(AppError::PluginLoadError(instance.manifest.id.clone(), "main.js not found".to_string()));
        }

        let _content = fs::read_to_string(&main_path)?;

        Ok(())
    }

    fn unload_js_plugin_impl(_instance: &mut PluginInstance) {
    }

    fn load_native_plugin_impl(instance: &mut PluginInstance, plugin_dir: &Path) -> Result<()> {
        let lib_name = format!("lib{}.dylib", instance.manifest.id);
        let lib_path = plugin_dir.join(&instance.manifest.id).join(&lib_name);

        if !lib_path.exists() {
            return Err(AppError::PluginLoadError(instance.manifest.id.clone(), "native library not found".to_string()));
        }

        Ok(())
    }

    fn unload_native_plugin_impl(_instance: &mut PluginInstance) {
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.plugins.get(plugin_id)
    }

    pub fn get_plugins(&self) -> Vec<&PluginInstance> {
        self.plugins.values().collect()
    }

    pub fn is_enabled(&self, plugin_id: &str) -> bool {
        self.plugins.get(plugin_id).map(|p| p.enabled).unwrap_or(false)
    }

    pub fn read_plugin_data(&self, plugin_id: &str) -> Result<String> {
        let instance = self.plugins.get(plugin_id).ok_or(AppError::PluginNotFound(plugin_id.to_string()))?;
        let data = instance.data.read().map_err(|e| AppError::PluginLoadError(plugin_id.to_string(), e.to_string()))?;
        Ok(serde_json::to_string_pretty(&*data)?)
    }

    pub fn write_plugin_data(&self, plugin_id: &str, data: &str) -> Result<()> {
        let instance = self.plugins.get(plugin_id).ok_or(AppError::PluginNotFound(plugin_id.to_string()))?;
        let value: serde_json::Value = serde_json::from_str(data)?;
        let mut data_mut = instance.data.write().map_err(|e| AppError::PluginLoadError(plugin_id.to_string(), e.to_string()))?;
        *data_mut = value;

        let data_path = self.plugin_dir.join(plugin_id).join("data.json");
        fs::create_dir_all(data_path.parent().unwrap())?;
        fs::write(data_path, data)?;

        Ok(())
    }
}
