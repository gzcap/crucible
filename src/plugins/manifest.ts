/**
 * 插件清单接口
 * 定义插件的元数据和配置信息
 * 
 * 遵循 Obsidian manifest.json 格式
 */
export interface PluginManifest {
  /** 插件唯一标识符（反向域名格式） */
  id: string
  /** 插件显示名称 */
  name: string
  /** 插件版本号 */
  version: string
  /** 插件作者 */
  author?: string
  /** 插件描述 */
  description?: string
  /** 插件官网地址 */
  authorUrl?: string
  /** 插件仓库地址 */
  repo?: string
  /** 支持的最低应用版本 */
  minAppVersion?: string
  /** 是否为付费插件 */
  isDesktopOnly?: boolean
}

/**
 * 创建内置插件清单的辅助函数
 */
export function createManifest(
  id: string,
  name: string,
  version: string = '1.0.0',
  description?: string
): PluginManifest {
  return {
    id,
    name,
    version,
    description,
  }
}
