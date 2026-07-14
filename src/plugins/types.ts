import type { PluginManifest } from './manifest'
import type { App } from './app'

/**
 * 组件基类
 * 提供资源注册和自动清理机制
 * 所有需要生命周期管理的功能都应继承此类
 */
export class Component {
  /** 子组件列表 */
  private children: Component[] = []
  /** 注册的清理函数列表 */
  private cleanupFunctions: (() => void)[] = []
  /** 是否已加载 */
  private _loaded = false

  /**
   * 组件加载时调用
   * 子类应在此方法中注册资源
   */
  onload(): void {
    this._loaded = true
  }

  /**
   * 组件卸载时调用
   * 自动清理所有注册的资源
   */
  onunload(): void {
    // 反向卸载子组件
    for (let i = this.children.length - 1; i >= 0; i--) {
      this.children[i].unload()
    }
    this.children = []

    // 执行所有清理函数
    for (const cleanup of this.cleanupFunctions) {
      try {
        cleanup()
      } catch (error) {
        console.error('[Component] Cleanup error:', error)
      }
    }
    this.cleanupFunctions = []

    this._loaded = false
  }

  /**
   * 安全卸载组件
   */
  unload(): void {
    if (this._loaded) {
      this.onunload()
    }
  }

  /**
   * 获取组件加载状态
   */
  get loaded(): boolean {
    return this._loaded
  }

  /**
   * 注册一个清理回调函数
   * 当组件卸载时自动执行
   * @param cleanup 清理函数
   */
  register(cleanup: () => void): void {
    this.cleanupFunctions.push(cleanup)
  }

  /**
   * 注册一个事件监听器
   * 组件卸载时自动移除监听
   * @param eventSource 事件源对象（必须有 off 方法）
   * @param eventName 事件名称
   * @param callback 回调函数
   */
  registerEvent<T>(
    eventSource: { on(eventName: string, callback: T): void; off(eventName: string, callback: T): void },
    eventName: string,
    callback: T
  ): void {
    eventSource.on(eventName, callback)
    this.register(() => {
      eventSource.off(eventName, callback)
    })
  }

  /**
   * 注册一个 DOM 事件监听器
   * 组件卸载时自动移除监听
   * @param element DOM 元素
   * @param eventName 事件名称
   * @param callback 事件处理函数
   * @param options 事件选项
   */
  registerDomEvent<K extends keyof HTMLElementEventMap>(
    element: HTMLElement,
    eventName: K,
    callback: (this: HTMLElement, event: HTMLElementEventMap[K]) => void,
    options?: boolean | AddEventListenerOptions
  ): void {
    element.addEventListener(eventName, callback, options)
    this.register(() => {
      element.removeEventListener(eventName, callback, options)
    })
  }

  /**
   * 注册一个定时器
   * 组件卸载时自动清除
   * @param callback 定时器回调
   * @param delay 延迟时间（毫秒）
   * @returns 定时器 ID
   */
  registerInterval(callback: () => void, delay: number): number {
    const id = window.setInterval(callback, delay)
    this.register(() => {
      window.clearInterval(id)
    })
    return id
  }

  /**
   * 注册一个子组件
   * 父组件卸载时自动卸载子组件
   * @param child 子组件
   */
  addChild(child: Component): void {
    this.children.push(child)
    if (this._loaded) {
      child.onload()
    }
  }

  /**
   * 移除子组件
   * @param child 子组件
   */
  removeChild(child: Component): void {
    const index = this.children.indexOf(child)
    if (index !== -1) {
      child.unload()
      this.children.splice(index, 1)
    }
  }
}

/**
 * 插件基类
 * 所有插件都应继承此类
 * 
 * 遵循 Obsidian 的插件设计模式：
 * - 通过 `this.app` 访问应用核心功能
 * - 在 `onload()` 中注册所有功能
 * - 在 `onunload()` 中手动清理资源（自动注册的资源会自动清理）
 */
export abstract class Plugin extends Component {
  /** 应用实例，提供对核心功能的访问 */
  app: App
  /** 插件清单，包含插件元数据 */
  manifest: PluginManifest

  /**
   * 创建插件实例
   * @param app 应用实例
   */
  constructor(app: App) {
    super()
    this.app = app
    // 从类的静态属性获取 manifest
    const cls = this.constructor as typeof Plugin
    this.manifest = cls.manifest
  }

  /** 插件清单（静态属性，由子类定义） */
  static manifest: PluginManifest

  /**
   * 加载插件数据
   * 从 localStorage 读取插件的持久化配置
   * @returns 插件数据对象
   */
  async loadData<T = unknown>(): Promise<T | null> {
    const key = `roc-plugin-data-${this.manifest.id}`
    const data = localStorage.getItem(key)
    return data ? JSON.parse(data) : null
  }

  /**
   * 保存插件数据
   * 将插件配置持久化到 localStorage
   * @param data 要保存的数据对象
   */
  async saveData(data: unknown): Promise<void> {
    const key = `roc-plugin-data-${this.manifest.id}`
    localStorage.setItem(key, JSON.stringify(data))
  }
}

/**
 * 命令配置接口
 * 用于注册命令面板中的命令
 */
export interface Command {
  /** 命令唯一标识符 */
  id: string
  /** 命令显示名称 */
  name: string
  /** 命令描述（可选） */
  desc?: string
  /** 快捷键组合（可选） */
  hotkeys?: Hotkey[]
  /** 命令执行回调 */
  callback: () => void
  /** 是否仅在特定条件下可用（可选） */
  checkCallback?: (checking: boolean) => boolean
}

/**
 * 快捷键定义
 */
export interface Hotkey {
  /** 按键 */
  key: string
  /** 是否按下 Ctrl/Cmd */
  ctrl?: boolean
  /** 是否按下 Alt */
  alt?: boolean
  /** 是否按下 Shift */
  shift?: boolean
  /** 是否按下 Meta/Cmd */
  meta?: boolean
}

/**
 * 设置面板配置
 */
export interface PluginSettingTab {
  /** 设置面板名称 */
  name: string
  /** 设置面板图标（可选） */
  icon?: string
  /** 渲染设置面板内容 */
  render(container: HTMLElement): void
  /** 设置面板显示时调用 */
  display(): void
  /** 设置面板隐藏时调用 */
  hide(): void
}

/**
 * 侧边栏视图配置
 */
export interface ViewConfig {
  /** 视图类型 */
  type: string
  /** 视图名称 */
  name: string
  /** 视图图标 */
  icon: string
  /** 创建视图内容的函数 */
  create: () => HTMLElement | any
}

/**
 * Markdown 后处理器
 */
export interface MarkdownPostProcessor {
  /** 处理器优先级 */
  priority?: number
  /** 处理函数 */
  process(el: HTMLElement, ctx: MarkdownPostProcessorContext): void
}

/**
 * Markdown 后处理器上下文
 */
export interface MarkdownPostProcessorContext {
  /** 当前文件路径 */
  path: string
  /** 是否为预览模式 */
  preview: boolean
}
