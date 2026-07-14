<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElIcon, ElSwitch, ElButton, ElTooltip, ElCard, ElInput, ElLoading } from 'element-plus'
import { 
  Setting, 
  InfoFilled, 
  Edit, 
  FolderOpened, 
  Document, 
  Link, 
  MoreFilled, 
  Key, 
  Folder, 
  Refresh, 
  Search,
  Share,
  Close,
  Check,
  Delete,
  Aim
} from '@element-plus/icons-vue'
import { useVaultStore } from '../../stores/vault'
import { rocApp, pluginLoader } from '../../plugins'
import type { PluginManifest } from '../../plugins/manifest'
import { openPath } from '@tauri-apps/plugin-opener'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

interface PluginItem {
  id: string
  name: string
  version: string
  description: string | undefined
  enabled: boolean
  builtin: boolean
  author?: string
  manifest: PluginManifest
}

const vaultStore = useVaultStore()
const activeTab = ref('plugins')
const searchQuery = ref('')
const plugins = ref<PluginItem[]>([])
const loading = ref(false)
const error = ref('')
const safeMode = ref(false)
const autoUpdate = ref(true)

interface NavItem {
  id: string
  icon: any
  label: string
}

const navItems: NavItem[] = [
  { id: 'about', icon: InfoFilled, label: '关于' },
  { id: 'editor', icon: Edit, label: '编辑器' },
  { id: 'files', icon: FolderOpened, label: '文件与链接' },
  { id: 'appearance', icon: Document, label: '外观' },
  { id: 'links', icon: Link, label: '链接' },
  { id: 'hotkeys', icon: MoreFilled, label: '快捷键' },
  { id: 'keys', icon: Key, label: '钥匙串' },
  { id: 'core', icon: Folder, label: '核心插件' },
  { id: 'plugins', icon: Setting, label: '第三方插件' },
]

const pluginDirPath = computed(() => {
  if (!vaultStore.currentVault) return null
  return `${vaultStore.currentVault.path}/.roc/plugins`
})

const filteredPlugins = computed(() => {
  if (!searchQuery.value) return plugins.value
  const query = searchQuery.value.toLowerCase()
  return plugins.value.filter(p => 
    p.name.toLowerCase().includes(query) ||
    p.description?.toLowerCase().includes(query) ||
    p.author?.toLowerCase().includes(query)
  )
})

const corePlugins = computed(() => filteredPlugins.value.filter(p => p.builtin))
const thirdPartyPlugins = computed(() => filteredPlugins.value.filter(p => !p.builtin))

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
      author: (entry.manifest as any).author,
      manifest: entry.manifest,
    }))
  } catch (e) {
    error.value = '加载插件列表失败'
    console.error('[SettingsModal] Failed to load plugins:', e)
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
    console.error('[SettingsModal] Failed to load plugins from vault:', e)
  } finally {
    loading.value = false
  }
}

async function openPluginDirectory() {
  if (!pluginDirPath.value) {
    error.value = '请先打开一个仓库'
    return
  }
  await openPath(pluginDirPath.value)
}

function closeModal() {
  emit('close')
}

