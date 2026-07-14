import type { Plugin } from './types'
import type { PluginManifest } from './manifest'
import { app } from './registry'
import { readDir, readTextFile } from '@tauri-apps/plugin-fs'

export interface PluginPackage {
  manifest: PluginManifest
  default: new (app: any) => Plugin
}

export interface PluginLoadOptions {
  allowRemote?: boolean
  timeout?: number
}

export class PluginLoader {
  private loadedPlugins = new Set<string>()

  async loadPluginFromURL(
    url: string,
    options: PluginLoadOptions = {}
  ): Promise<Plugin | null> {
    const { allowRemote = true, timeout = 10000 } = options

    if (!allowRemote && !url.startsWith('http')) {
      console.error('[PluginLoader] Remote plugins are not allowed')
      return null
    }

    try {
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), timeout)

      const response = await fetch(url, { signal: controller.signal })
      clearTimeout(timeoutId)

      if (!response.ok) {
        throw new Error(`Failed to fetch plugin: ${response.status}`)
      }

      const module = await response.json()

      clearTimeout(timeoutId)
      return this.instantiatePlugin(module)
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugin from URL:', error)
      return null
    }
  }

  async loadPluginFromFile(filePath: string): Promise<Plugin | null> {
    try {
      const content = await readTextFile(filePath)
      return this.instantiatePluginFromString(content)
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugin from file:', error)
      return null
    }
  }

  private async instantiatePluginFromString(content: string): Promise<Plugin | null> {
    try {
      const module = await this.evalPluginModule(content)
      return this.instantiatePlugin(module)
    } catch (error) {
      console.error('[PluginLoader] Failed to evaluate plugin module:', error)
      return null
    }
  }

  private async evalPluginModule(content: string): Promise<any> {
    const blob = new Blob([content], { type: 'application/javascript' })
    const url = URL.createObjectURL(blob)
    
    try {
      const module = await import(url)
      return module
    } finally {
      URL.revokeObjectURL(url)
    }
  }

  private instantiatePlugin(module: any): Plugin | null {
    try {
      const pluginClass = module.default || module
      if (typeof pluginClass !== 'function') {
        console.error('[PluginLoader] Plugin module must export a class')
        return null
      }

      const manifest = (pluginClass as any).manifest
      if (!manifest || !manifest.id) {
        console.error('[PluginLoader] Plugin class must have a static manifest')
        return null
      }

      if (this.loadedPlugins.has(manifest.id)) {
        console.warn(`[PluginLoader] Plugin "${manifest.id}" is already loaded`)
        return null
      }

      app.registerPlugin(pluginClass)
      app.enablePlugin(manifest.id)

      this.loadedPlugins.add(manifest.id)
      return app.getPlugin(manifest.id)
    } catch (error) {
      console.error('[PluginLoader] Failed to instantiate plugin:', error)
      return null
    }
  }

  async loadPluginsFromDirectory(directory: string): Promise<Plugin[]> {
    const plugins: Plugin[] = []

    try {
      const files = await this.readDirectory(directory)

      for (const file of files) {
        if (file.endsWith('.js')) {
          const plugin = await this.loadPluginFromFile(`${directory}/${file}`)
          if (plugin) {
            plugins.push(plugin)
          }
        }
      }
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugins from directory:', error)
    }

    return plugins
  }

  private async readDirectory(directory: string): Promise<string[]> {
    try {
      const entries = await readDir(directory)
      return entries
        .filter((entry) => entry.isFile)
        .map((entry) => entry.name)
    } catch {
      return []
    }
  }

  unloadPlugin(pluginId: string): void {
    app.disablePlugin(pluginId)
    this.loadedPlugins.delete(pluginId)
  }

  isLoaded(pluginId: string): boolean {
    return this.loadedPlugins.has(pluginId)
  }
}

export const pluginLoader = new PluginLoader()