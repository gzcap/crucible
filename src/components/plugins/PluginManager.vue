<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElIcon, ElSwitch, ElButton, ElTooltip, ElCard, ElLoading, ElDialog, ElInput } from 'element-plus'
import { Refresh, FolderOpened, Download, Setting, Upload, RefreshRight, Close, Check } from '@element-plus/icons-vue'
import { useVaultStore } from '../../stores/vault'
import { pluginLoader } from '../../plugins'
import type { PluginManifest } from '../../plugins/manifest'
import { openPath } from '@tauri-apps/plugin-opener'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface PluginItem {
  id: string
  name: string
  version: string
  description: string | undefined
  author: string | undefined
  enabled: boolean
  permissions: string[]
  manifest: PluginManifest
}

const vaultStore = useVaultStore()
const plugins = ref<PluginItem[]>([])
const loading = ref(false)
const error = ref('')
const showSettings = ref(false)
const selectedPlugin = ref<PluginItem | null>(null)
const pluginData = ref('')
const importingPlugin = ref(false)
const importPath = ref('')

const pluginDirPath = computed(() => {
  if (!vaultStore.currentVault) return null
  return `${vaultStore.currentVault.path}/.roc/plugins`
})

interface PluginInfo {
  id: string
  name: string
  version: string
  description: string | undefined
  author: string | undefined
  author_url: string | undefined
  repo: string | undefined
  min_app_version: string | undefined
  is_desktop_only: boolean | undefined
  permissions: string[] | undefined
  enabled: boolean
}

async function loadPlugins() {
  loading.value = true
  error.value = ''
  
  try {
    const pluginInfos = await invoke<PluginInfo[]>('list_installed_plugins')
    
    plugins.value = pluginInfos.map(info => ({
      id: info.id,
      name: info.name,
      version: info.version,
      description: info.description || '',
      author: info.author || '',
      enabled: info.enabled,
      permissions: info.permissions || [],
      manifest: {
        id: info.id,
        name: info.name,
        version: info.version,
        description: info.description,
        author: info.author,
        authorUrl: info.author_url,
        repo: info.repo,
        minAppVersion: info.min_app_version,
        isDesktopOnly: info.is_desktop_only,
        permissions: info.permissions,
      },
    } as PluginItem))
  } catch (e) {
    error.value = '加载插件列表失败'
    console.error('[PluginMarket] Failed to load plugins:', e)
  } finally {
    loading.value = false
  }
}

async function togglePlugin(pluginId: string, enabled: unknown) {
  try {
    if (enabled) {
      await invoke('load_plugin', { pluginId })
      await pluginLoader.enablePlugin(pluginId)
    } else {
      await invoke('unload_plugin', { pluginId })
      await pluginLoader.disablePlugin(pluginId)
    }
    await loadPlugins()
  } catch (e) {
    console.error('[PluginMarket] Failed to toggle plugin:', e)
    error.value = '操作失败'
  }
}

async function reloadPlugin(pluginId: string) {
  try {
    await pluginLoader.reloadPlugin(pluginId)
    await loadPlugins()
  } catch (e) {
    console.error('[PluginMarket] Failed to reload plugin:', e)
    error.value = '重载失败'
  }
}

async function openPluginDirectory() {
  if (!pluginDirPath.value) {
    error.value = '请先打开一个仓库'
    return
  }
  
  await openPath(pluginDirPath.value)
}

async function openPluginSettings(plugin: PluginItem) {
  selectedPlugin.value = plugin
  try {
    const data = await pluginLoader.getPluginData(plugin.id)
    pluginData.value = JSON.stringify(data, null, 2)
  } catch (e) {
    pluginData.value = '{}'
  }
  showSettings.value = true
}

async function savePluginData() {
  if (!selectedPlugin.value) return
  try {
    await pluginLoader.setPluginData(selectedPlugin.value.id, JSON.parse(pluginData.value))
    showSettings.value = false
    await loadPlugins()
  } catch (e) {
    console.error('[PluginMarket] Failed to save plugin data:', e)
    error.value = '保存失败'
  }
}

async function uninstallPlugin(pluginId: string) {
  if (!confirm('确定要卸载这个插件吗？这将删除插件文件。')) return
  
  try {
    await invoke('uninstall_plugin', { pluginId })
    await loadPlugins()
    console.log(`[PluginMarket] Plugin "${pluginId}" uninstalled`)
  } catch (e) {
    console.error('[PluginMarket] Failed to uninstall plugin:', e)
    error.value = '卸载失败'
  }
}

