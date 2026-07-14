import { Plugin } from './types'
import { createManifest, type PluginManifest } from './manifest'
import type { App } from './app'

/**
 * Tags 插件
 * 提供标签管理和搜索功能
 * 
 * 功能：
 * - 解析和索引笔记中的标签
 * - 提供标签云展示
 * - 支持按标签搜索笔记
 */
export class TagsPlugin extends Plugin {
  static manifest: PluginManifest = createManifest(
    'roc-tags',
    'Tags',
    '1.0.0',
    '提供标签管理和搜索功能'
  )

  constructor(app: App) {
    super(app)
  }

  onload(): void {
    console.log('[TagsPlugin] Loading...')

    // 注册命令
    this.app.commands.addCommand({
      id: 'roc-tags-search',
      name: '搜索标签',
      desc: '按标签搜索笔记',
      callback: () => {
        this.searchTags()
      },
    })

    // 订阅文件变更事件，更新标签索引
    this.registerEvent(this.app.events, 'roc://file-changed', () => {
      this.updateTags()
    })

    console.log('[TagsPlugin] Loaded')
  }

  onunload(): void {
    console.log('[TagsPlugin] Unloaded')
  }

  /**
   * 搜索标签
   */
  private searchTags(): void {
    this.app.workspace.openCommandPalette()
  }

  /**
   * 更新标签索引
   */
  private async updateTags(): Promise<void> {
    const tags = await this.app.metadataCache.getTags()
    console.log('[TagsPlugin] Updated tags:', tags.length)
  }
}
