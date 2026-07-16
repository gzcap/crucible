import { Plugin } from '../types'
import { createManifest } from '../manifest'
import type { App } from '../app'

export class HelloPlugin extends Plugin {
  static manifest = createManifest(
    'roc-hello',
    'Hello Plugin',
    '1.0.0',
    '一个简单的示例插件，展示如何开发第三方插件'
  )

  onload(): void {
    console.log(`[${this.manifest.name}] 插件已加载`)

    this.app.commands.addCommand({
      id: 'roc-hello-say-hello',
      name: 'Say Hello',
      desc: '显示一个友好的问候消息',
      callback: () => {
        alert(`Hello from ${this.manifest.name}!`)
      },
    })

    this.registerEvent(this.app.events, 'app:mounted', () => {
      console.log(`[${this.manifest.name}] 应用已挂载`)
    })
  }

  onunload(): void {
    console.log(`[${this.manifest.name}] 插件已卸载`)
  }
}