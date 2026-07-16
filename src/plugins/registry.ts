import type { Plugin } from './types'
import type { PluginManifest } from './manifest'
import type {
  App as AppInterface,
  Vault,
  Workspace,
  MetadataCache,
  Commands,
  Events,
  Command,
  Hotkey,
  SidebarPanel,
  PluginSettingTab,
  PopoverOptions,
  ModalOptions,
  FileCache,
  LinkCache,
  HeadingCache,
  Backlink,
  Position,
} from './app'
import { useNotesStore } from '../stores/notes'
import { useVaultStore } from '../stores/vault'
import { useSearchStore } from '../stores/search'
import {
  listAllNotes,
  deleteNote,
  createNote,
  createFolder,
  renameNote,
  getBacklinks,
  getAllTags,
  resolveWikilink,
} from '../lib/tauri'
import { readTextFile, writeTextFile, mkdir } from '@tauri-apps/plugin-fs'

/**
 * 插件注册表项
 */
interface PluginEntry {
  pluginClass: new (app: App) => Plugin
  manifest: PluginManifest
  instance?: Plugin
  enabled: boolean
}

/**
 * 事件发射器实现
 */
class EventEmitter implements Events {
  private listeners = new Map<string, Set<(...args: any[]) => void>>()

  on(event: string, callback: (...args: any[]) => void): void {
    const set = this.listeners.get(event) || new Set()
    set.add(callback)
    this.listeners.set(event, set)
  }

  off(event: string, callback: (...args: any[]) => void): void {
    const set = this.listeners.get(event)
    if (set) {
      set.delete(callback)
      if (set.size === 0) {
        this.listeners.delete(event)
      }
    }
  }

  emit(event: string, ...args: any[]): void {
    const set = this.listeners.get(event)
    if (set) {
      set.forEach((callback) => {
        try {
          callback(...args)
        } catch (error) {
          console.error(`[EventEmitter] Error emitting "${event}":`, error)
        }
      })
    }
  }
}

/**
 * Vault 实现
 * 基于 Tauri 后端和 notesStore 提供文件操作能力
 */
class VaultImpl implements Vault {
  get path(): string | null {
    return useVaultStore().currentVault?.path || null
  }

  async getMarkdownFiles(): Promise<string[]> {
    const notes = await listAllNotes()
    return notes.map((n) => n.path)
  }

  async read(filePath: string): Promise<string> {
    return readTextFile(filePath)
  }

  async write(filePath: string, content: string): Promise<void> {
    await writeTextFile(filePath, content)
  }

  async create(filePath: string, content = ''): Promise<void> {
    await writeTextFile(filePath, content || '# New Note\n\n')
  }

  async delete(filePath: string): Promise<void> {
    await deleteNote(filePath)
  }

  async createFolder(folderPath: string): Promise<void> {
    await mkdir(folderPath, { recursive: true })
  }

  async rename(oldPath: string, newPath: string): Promise<void> {
    await renameNote(oldPath, newPath)
  }
}

/**
 * Workspace 实现
 * 基于 notesStore 和路由提供界面管理能力
 */
class WorkspaceImpl implements Workspace {
  private sidebarPanels = new Map<string, SidebarPanel>()
  private settingTabs = new Map<string, PluginSettingTab>()

  get activeFilePath(): string | null {
    return useNotesStore().currentPath
  }

  openFile(filePath: string): void {
    useNotesStore().openNote(filePath)
  }

  closeFile(filePath: string): void {
    useNotesStore().closeTab(filePath)
  }

  createTab(filePath: string): void {
    useNotesStore().openNote(filePath)
  }

  switchView(viewType: string): void {
    if (viewType === 'graph') {
      useNotesStore().openGraph()
    }
  }

  registerSidebarPanel(panel: SidebarPanel): void {
    this.sidebarPanels.set(panel.id, panel)
    app.events.emit('workspace:sidebar-panel-registered', panel)
    console.log('[Workspace] Registered sidebar panel:', panel.name)
  }

  getSidebarPanels(): SidebarPanel[] {
    return Array.from(this.sidebarPanels.values())
  }

  clearPanels(): void {
    this.sidebarPanels.clear()
    this.settingTabs.clear()
    console.log('[Workspace] Cleared all panels')
  }

  openSidebarPanel(panelId: string): void {
    const panel = this.sidebarPanels.get(panelId)
    if (panel) {
      app.events.emit('workspace:sidebar-panel-open', panel)
      console.log('[Workspace] Opening sidebar panel:', panel.name)
    }
  }

