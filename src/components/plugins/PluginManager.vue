<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElIcon, ElSwitch, ElButton, ElTooltip, ElCard, ElLoading } from 'element-plus'
import { Refresh, FolderOpened, Plus, Delete, InfoFilled } from '@element-plus/icons-vue'
import { useVaultStore } from '../../stores/vault'
import { rocApp, pluginLoader } from '../../plugins'
import type { PluginManifest } from '../../plugins/manifest'
import { openPath } from '@tauri-apps/plugin-opener'

interface PluginItem {
  id: string
  name: string
  version: string
  description: string | undefined
  enabled: boolean
  builtin: boolean
  manifest: PluginManifest
}

const vaultStore = useVaultStore()
const plugins = ref<PluginItem[]>([])
const loading = ref(false)
const error = ref('')

const pluginDirPath = computed(() => {
  if (!vaultStore.currentVault) return null
  return `${vaultStore.currentVault.path}/.roc/plugins`
})

async function loadPlugins() {
  loading.value = true
  error.value = ''
  
  try {
    const pluginEntries = rocApp.getPlugins()
    plugins.value = pluginEntries.map(entry => ({
      id: entry.manifest.id,
      name: entry.manifest.name,
      version: entry.manifest.version,
      description: entry.manifest.description,
      enabled: entry.enabled,
      builtin: entry.manifest.id.startsWith('roc-'),
      manifest: entry.manifest,
    }))
  } catch (e) {
    error.value = '加载插件列表失败'
    console.error('[PluginManager] Failed to load plugins:', e)
  } finally {
    loading.value = false
  }
}

async function togglePlugin(pluginId: string, enabled: unknown) {
  if (enabled) {
    rocApp.enablePlugin(pluginId)
  } else {
    rocApp.disablePlugin(pluginId)
  }
  await loadPlugins()
}

async function loadPluginsFromVault() {
  if (!pluginDirPath.value) {
    error.value = '请先打开一个仓库'
    return
  }

  loading.value = true
  error.value = ''

  try {
    await pluginLoader.loadPluginsFromDirectory(pluginDirPath.value)
    await loadPlugins()
  } catch (e) {
    error.value = '从仓库加载插件失败'
    console.error('[PluginManager] Failed to load plugins from vault:', e)
  } finally {
    loading.value = false
  }
}

async function refreshPlugins() {
  await loadPlugins()
}

async function openPluginDirectory() {
  if (!pluginDirPath.value) {
    error.value = '请先打开一个仓库'
    return
  }
  
  await openPath(pluginDirPath.value)
}

onMounted(() => {
  loadPlugins()
})
</script>

<template>
  <div class="plugin-manager">
    <div class="plugin-header">
      <div class="header-left">
        <h3 class="plugin-title">插件管理</h3>
        <span v-if="pluginDirPath" class="plugin-dir-hint">
          插件目录: {{ pluginDirPath }}
        </span>
        <span v-else class="plugin-dir-hint plugin-dir-hint--empty">
          请打开一个仓库以管理插件
        </span>
      </div>
      <div class="header-actions">
        <el-tooltip content="刷新插件列表" placement="bottom">
          <el-button 
            size="small" 
            @click="refreshPlugins"
            :loading="loading"
          >
            <ElIcon :size="14"><Refresh /></ElIcon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="从仓库加载插件" placement="bottom">
          <el-button 
            size="small" 
            @click="loadPluginsFromVault"
            :disabled="!pluginDirPath"
          >
            <ElIcon :size="14"><FolderOpen /></ElIcon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="打开插件目录" placement="bottom">
          <el-button 
            size="small" 
            @click="openPluginDirectory"
            :disabled="!pluginDirPath"
          >
            <ElIcon :size="14"><Plus /></ElIcon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <div v-if="error" class="plugin-error">{{ error }}</div>

    <div v-if="loading" class="plugin-loading">
      <ElLoading :text="'加载中...'" />
    </div>

    <div v-else-if="plugins.length === 0" class="plugin-empty">
      <InfoFilled :size="32" class="empty-icon" />
      <p>暂无插件</p>
      <p class="empty-hint">将插件文件放入 {{ pluginDirPath || '.roc/plugins' }} 目录后点击加载</p>
    </div>

    <div v-else class="plugin-list">
      <ElCard 
        v-for="plugin in plugins" 
        :key="plugin.id" 
        class="plugin-card"
        :class="{ 'plugin-card--builtin': plugin.builtin }"
      >
        <div class="plugin-card-content">
          <div class="plugin-info">
            <div class="plugin-name-row">
              <span class="plugin-name">{{ plugin.name }}</span>
              <span v-if="plugin.builtin" class="plugin-badge">内置</span>
            </div>
            <span class="plugin-version">{{ plugin.version }}</span>
            <p class="plugin-description">{{ plugin.description }}</p>
          </div>
          <div class="plugin-actions">
            <ElSwitch 
              v-model="plugin.enabled"
              @change="(val) => togglePlugin(plugin.id, val)"
              :disabled="plugin.builtin"
              active-text="启用"
              inactive-text="禁用"
            />
          </div>
        </div>
      </ElCard>
    </div>

    <div class="plugin-footer">
      <p>提示：将插件 JS 文件放入仓库根目录的 .roc/plugins 文件夹中即可自动加载</p>
    </div>
  </div>
</template>

<style scoped>
.plugin-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow: hidden;
}

.plugin-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.header-left {
  flex: 1;
}

.plugin-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: var(--roc-text-primary);
}

.plugin-dir-hint {
  font-size: 11px;
  color: var(--roc-text-muted);
  display: block;
}

.plugin-dir-hint--empty {
  color: var(--roc-text-danger);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.plugin-error {
  background: var(--roc-bg-error);
  color: var(--roc-text-danger);
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
  margin-bottom: 12px;
}

.plugin-loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--roc-text-muted);
}

.empty-icon {
  margin-bottom: 8px;
  opacity: 0.5;
}

.empty-hint {
  font-size: 12px;
  margin-top: 4px;
}

.plugin-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.plugin-card {
  border-radius: 8px;
  border: 1px solid var(--roc-border);
  transition: all 0.2s;
}

.plugin-card:hover {
  border-color: var(--roc-border-hover);
}

.plugin-card--builtin {
  background: var(--roc-bg-tertiary);
}

.plugin-card-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.plugin-info {
  flex: 1;
  margin-right: 16px;
}

.plugin-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--roc-text-primary);
}

.plugin-badge {
  font-size: 10px;
  padding: 1px 6px;
  background: var(--roc-bg-accent);
  color: var(--roc-text-primary);
  border-radius: 10px;
}

.plugin-version {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin-left: 4px;
}

.plugin-description {
  font-size: 12px;
  color: var(--roc-text-secondary);
  margin: 4px 0 0 0;
  line-height: 1.4;
}

.plugin-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.plugin-footer {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--roc-border);
}

.plugin-footer p {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin: 0;
  text-align: center;
}
</style>