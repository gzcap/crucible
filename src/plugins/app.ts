import type { Plugin } from './types'
import type { PluginManifest } from './manifest'

/**
 * Vault 接口
 * 提供文件系统操作能力
 */
export interface Vault {
  /** 当前打开的仓库路径 */
  path: string | null
  /** 获取所有笔记文件 */
  getMarkdownFiles(): Promise<string[]>
  /** 读取文件内容 */
  read(filePath: string): Promise<string>
  /** 写入文件内容 */
  write(filePath: string, content: string): Promise<void>
  /** 创建新文件 */
  create(filePath: string, content?: string): Promise<void>
  /** 删除文件 */
  delete(filePath: string): Promise<void>
  /** 创建文件夹 */
  createFolder(folderPath: string): Promise<void>
  /** 重命名文件 */
  rename(oldPath: string, newPath: string): Promise<void>
}

/**
 * Workspace 接口
 * 提供界面布局和视图管理能力
 */
export interface Workspace {
  /** 当前活动文件路径 */
  activeFilePath: string | null
  /** 打开文件 */
  openFile(filePath: string): void
  /** 关闭文件 */
  closeFile(filePath: string): void
  /** 创建新标签页 */
  createTab(filePath: string): void
  /** 切换到指定视图 */
  switchView(viewType: string): void
  /** 注册侧边栏面板 */
  registerSidebarPanel(panel: SidebarPanel): void
  /** 获取所有已注册的侧边栏面板 */
  getSidebarPanels(): SidebarPanel[]
  /** 打开侧边栏面板 */
  openSidebarPanel(panelId: string): void
  /** 注册设置面板 */
  registerSettingTab(tab: PluginSettingTab): void
  /** 创建浮动面板 */
  createPopover(options: PopoverOptions): HTMLElement
  /** 创建模态对话框 */
  createModal(options: ModalOptions): HTMLElement
  /** 打开命令面板 */
  openCommandPalette(): void
  /** 清空所有面板（切换 vault 时使用） */
  clearPanels(): void
}

/**
 * 侧边栏面板配置
 */
export interface SidebarPanel {
  /** 面板唯一标识符 */
  id: string
  /** 面板显示名称 */
  name: string
  /** 面板图标（emoji 或图标名称） */
  icon: string
  /** 面板内容组件（Vue 组件或 HTML 字符串） */
  component: any
  /** 面板内容渲染函数 */
  render?: (container: HTMLElement) => void
}

/**
 * 设置面板配置
 */
export interface PluginSettingTab {
  /** 设置面板唯一标识符 */
  id: string
  /** 设置面板名称 */
  name: string
  /** 设置面板图标 */
  icon?: string
  /** 渲染设置面板内容 */
  render(container: HTMLElement): void
  /** 设置面板显示时调用 */
  display?(): void
  /** 设置面板隐藏时调用 */
  hide?(): void
}

/**
 * 浮动面板配置选项
 */
export interface PopoverOptions {
  /** 面板标题 */
  title?: string
  /** 面板内容（HTML 字符串或渲染函数） */
  content: string | ((container: HTMLElement) => void)
  /** 定位目标元素 */
  target?: HTMLElement
  /** 位置：top, bottom, left, right */
  position?: 'top' | 'bottom' | 'left' | 'right'
  /** 是否可关闭 */
  closable?: boolean
  /** 关闭回调 */
  onClose?: () => void
}

/**
 * 模态对话框配置选项
 */
export interface ModalOptions {
  /** 对话框标题 */
  title?: string
  /** 对话框内容（HTML 字符串或渲染函数） */
  content: string | ((container: HTMLElement) => void)
  /** 对话框宽度 */
  width?: string
  /** 是否可关闭 */
  closable?: boolean
  /** 确认按钮文本 */
  confirmText?: string
  /** 取消按钮文本 */
  cancelText?: string
  /** 确认回调 */
  onConfirm?: () => void
  /** 取消回调 */
  onCancel?: () => void
}

/**
 * MetadataCache 接口
 * 提供链接解析和元数据管理能力
 */
export interface MetadataCache {
  /** 获取文件元数据 */
  getFileCache(filePath: string): FileCache | null
  /** 解析 wikilink 目标 */
  resolveWikilink(target: string, sourcePath?: string): Promise<string | null>
  /** 获取所有反向链接 */
  getBacklinksForFile(filePath: string): Promise<Backlink[]>
  /** 获取所有标签 */
  getTags(): Promise<string[]>
}

/**
 * 文件缓存信息
 */
export interface FileCache {
  /** 文件路径 */
  path: string
  /** 文件标题 */
  title: string
  /** 文件标签 */
  tags: string[]
  /** 文件中的链接 */
  links: LinkCache[]
  /** 文件标题列表 */
  headings: HeadingCache[]
}

/**
 * 链接缓存信息
 */
export interface LinkCache {
  /** 链接目标 */
  link: string
  /** 链接位置 */
  position: Position
}

/**
 * 标题缓存信息
 */
export interface HeadingCache {
  /** 标题文本 */
  heading: string
  /** 标题级别 */
  level: number
  /** 标题位置 */
  position: Position
}

/**
 * 反向链接信息
 */
export interface Backlink {
  /** 源文件路径 */
  source: string
  /** 源文件标题 */
  source_title: string
  /** 包含链接的行文本 */
  snippet: string
}

/**
 * 位置信息
 */
export interface Position {
  start: { line: number; column: number }
  end: { line: number; column: number }
}

/**
 * Commands 接口
 * 提供命令注册和执行能力
 */
export interface Commands {
  /** 注册命令 */
  addCommand(command: Command): void
  /** 执行命令 */
  executeCommand(commandId: string): void
  /** 获取所有已注册命令 */
  getCommands(): Command[]
  /** 清空所有命令（切换 vault 时使用） */
  clearCommands(): void
}

/**
 * 命令接口
 */
export interface Command {
  id: string
  name: string
  desc?: string
  hotkeys?: Hotkey[]
  callback: () => void
  checkCallback?: (checking: boolean) => boolean
}

/**
 * 快捷键接口
 */
export interface Hotkey {
  key: string
  ctrl?: boolean
  alt?: boolean
  shift?: boolean
  meta?: boolean
}

/**
 * Events 接口
 * 提供事件订阅能力
 */
export interface Events {
  /** 订阅事件 */
  on(event: string, callback: (...args: any[]) => void): void
  /** 取消订阅事件 */
  off(event: string, callback: (...args: any[]) => void): void
  /** 触发事件 */
  emit(event: string, ...args: any[]): void
}

/**
 * App 接口
 * 应用核心对象，提供对所有子系统的访问
 * 
 * 插件通过 this.app 访问应用功能
 */
export interface App {
  /** 应用名称 */
  name: string
  /** 应用版本 */
  version: string
  /** 文件系统操作 */
  vault: Vault
  /** 界面布局管理 */
  workspace: Workspace
  /** 元数据缓存 */
  metadataCache: MetadataCache
  /** 命令系统 */
  commands: Commands
  /** 事件系统 */
  events: Events
  /**
   * 注册插件
   * @param pluginClass 插件类（必须有静态的 manifest 属性）
   */
  registerPlugin(pluginClass: new (app: App) => Plugin): void
  /**
   * 启用插件
   * @param pluginId 插件ID
   */
  enablePlugin(pluginId: string): void
  /**
   * 禁用插件
   * @param pluginId 插件ID
   */
  disablePlugin(pluginId: string): void
  /**
   * 获取插件实例
   * @param pluginId 插件ID
   */
  getPlugin(pluginId: string): Plugin | null
}