  registerSettingTab(tab: PluginSettingTab): void {
    this.settingTabs.set(tab.id, tab)
    app.events.emit('workspace:setting-tab-registered', tab)
    console.log('[Workspace] Registered setting tab:', tab.name)
  }

  createPopover(options: PopoverOptions): HTMLElement {
    const popover = document.createElement('div')
    popover.className = 'roc-plugin-popover'
    popover.style.cssText = `
      position: fixed;
      background: var(--roc-bg-primary);
      border: 1px solid var(--roc-border);
      border-radius: 8px;
      box-shadow: 0 4px 20px rgba(0,0,0,0.2);
      padding: 12px;
      z-index: 1000;
      min-width: 200px;
    `

    if (options.title) {
      const title = document.createElement('div')
      title.className = 'roc-plugin-popover-title'
      title.style.cssText = `
        font-size: 13px;
        font-weight: 600;
        margin-bottom: 8px;
        padding-bottom: 8px;
        border-bottom: 1px solid var(--roc-border);
      `
      title.textContent = options.title
      popover.appendChild(title)
    }

    const content = document.createElement('div')
    content.className = 'roc-plugin-popover-content'
    content.style.cssText = 'font-size: 13px; line-height: 1.5;'
    
    if (typeof options.content === 'string') {
      content.innerHTML = options.content
    } else {
      options.content(content)
    }
    popover.appendChild(content)

    if (options.closable !== false) {
      const closeBtn = document.createElement('button')
      closeBtn.textContent = '×'
      closeBtn.style.cssText = `
        position: absolute;
        top: 4px;
        right: 8px;
        border: none;
        background: transparent;
        font-size: 18px;
        cursor: pointer;
        color: var(--roc-text-muted);
      `
      closeBtn.addEventListener('click', () => this.removeElement(popover, options.onClose))
      popover.appendChild(closeBtn)
    }

    if (options.target) {
      const rect = options.target.getBoundingClientRect()
      const pos = options.position || 'bottom'
      if (pos === 'bottom') {
        popover.style.left = `${rect.left}px`
        popover.style.top = `${rect.bottom + 8}px`
      } else if (pos === 'top') {
        popover.style.left = `${rect.left}px`
        popover.style.top = `${rect.top - popover.offsetHeight - 8}px`
      } else if (pos === 'right') {
        popover.style.left = `${rect.right + 8}px`
        popover.style.top = `${rect.top}px`
      } else {
        popover.style.left = `${rect.left - popover.offsetWidth - 8}px`
        popover.style.top = `${rect.top}px`
      }
    } else {
      popover.style.left = '50%'
      popover.style.top = '50%'
      popover.style.transform = 'translate(-50%, -50%)'
    }

    document.body.appendChild(popover)
    return popover
  }

