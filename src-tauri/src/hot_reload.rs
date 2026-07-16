//! 插件热加载模块
//!
//! 负责插件的动态加载、卸载和热更新。
//! 监听插件目录变化，自动触发重载，无需重启主程序。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::Result;
use crate::plugin_runtime::PluginManager;

pub struct HotReloader {
    watcher: Option<RecommendedWatcher>,
    plugin_manager: Arc<Mutex<PluginManager>>,
    plugin_dir: PathBuf,
    last_reload: Arc<Mutex<Instant>>,
    debounce_duration: Duration,
}

impl HotReloader {
    pub fn new(plugin_manager: Arc<Mutex<PluginManager>>, plugin_dir: PathBuf) -> Self {
        Self {
            watcher: None,
            plugin_manager,
            plugin_dir,
            last_reload: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10))),
            debounce_duration: Duration::from_millis(500),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let plugin_dir = self.plugin_dir.clone();
        let plugin_manager = self.plugin_manager.clone();
        let last_reload = self.last_reload.clone();
        let debounce_duration = self.debounce_duration;

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    Self::handle_event(event, &plugin_dir, &plugin_manager, &last_reload, debounce_duration);
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&self.plugin_dir, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.watcher.take();
    }

    fn handle_event(
        event: Event,
        plugin_dir: &Path,
        plugin_manager: &Arc<Mutex<PluginManager>>,
        last_reload: &Arc<Mutex<Instant>>,
        debounce_duration: Duration,
    ) {
        let now = Instant::now();
        let mut last = last_reload.lock().unwrap();
        if now.duration_since(*last) < debounce_duration {
            return;
        }
        *last = now;

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(plugin_id) = Self::extract_plugin_id(&path, plugin_dir) {
                        Self::reload_plugin(plugin_id, plugin_manager);
                        break;
                    }
                }
            }
            EventKind::Any | EventKind::Access(_) | EventKind::Other => {}
        }
    }

    fn extract_plugin_id(path: &Path, plugin_dir: &Path) -> Option<String> {
        path.strip_prefix(plugin_dir).ok().and_then(|rel| {
            rel.components().next().and_then(|c| c.as_os_str().to_str().map(|s| s.to_string()))
        })
    }

    fn reload_plugin(plugin_id: String, plugin_manager: &Arc<Mutex<PluginManager>>) {
        let mut manager = plugin_manager.lock().unwrap();

        let was_enabled = manager.is_enabled(&plugin_id);

        if was_enabled {
            let _ = manager.unload_plugin(&plugin_id);
        }

        let _ = manager.scan_plugins();

        if was_enabled {
            let _ = manager.load_plugin(&plugin_id);
        }
    }

    fn load_plugin(plugin_id: &str, plugin_manager: &Arc<Mutex<PluginManager>>) {
        let mut manager = plugin_manager.lock().unwrap();
        let _ = manager.scan_plugins();
        let _ = manager.load_plugin(plugin_id);
    }

    fn unload_plugin(plugin_id: &str, plugin_manager: &Arc<Mutex<PluginManager>>) {
        let mut manager = plugin_manager.lock().unwrap();
        let _ = manager.unload_plugin(plugin_id);
    }

    pub fn trigger_reload(&self, plugin_id: &str) -> Result<()> {
        let mut manager = self.plugin_manager.lock().unwrap();
        let was_enabled = manager.is_enabled(plugin_id);

        if was_enabled {
            manager.unload_plugin(plugin_id)?;
        }

        manager.scan_plugins()?;

        if was_enabled {
            manager.load_plugin(plugin_id)?;
        }

        Ok(())
    }

    pub fn reload_all(&self) -> Result<()> {
        let mut manager = self.plugin_manager.lock().unwrap();
        manager.unload_all_plugins();
        manager.scan_plugins()?;
        manager.load_all_plugins()?;
        Ok(())
    }
}
