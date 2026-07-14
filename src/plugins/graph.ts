import { Plugin } from './types'
import { createManifest, type PluginManifest } from './manifest'
import type { App } from './app'

/**
 * Graph 插件
 * 提供笔记关系图谱功能
 * 
 * 功能：
 * - 可视化显示笔记之间的链接关系
 * - 支持点击节点打开笔记
 * - 支持图谱布局调整
 */
export class GraphPlugin extends Plugin {
  static manifest: PluginManifest = createManifest(
    'roc-graph',
    'Graph View',
    '1.0.0',
    '提供笔记关系图谱可视化'
  )

  constructor(app: App) {
    super(app)
  }

  onload(): void {
    console.log('[GraphPlugin] Loading...')

    // 注册命令
    this.app.commands.addCommand({
      id: 'roc-graph-open',
      name: '打开关系图谱',
      desc: '在新标签页中打开关系图谱',
      callback: () => {
        this.openGraph()
      },
    })

    console.log('[GraphPlugin] Loaded')
  }

  onunload(): void {
    console.log('[GraphPlugin] Unloaded')
  }

  /**
   * 打开关系图谱
   */
  private openGraph(): void {
    this.app.workspace.switchView('graph')
  }
}