onMounted(() => {
  loadPlugins()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="settings-overlay" @click.self="closeModal">
        <div class="settings-modal">
          <div class="settings-header">
            <span class="settings-title">选项</span>
            <button class="close-btn" @click="closeModal">
              <Close :size="16" />
            </button>
          </div>
          
          <div class="settings-body">
            <aside class="settings-sidebar">
              <nav class="settings-nav">
                <button
                  v-for="item in navItems"
                  :key="item.id"
                  class="nav-item"
                  :class="{ active: activeTab === item.id }"
                  @click="activeTab = item.id"
                >
                  <ElIcon :size="16" class="nav-icon">
                    <component :is="item.icon" />
                  </ElIcon>
                  <span class="nav-label">{{ item.label }}</span>
                </button>
              </nav>
              
              <div v-if="thirdPartyPlugins.length > 0" class="sidebar-footer">
                <span class="sidebar-footer-title">第三方插件</span>
                <button
                  v-for="plugin in thirdPartyPlugins"
                  :key="plugin.id"
                  class="footer-plugin"
                  :class="{ active: activeTab === plugin.id }"
                  @click="activeTab = plugin.id"
                >
                  {{ plugin.name }}
                </button>
              </div>
            </aside>
            
            <main class="settings-content">
              <div v-if="activeTab === 'plugins'" class="plugins-page">
                <div class="page-section">
                  <div class="section-header">
                    <h3 class="section-title">安全模式</h3>
                    <ElSwitch 
                      v-model="safeMode"
                      active-text="开启"
                      inactive-text="关闭"
                    />
                  </div>
                  <p class="section-description">
                    安全模式已关闭。开启以限制第三方插件运行。
                  </p>
                </div>
                
                <div class="page-section">
                  <div class="section-header">
                    <h3 class="section-title">社区插件市场</h3>
                    <ElButton size="small" class="primary-btn">浏览</ElButton>
                  </div>
                  <p class="section-description">
                    浏览、安装社区成员制作的第三方插件。
                  </p>
                </div>
                
                <div class="page-section">
                  <div class="section-header">
                    <h3 class="section-title">插件安装情况</h3>
                    <ElButton size="small">检查更新</ElButton>
                  </div>
                  <p class="section-description">
                    你目前已经安装了 {{ plugins.length }} 个插件。
                  </p>
                </div>
                
                <div class="page-section">
                  <div class="section-header">
                    <h3 class="section-title">自动检查插件更新</h3>
                    <ElSwitch 
                      v-model="autoUpdate"
                      active-text="开启"
                      inactive-text="关闭"
                    />
                  </div>
                  <p class="section-description">
                    定期检查第三方插件的更新。
                  </p>
                </div>
                
                <div class="installed-plugins">
                  <div class="installed-header">
                    <h3 class="installed-title">已安装插件</h3>
                    <div class="installed-actions">
                      <el-tooltip content="刷新插件列表" placement="bottom">
                        <button class="action-btn" @click="loadPlugins" :disabled="loading">
                          <Refresh :size="14" />
                        </button>
                      </el-tooltip>
                      <el-tooltip content="打开插件目录" placement="bottom">
                        <button class="action-btn" @click="openPluginDirectory" :disabled="!pluginDirPath">
                          <Share :size="14" />
                        </button>
                      </el-tooltip>
                    </div>
                  </div>
                  
                  <ElInput
                    v-model="searchQuery"
                    placeholder="搜索已安装的插件..."
                    size="small"
                    class="plugin-search"
                    prefix-icon="Search"
                  />
                  
                  <div v-if="error" class="plugin-error">{{ error }}</div>
                  
                  <div v-if="loading" class="plugin-loading">
                    <ElLoading :text="'加载中...'" />
                  </div>
                  
                  <div v-else-if="plugins.length === 0" class="plugin-empty">
                    <Aim :size="32" class="empty-icon" />
                    <p>暂无插件</p>
                    <p class="empty-hint">将插件文件放入 {{ pluginDirPath || '.roc/plugins' }} 目录后点击刷新</p>
                  </div>
                  
                  <div v-else class="plugin-list">
                    <div v-for="plugin in filteredPlugins" :key="plugin.id" class="plugin-card">
                      <div class="plugin-card-header">
                        <div class="plugin-info">
                          <span class="plugin-name">{{ plugin.name }}</span>
                          <span v-if="plugin.builtin" class="plugin-badge">内置</span>
                        </div>
                        <div class="plugin-card-actions">
                          <el-tooltip content="设置" placement="bottom">
                            <button class="card-action-btn" :disabled="!plugin.enabled">
                              <Setting :size="14" />
                            </button>
                          </el-tooltip>
                          <el-tooltip content="复制ID" placement="bottom">
                            <button class="card-action-btn">
                              <Check :size="14" />
                            </button>
                          </el-tooltip>
                          <el-tooltip content="删除" placement="bottom">
                            <button class="card-action-btn delete-btn" :disabled="plugin.builtin">
                              <Delete :size="14" />
                            </button>
                          </el-tooltip>
                          <ElSwitch 
                            v-model="plugin.enabled"
                            @change="(val) => togglePlugin(plugin.id, val)"
                            :disabled="plugin.builtin"
                            class="plugin-switch"
                          />
                        </div>
                      </div>
                      <div class="plugin-card-body">
                        <span class="plugin-version">版本: {{ plugin.version }}</span>
                        <span v-if="plugin.author" class="plugin-author">作者: {{ plugin.author }}</span>
                        <p class="plugin-description">{{ plugin.description }}</p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              
              <div v-else class="placeholder-page">
                <Aim :size="48" class="placeholder-icon" />
                <h3 class="placeholder-title">{{ navItems.find(n => n.id === activeTab)?.label }}</h3>
                <p class="placeholder-description">此功能正在开发中...</p>
              </div>
            </main>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-modal {
  width: 800px;
  height: 600px;
  background: var(--roc-bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--roc-border);
  background: var(--roc-bg-secondary);
}

.settings-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--roc-text-primary);
}

