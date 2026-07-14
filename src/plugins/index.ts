/**
 * 插件系统入口
 * 导出插件基类、类型定义、内置插件和加载工具
 * 
 * 使用方式：
 * 
 * ```typescript
 * import { Plugin, Component, rocApp, pluginLoader } from './plugins'
 * import { WikilinkPlugin, GraphPlugin, TagsPlugin } from './plugins'
 * 
 * // 注册插件
 * rocApp.registerPlugin(WikilinkPlugin)
 * rocApp.registerPlugin(GraphPlugin)
 * 
 * // 启用插件
 * rocApp.enablePlugin('roc-wikilink')
 * 
 * // 动态加载第三方插件
 * const plugin = await pluginLoader.loadPluginFromURL('https://example.com/plugin.js')
 * 
 * // 从本地文件加载
 * const localPlugin = await rocApp.loadPluginFromFile('./my-plugin.js')
 * ```
 */

// 导出核心类型和基类
export { Component, Plugin, type Command, type Hotkey } from './types'
export { type PluginManifest, createManifest } from './manifest'
import { app as rocApp, useApp as useRocApp, App } from './registry'
export { rocApp, useRocApp, App }

// 导出插件加载器
export { pluginLoader, PluginLoader, type PluginPackage, type PluginLoadOptions } from './loader'

// 导出内置插件
export { WikilinkPlugin } from './wikilink'
export { GraphPlugin } from './graph'
export { TagsPlugin } from './tags'

// 导出示例插件（供第三方开发者参考）
export { HelloPlugin } from './examples/HelloPlugin'

import { WikilinkPlugin } from './wikilink'
import { GraphPlugin } from './graph'
import { TagsPlugin } from './tags'

/**
 * 注册所有内置插件
 */
export function registerBuiltinPlugins(): void {
  rocApp.registerPlugin(WikilinkPlugin)
  rocApp.registerPlugin(GraphPlugin)
  rocApp.registerPlugin(TagsPlugin)
}

/**
 * 启用所有内置插件
 */
export function enableBuiltinPlugins(): void {
  rocApp.enableAllPlugins()
}
