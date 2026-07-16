import { Plugin } from './types'
import { createManifest, type PluginManifest } from './manifest'
import type { App } from './app'

/**
 * Wikilink 插件
 * 提供 wikilink 语法支持（[[笔记名]]）
 * 
 * 功能：
 * - 在编辑器中渲染 wikilink 为可点击的链接
 * - 支持自动补全功能
 * - 点击 wikilink 打开对应笔记
 */
export class WikilinkPlugin extends Plugin {
  static manifest: PluginManifest = createManifest(
    'roc-wikilink',
    'Wikilink',
    '1.0.0',
    '提供 [[笔记名]] 语法支持和自动补全'
  )

  constructor(app: App) {
    super(app)
  }

  onload(): void {
    console.log('[WikilinkPlugin] Loading...')

    // 注册命令
    this.registerCommand({
      id: 'roc-wikilink-insert',
      name: '插入 Wikilink',
      desc: '在当前位置插入 [[ ]] 语法',
      callback: () => {
        this.insertWikilink()
      },
    })

    // 注册快捷键
    this.registerHotkey({
      key: '[',
      ctrl: true,
      callback: () => {
        this.insertWikilink()
      },
    })

    console.log('[WikilinkPlugin] Loaded')
  }

  onunload(): void {
    console.log('[WikilinkPlugin] Unloaded')
  }

  /**
   * 注册命令（简化的包装方法）
   */
  private registerCommand(command: {
    id: string
    name: string
    desc?: string
    callback: () => void
  }): void {
    this.app.commands.addCommand({
      id: command.id,
      name: command.name,
      desc: command.desc,
      callback: command.callback,
    })
  }

  /**
   * 注册快捷键（简化的包装方法）
   */
  private registerHotkey(hotkey: {
    key: string
    ctrl?: boolean
    alt?: boolean
    shift?: boolean
    meta?: boolean
    callback: () => void
  }): void {
    const commandId = `hotkey-${hotkey.key}`
    this.app.commands.addCommand({
      id: commandId,
      name: `快捷键 ${hotkey.key}`,
      hotkeys: [{
        key: hotkey.key,
        ctrl: hotkey.ctrl,
        alt: hotkey.alt,
        shift: hotkey.shift,
        meta: hotkey.meta,
      }],
      callback: hotkey.callback,
    })
  }

  /**
   * 插入 Wikilink 语法
   */
  private insertWikilink(): void {
    // 这里可以集成到编辑器中
    console.log('[WikilinkPlugin] Inserting wikilink')
  }
}