.close-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 4px;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: background 0.15s;
}

.close-btn:hover {
  background: var(--roc-bg-tertiary);
}

.settings-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.settings-sidebar {
  width: 180px;
  border-right: 1px solid var(--roc-border);
  background: var(--roc-bg-secondary);
  display: flex;
  flex-direction: column;
}

.settings-nav {
  padding: 8px 0;
}

.nav-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: background 0.15s;
  color: var(--roc-text-secondary);
}

.nav-item:hover {
  background: var(--roc-bg-tertiary);
}

.nav-item.active {
  background: rgba(0, 122, 204, 0.15);
  color: var(--roc-accent);
}

.nav-icon {
  flex-shrink: 0;
}

.nav-label {
  font-size: 12px;
}

.sidebar-footer {
  margin-top: auto;
  padding: 8px 0;
  border-top: 1px solid var(--roc-border);
}

.sidebar-footer-title {
  display: block;
  font-size: 10px;
  color: var(--roc-text-muted);
  padding: 4px 12px;
}

.footer-plugin {
  width: 100%;
  text-align: left;
  padding: 4px 12px;
  padding-left: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  color: var(--roc-text-secondary);
  transition: background 0.15s;
}

.footer-plugin:hover {
  background: var(--roc-bg-tertiary);
}

.footer-plugin.active {
  background: rgba(0, 122, 204, 0.15);
  color: var(--roc-accent);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.plugins-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-section {
  background: var(--roc-bg-secondary);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--roc-text-primary);
  margin: 0;
}

.section-description {
  font-size: 12px;
  color: var(--roc-text-secondary);
  margin: 0;
}

.primary-btn {
  background: var(--roc-accent);
  color: white;
  border: none;
}

.primary-btn:hover {
  background: var(--roc-accent-hover);
}

.installed-plugins {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.installed-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.installed-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--roc-text-primary);
  margin: 0;
}

.installed-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: var(--roc-bg-secondary);
  border-radius: 4px;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: background 0.15s;
}

.action-btn:hover:not(:disabled) {
  background: var(--roc-bg-tertiary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.plugin-search {
  margin-bottom: 12px;
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
  gap: 8px;
}

.plugin-card {
  background: var(--roc-bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--roc-border);
  overflow: hidden;
}

.plugin-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--roc-border);
}

.plugin-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 13px;
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

.plugin-card-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.card-action-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 4px;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: background 0.15s;
}

.card-action-btn:hover:not(:disabled) {
  background: var(--roc-bg-tertiary);
}

.card-action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.delete-btn:hover:not(:disabled) {
  color: var(--roc-text-danger);
}

.plugin-switch {
  margin-left: 8px;
}

.plugin-card-body {
  padding: 12px;
}

.plugin-version {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin-right: 12px;
}

.plugin-author {
  font-size: 11px;
  color: var(--roc-text-muted);
}

.plugin-description {
  font-size: 12px;
  color: var(--roc-text-secondary);
  margin: 6px 0 0 0;
  line-height: 1.4;
}

.placeholder-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--roc-text-muted);
}

.placeholder-icon {
  margin-bottom: 12px;
  opacity: 0.3;
}

.placeholder-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--roc-text-primary);
  margin: 0 0 8px 0;
}

.placeholder-description {
  font-size: 12px;
  margin: 0;
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .settings-modal,
.modal-leave-to .settings-modal {
  transform: scale(0.95);
}
</style>