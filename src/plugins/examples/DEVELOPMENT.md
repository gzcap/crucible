# Roc 插件开发指南

## 概述

Roc 的插件系统完全仿照 Obsidian 的插件机制设计，提供了相同的 API 风格和开发体验。插件可以扩展应用功能，包括：

- 注册命令和快捷键
- 添加侧边栏面板
- 监听应用事件
- 修改编辑器行为
- 访问文件系统

## 快速开始

### 创建插件

创建一个新的插件类，继承 `Plugin` 基类：

```typescript
import { Plugin } from '@roc/plugins'
import { createManifest } from '@roc/plugins'

export class MyPlugin extends Plugin {
  static manifest = createManifest(
    'my-plugin-id',
    'My Plugin',
    '1.0.0',
    '插件描述信息'
  )

  onload(): void {
    // 插件加载时执行的初始化逻辑
    console.log('插件已加载')
    
    // 注册命令
    this.app.commands.addCommand({
      id: 'my-plugin-do-something',
      name: 'Do Something',
      callback: () => {
        // 执行命令逻辑
      }
    })
  }

  onunload(): void {
    // 插件卸载时的清理逻辑（可选）
    console.log('插件已卸载')
  }
}
```

### 注册和启用

在应用中注册和启用插件：

```typescript
import { rocApp, MyPlugin } from '@roc/plugins'

// 注册插件
rocApp.registerPlugin(MyPlugin)

// 启用插件
rocApp.enablePlugin('my-plugin-id')
```

## API 参考

### Plugin 基类

#### 生命周期方法

- `onload()` - 插件加载时调用，在此注册所有功能
- `onunload()` - 插件卸载时调用，手动清理资源

#### 数据持久化

- `loadData<T>()` - 从 localStorage 加载插件配置
- `saveData(data)` - 将配置保存到 localStorage

#### 组件管理

继承自 `Component` 基类，提供资源自动清理机制：

- `register(cleanup)` - 注册清理回调
- `registerEvent(source, name, callback)` - 注册事件监听
- `registerDomEvent(element, name, callback)` - 注册 DOM 事件
- `registerInterval(callback, delay)` - 注册定时器
- `addChild(child)` - 添加子组件

### App 对象

通过 `this.app` 访问应用核心功能：

#### vault - 文件系统

```typescript
// 获取 vault 路径
const path = this.app.vault.path

// 读取文件
const content = await this.app.vault.read('note.md')

// 写入文件
await this.app.vault.write('note.md', '# Hello')

// 创建文件
await this.app.vault.create('new-note.md')

// 删除文件
await this.app.vault.delete('note.md')

// 获取所有 markdown 文件
const files = await this.app.vault.getMarkdownFiles()
```

#### workspace - 界面管理

```typescript
// 获取当前打开的文件
const activeFile = this.app.workspace.activeFilePath

// 打开文件
this.app.workspace.openFile('note.md')

// 创建新标签页
this.app.workspace.createTab('note.md')

// 打开命令面板
this.app.workspace.openCommandPalette()
```

#### commands - 命令系统

```typescript
// 添加命令
this.app.commands.addCommand({
  id: 'my-command',
  name: 'My Command',
  desc: '命令描述',
  callback: () => {
    // 命令逻辑
  }
})

// 执行命令
this.app.commands.executeCommand('my-command')

// 获取所有命令
const commands = this.app.commands.getCommands()
```

#### metadataCache - 元数据缓存

```typescript
// 获取文件元数据
const cache = this.app.metadataCache.getFileCache('note.md')

// 解析 wikilink
const resolved = await this.app.metadataCache.resolveWikilink('Target Note')

// 获取文件的反向链接
const backlinks = await this.app.metadataCache.getBacklinksForFile('note.md')

// 获取所有标签
const tags = await this.app.metadataCache.getTags()
```

#### events - 事件系统

```typescript
// 监听事件
this.app.events.on('app:mounted', () => {
  console.log('应用已挂载')
})

// 触发事件
this.app.events.emit('my-plugin:event', data)

// 使用 registerEvent 自动清理
this.registerEvent(this.app.events, 'file:changed', (event) => {
  // 事件处理
})
```

## 第三方插件导入

### 方式一：通过 PluginLoader 动态加载

```typescript
import { pluginLoader } from '@roc/plugins'

// 从 URL 加载
const plugin = await pluginLoader.loadPluginFromURL(
  'https://example.com/my-plugin.js',
  { timeout: 10000 }
)

// 从本地文件加载
const localPlugin = await pluginLoader.loadPluginFromFile('./plugins/my-plugin.js')

// 从目录加载多个插件
const plugins = await pluginLoader.loadPluginsFromDirectory('./plugins')
```

### 方式二：通过 App 直接加载

```typescript
import { rocApp } from '@roc/plugins'

// 从 URL 加载
const plugin = await rocApp.loadPluginFromUrl('https://example.com/my-plugin.js')

// 从文件加载
const localPlugin = await rocApp.loadPluginFromFile('./my-plugin.js')
```

### 方式三：静态导入

```typescript
import { rocApp, MyPlugin } from './my-plugin'

rocApp.registerPlugin(MyPlugin)
rocApp.enablePlugin('my-plugin-id')
```

## 插件包格式

第三方插件需要导出一个符合以下格式的模块：

```typescript
// my-plugin.js
import { Plugin } from '@roc/plugins'
import { createManifest } from '@roc/plugins'

class MyPlugin extends Plugin {
  static manifest = createManifest('my-plugin', 'My Plugin', '1.0.0', 'Description')
  
  onload() {
    // 插件逻辑
  }
}

export default MyPlugin
```

## 示例插件

查看 [HelloPlugin](examples/HelloPlugin.ts) 了解完整的插件示例。

## 最佳实践

### 资源清理

使用 `register` 方法注册所有需要清理的资源，确保插件卸载时自动释放：

```typescript
onload(): void {
  const button = document.createElement('button')
  this.registerDomEvent(button, 'click', () => {})
  
  const interval = this.registerInterval(() => {}, 1000)
  
  this.registerEvent(this.app.events, 'event', () => {})
  
  this.register(() => {
    // 自定义清理逻辑
    button.remove()
  })
}
```

### 错误处理

使用 try-catch 包裹可能失败的操作：

```typescript
onload(): void {
  try {
    // 初始化逻辑
  } catch (error) {
    console.error('[MyPlugin] Failed to load:', error)
  }
}
```

### 配置管理

使用 `loadData` 和 `saveData` 管理插件配置：

```typescript
interface MyPluginConfig {
  enabled: boolean
  options: string[]
}

onload(): void {
  this.loadData<MyPluginConfig>().then(config => {
    if (config) {
      // 使用配置
    }
  })
}

async updateConfig(newConfig: MyPluginConfig): Promise<void> {
  await this.saveData(newConfig)
}
```

## 事件列表

### 应用事件

- `app:mounted` - 应用挂载完成
- `app:unmounted` - 应用卸载

### 文件事件

- `file:created` - 文件创建
- `file:changed` - 文件修改
- `file:deleted` - 文件删除

### 编辑器事件

- `editor:open` - 编辑器打开
- `editor:close` - 编辑器关闭
- `editor:change` - 编辑器内容变化

### 命令事件

- `command:executed` - 命令执行