  createModal(options: ModalOptions): HTMLElement {
    const overlay = document.createElement('div')
    overlay.className = 'roc-plugin-modal-overlay'
    overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0,0,0,0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 2000;
    `

    const modal = document.createElement('div')
    modal.className = 'roc-plugin-modal'
    modal.style.cssText = `
      background: var(--roc-bg-primary);
      border: 1px solid var(--roc-border);
      border-radius: 8px;
      box-shadow: 0 10px 40px rgba(0,0,0,0.3);
      width: ${options.width || '400px'};
      max-height: 80vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    `

    if (options.title) {
      const header = document.createElement('div')
      header.style.cssText = `
        padding: 16px;
        border-bottom: 1px solid var(--roc-border);
        font-size: 14px;
        font-weight: 600;
      `
      header.textContent = options.title
      modal.appendChild(header)
    }

    const content = document.createElement('div')
    content.style.cssText = `
      padding: 16px;
      overflow-y: auto;
      flex: 1;
      font-size: 13px;
      line-height: 1.5;
    `
    
    if (typeof options.content === 'string') {
      content.innerHTML = options.content
    } else {
      options.content(content)
    }
    modal.appendChild(content)

    if (options.confirmText || options.cancelText) {
      const footer = document.createElement('div')
      footer.style.cssText = `
        padding: 12px 16px;
        border-top: 1px solid var(--roc-border);
        display: flex;
        justify-content: flex-end;
        gap: 8px;
      `

      if (options.cancelText) {
        const cancelBtn = document.createElement('button')
        cancelBtn.textContent = options.cancelText
        cancelBtn.style.cssText = `
          padding: 6px 16px;
          border: 1px solid var(--roc-border);
          border-radius: 4px;
          background: transparent;
          cursor: pointer;
          font-size: 13px;
        `
        cancelBtn.addEventListener('click', () => this.removeElement(overlay, options.onCancel))
        footer.appendChild(cancelBtn)
      }

      if (options.confirmText) {
        const confirmBtn = document.createElement('button')
        confirmBtn.textContent = options.confirmText
        confirmBtn.style.cssText = `
          padding: 6px 16px;
          border: none;
          border-radius: 4px;
          background: var(--roc-accent);
          color: white;
          cursor: pointer;
          font-size: 13px;
        `
        confirmBtn.addEventListener('click', () => {
          options.onConfirm?.()
          this.removeElement(overlay)
        })
        footer.appendChild(confirmBtn)
      }

      modal.appendChild(footer)
    }

    if (options.closable !== false) {
      overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
          this.removeElement(overlay, options.onCancel)
        }
      })
    }

    overlay.appendChild(modal)
    document.body.appendChild(overlay)
    return modal
  }

  private removeElement(element: HTMLElement, callback?: () => void): void {
    element.remove()
    callback?.()
  }

  openCommandPalette(): void {
    useSearchStore().open()
  }
}

/**
 * MetadataCache 实现
 * 基于 notesStore 和 Tauri 后端提供元数据能力
 */
class MetadataCacheImpl implements MetadataCache {
  getFileCache(filePath: string): FileCache | null {
    const note = useNotesStore().notes.find((n) => n.path === filePath)
    if (!note) return null

    return {
      path: note.path,
      title: note.title,
      tags: note.tags,
      links: [],
      headings: [],
    }
  }

  async resolveWikilink(target: string, sourcePath?: string): Promise<string | null> {
    return resolveWikilink(target)
  }

  async getBacklinksForFile(filePath: string): Promise<Backlink[]> {
    return getBacklinks(filePath)
  }

  async getTags(): Promise<string[]> {
    return getAllTags()
  }
}

/**
 * Commands 实现
 * 提供命令注册和执行能力
 */
class CommandsImpl implements Commands {
  private commands = new Map<string, Command>()

  addCommand(command: Command): void {
    this.commands.set(command.id, command)
    console.log('[Commands] Registered command:', command.name)
  }

  executeCommand(commandId: string): void {
    const command = this.commands.get(commandId)
    if (command) {
      try {
        command.callback()
      } catch (error) {
        console.error(`[Commands] Error executing "${commandId}":`, error)
      }
    }
  }

  getCommands(): Command[] {
    return Array.from(this.commands.values())
  }

  clearCommands(): void {
    this.commands.clear()
    console.log('[Commands] Cleared all commands')
  }
}

/**
 * App 实现
 * 应用核心对象，整合所有子系统
 */
export class App implements AppInterface {
  name = 'Roc'
  version = '0.1.0'

  vault: Vault = new VaultImpl()
  workspace: Workspace = new WorkspaceImpl()
  metadataCache: MetadataCache = new MetadataCacheImpl()
  commands: Commands = new CommandsImpl()
  events: Events = new EventEmitter()

  private plugins = new Map<string, PluginEntry>()

  /**
   * 注册插件
   * @param pluginClass 插件类（必须有静态的 manifest 属性）
   */
  registerPlugin(
    pluginClass: new (app: App) => Plugin & { manifest: PluginManifest }
  ): void {
    const manifest = (pluginClass as any).manifest
    if (!manifest || !manifest.id) {
      console.error('[App] Plugin class must have a static manifest property')
      return
    }

    if (this.plugins.has(manifest.id)) {
      console.warn(`[App] Plugin "${manifest.id}" is already registered`)
      return
    }

    this.plugins.set(manifest.id, {
      pluginClass: pluginClass as any,
      manifest,
      enabled: false,
    })

    console.log(`[App] Registered plugin: ${manifest.name} (${manifest.id})`)
  }

  /**
   * 启用插件
   * @param pluginId 插件ID
   */
  enablePlugin(pluginId: string): void {
    const entry = this.plugins.get(pluginId)
    if (!entry) {
      console.error(`[App] Plugin "${pluginId}" not found`)
      return
    }

    if (entry.enabled) {
      console.warn(`[App] Plugin "${pluginId}" is already enabled`)
      return
    }

    try {
      const plugin = new entry.pluginClass(this)
      entry.instance = plugin
      entry.enabled = true

      plugin.onload()

      console.log(`[App] Enabled plugin: ${entry.manifest.name}`)
    } catch (error) {
      console.error(`[App] Failed to enable plugin "${pluginId}":`, error)
      entry.enabled = false
      entry.instance = undefined
    }
  }

  /**
   * 禁用插件
   * @param pluginId 插件ID
   */
  disablePlugin(pluginId: string): void {
    const entry = this.plugins.get(pluginId)
    if (!entry) {
      console.error(`[App] Plugin "${pluginId}" not found`)
      return
    }

    if (!entry.enabled || !entry.instance) {
      console.warn(`[App] Plugin "${pluginId}" is not enabled`)
      return
    }

    try {
      entry.instance.unload()
      entry.enabled = false
      entry.instance = undefined

      console.log(`[App] Disabled plugin: ${entry.manifest.name}`)
    } catch (error) {
      console.error(`[App] Failed to disable plugin "${pluginId}":`, error)
    }
  }

  /**
   * 获取插件实例
   * @param pluginId 插件ID
   */
  getPlugin(pluginId: string): Plugin | null {
    return this.plugins.get(pluginId)?.instance || null
  }

  /**
   * 获取所有已注册插件
   */
  getPlugins(): PluginEntry[] {
    return Array.from(this.plugins.values())
  }

  /**
   * 启用所有已注册插件
   */
  enableAllPlugins(): void {
    this.plugins.forEach((entry, pluginId) => {
      if (!entry.enabled) {
        this.enablePlugin(pluginId)
      }
    })
  }

  /**
   * 清空所有插件注册的资源（面板、命令等）
   * 用于切换 vault 时清理旧插件的资源
   */
  clearPluginResources(): void {
    this.workspace.clearPanels()
    this.commands.clearCommands()
    console.log('[App] Cleared all plugin resources')
  }

  /**
   * 动态加载外部插件
   * 支持从 URL 或本地文件加载插件模块
   * @param module 插件模块对象（应包含 default 导出或直接导出插件类）
   * @returns 加载的插件实例，如果加载失败返回 null
   */
  async loadExternalPlugin(module: any): Promise<Plugin | null> {
    try {
      const pluginClass = module.default || module
      
      if (typeof pluginClass !== 'function') {
        console.error('[App] External plugin module must export a class')
        return null
      }

      const manifest = (pluginClass as any).manifest
      if (!manifest || !manifest.id) {
        console.error('[App] External plugin must have a static manifest property')
        return null
      }

      if (this.plugins.has(manifest.id)) {
        console.warn(`[App] External plugin "${manifest.id}" is already registered`)
        return null
      }

      this.registerPlugin(pluginClass)
      this.enablePlugin(manifest.id)

      console.log(`[App] Loaded external plugin: ${manifest.name}`)
      return this.getPlugin(manifest.id)
    } catch (error) {
      console.error('[App] Failed to load external plugin:', error)
      return null
    }
  }

  /**
   * 从 URL 加载外部插件
   * @param url 插件模块的 URL
   * @param options 加载选项
   * @returns 加载的插件实例，如果加载失败返回 null
   */
  async loadPluginFromUrl(
    url: string,
    options: { timeout?: number } = {}
  ): Promise<Plugin | null> {
    const { timeout = 10000 } = options

    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), timeout)

      const response = await fetch(url, { signal: controller.signal })
      clearTimeout(timeoutId)

      if (!response.ok) {
        throw new Error(`Failed to fetch plugin: ${response.status}`)
      }

      const module = await response.json()
      return this.loadExternalPlugin(module)
    } catch (error) {
      console.error('[App] Failed to load plugin from URL:', error)
      return null
    }
  }

  /**
   * 从本地文件路径加载外部插件
   * @param filePath 插件文件路径
   * @returns 加载的插件实例，如果加载失败返回 null
   */
  async loadPluginFromFile(filePath: string): Promise<Plugin | null> {
    try {
      const module = await import(filePath)
      return this.loadExternalPlugin(module)
    } catch (error) {
      console.error('[App] Failed to load plugin from file:', error)
      return null
    }
  }
}

/**
 * 创建全局唯一的应用实例
 */
export const app = new App()

/**
 * Vue组合式函数，用于在组件中访问应用实例
 */
export function useApp() {
  return app
}