async function importPlugin() {
  if (!importPath.value || !pluginDirPath.value) return
  
  importingPlugin.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('copy_plugin', { sourcePath: importPath.value, targetDir: pluginDirPath.value })
    await loadPlugins()
    importPath.value = ''
  } catch (e) {
    console.error('[PluginMarket] Failed to import plugin:', e)
    error.value = '导入失败'
  } finally {
    importingPlugin.value = false
  }
}

onMounted(() => {
  loadPlugins()

  listen('roc://vault-opened', () => {
    loadPlugins()
  })

  listen('roc://vault-closed', () => {
    plugins.value = []
  })
})
</script>

<template>
  <div class="plugin-market">
    <div class="plugin-header">
      <div class="header-left">
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
            @click="loadPlugins"
            :loading="loading"
          >
            <ElIcon :size="14"><Refresh /></ElIcon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="从后端加载插件" placement="bottom">
          <el-button 
            size="small" 
            @click="loadPlugins"
            :disabled="!pluginDirPath"
          >
            <ElIcon :size="14"><Download /></ElIcon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="打开插件目录" placement="bottom">
          <el-button 
            size="small" 
            @click="openPluginDirectory"
            :disabled="!pluginDirPath"
          >
            <ElIcon :size="14"><FolderOpened /></ElIcon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <div v-if="error" class="plugin-error">{{ error }}</div>

    <div class="import-section" v-if="pluginDirPath">
      <div class="import-input-wrapper">
        <ElInput 
          v-model="importPath" 
          placeholder="输入插件目录路径" 
          size="small"
          class="import-input"
        />
        <el-button 
          size="small" 
          @click="importPlugin"
          :loading="importingPlugin"
        >
          <ElIcon :size="14"><Upload /></ElIcon>
          导入
        </el-button>
      </div>
    </div>

    <div v-if="loading" class="plugin-loading">
      <ElLoading :text="'加载中...'" />
    </div>

    <div v-else-if="plugins.length === 0" class="plugin-empty">
      <Download :size="32" class="empty-icon" />
      <p>暂无插件</p>
      <p class="empty-hint">将插件放入 {{ pluginDirPath || '.roc/plugins' }} 目录后点击加载</p>
    </div>

    <div v-else class="plugin-list">
      <ElCard 
        v-for="plugin in plugins" 
        :key="plugin.id" 
        class="plugin-card"
      >
        <div class="plugin-card-content">
          <div class="plugin-info">
            <div class="plugin-name-row">
              <span class="plugin-name">{{ plugin.name }}</span>
              <span v-if="plugin.author" class="plugin-author">{{ plugin.author }}</span>
            </div>
            <span class="plugin-version">{{ plugin.version }}</span>
            <p class="plugin-description">{{ plugin.description }}</p>
            <div v-if="plugin.permissions.length > 0" class="plugin-permissions">
              <span 
                v-for="perm in plugin.permissions" 
                :key="perm" 
                class="permission-tag"
              >
                {{ perm }}
              </span>
            </div>
          </div>
          <div class="plugin-actions">
            <div class="action-row">
              <ElSwitch 
                v-model="plugin.enabled"
                @change="(val) => togglePlugin(plugin.id, val)"
                active-text="启用"
                inactive-text="禁用"
              />
            </div>
            <div class="action-row">
              <el-tooltip content="重载插件" placement="bottom">
                <el-button 
                  size="small" 
                  @click="reloadPlugin(plugin.id)"
                  :disabled="!plugin.enabled"
                >
                  <ElIcon :size="14"><RefreshRight /></ElIcon>
                </el-button>
              </el-tooltip>
              <el-tooltip content="插件设置" placement="bottom">
                <el-button 
                  size="small" 
                  @click="openPluginSettings(plugin)"
                >
                  <ElIcon :size="14"><Setting /></ElIcon>
                </el-button>
              </el-tooltip>
            </div>
          </div>
        </div>
      </ElCard>
    </div>

    <div class="plugin-footer">
      <p>提示：将插件文件夹放入 .roc/plugins 目录后点击加载按钮</p>
      <p class="footer-hint">插件格式：包含 manifest.json 和 main.js 的文件夹</p>
    </div>

    <ElDialog 
      v-model="showSettings" 
      :title="selectedPlugin?.name + ' 设置'"
      width="650px"
      :close-on-click-modal="false"
    >
      <div class="settings-content">
        <div class="settings-info">
          <div class="info-row">
            <span class="info-label">插件 ID:</span>
            <span class="info-value">{{ selectedPlugin?.id }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">版本:</span>
            <span class="info-value">{{ selectedPlugin?.version }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">作者:</span>
            <span class="info-value">{{ selectedPlugin?.author || '未知' }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">描述:</span>
            <span class="info-value description">{{ selectedPlugin?.description || '无' }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">状态:</span>
            <span class="info-value" :class="selectedPlugin?.enabled ? 'status-enabled' : 'status-disabled'">
              {{ selectedPlugin?.enabled ? '已启用' : '已禁用' }}
            </span>
          </div>
          <div v-if="selectedPlugin && selectedPlugin.permissions && selectedPlugin.permissions.length > 0" class="info-row">
            <span class="info-label">权限:</span>
            <div class="permissions-wrap">
              <span 
                v-for="perm in selectedPlugin?.permissions" 
                :key="perm" 
                class="permission-tag"
              >
                {{ perm }}
              </span>
            </div>
          </div>
        </div>
        <div class="settings-divider"></div>
        <div class="settings-data">
          <p class="data-label">插件配置数据 (JSON)</p>
          <textarea 
            v-model="pluginData" 
            rows="10"
            class="data-textarea"
            placeholder="插件配置数据...（JSON格式）"
          />
          <p class="data-hint">修改后点击保存按钮生效</p>
        </div>
        <div class="settings-divider"></div>
        <div class="settings-actions">
          <p class="actions-label">危险操作</p>
          <el-button 
            type="danger" 
            size="small"
            @click="uninstallPlugin(selectedPlugin?.id || '')"
          >
            <ElIcon :size="14"><Close /></ElIcon>
            卸载插件
          </el-button>
          <p class="actions-hint">卸载将删除插件的所有文件，此操作不可恢复</p>
        </div>
      </div>
      <template #footer>
        <el-button @click="showSettings = false">取消</el-button>
        <el-button type="primary" @click="savePluginData">
          <ElIcon :size="14"><Check /></ElIcon>
          保存
        </el-button>
      </template>
    </ElDialog>
  </div>
</template>

<style scoped>
.plugin-market {
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

.import-section {
  margin-bottom: 12px;
}

.import-input-wrapper {
  display: flex;
  gap: 8px;
}

.import-input {
  flex: 1;
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

.plugin-author {
  font-size: 11px;
  color: var(--roc-text-muted);
}

.plugin-version {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin-left: 4px;
}

.plugin-description {
  font-size: 12px;
  color: var(--roc-text-secondary);
  margin: 4px 0 8px 0;
  line-height: 1.4;
}

.plugin-permissions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.permission-tag {
  font-size: 10px;
  padding: 2px 6px;
  background: var(--roc-bg-accent);
  color: var(--roc-text-primary);
  border-radius: 4px;
}

.plugin-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-row {
  display: flex;
  gap: 4px;
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

.footer-hint {
  margin-top: 4px !important;
}

.settings-content {
  padding: 8px 0;
}

.settings-info {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--roc-border);
}

.settings-info p {
  margin: 4px 0;
  font-size: 12px;
  color: var(--roc-text-secondary);
}

.settings-data {
  margin-top: 16px;
}

.data-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--roc-text-primary);
  margin: 0 0 8px 0;
}

.data-textarea {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 11px;
  width: 100%;
  padding: 8px;
  border: 1px solid var(--roc-border);
  border-radius: 4px;
  resize: vertical;
}

.data-hint {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin: 4px 0 0 0;
}

.settings-content {
  padding: 8px 0;
}

.settings-info {
  margin-bottom: 16px;
}

.info-row {
  display: flex;
  align-items: flex-start;
  margin-bottom: 8px;
}

.info-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--roc-text-muted);
  width: 80px;
  flex-shrink: 0;
}

.info-value {
  font-size: 12px;
  color: var(--roc-text-primary);
  flex: 1;
}

.info-value.description {
  line-height: 1.4;
}

.status-enabled {
  color: #67c23a;
}

.status-disabled {
  color: #909399;
}

.permissions-wrap {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.settings-divider {
  height: 1px;
  background: var(--roc-border);
  margin: 16px 0;
}

.settings-actions {
  padding-top: 8px;
}

.actions-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--roc-text-danger);
  margin: 0 0 8px 0;
}

.actions-hint {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin: 8px 0 0 0;
}
</style>
