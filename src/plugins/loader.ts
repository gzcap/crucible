import { Plugin } from './types'
import type { PluginManifest } from './manifest'
import { app } from './registry'
import { invoke } from '@tauri-apps/api/core'

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

  async initPluginSystem(): Promise<void> {
    try {
      await invoke('init_plugin_system')
      console.log('[PluginLoader] Plugin system initialized from backend')
    } catch (error) {
      console.error('[PluginLoader] Failed to initialize plugin system:', error)
    }
  }

  async loadPluginsFromBackend(): Promise<Plugin[]> {
    const plugins: Plugin[] = []

    try {
      await this.initPluginSystem()

      const manifests = await invoke<PluginManifest[]>('list_installed_plugins')

      for (const manifest of manifests) {
        try {
          const plugin = await this.loadPluginFromBackend(manifest)
          if (plugin) {
            plugins.push(plugin)
          }
        } catch (error) {
          console.error(`[PluginLoader] Failed to load plugin ${manifest.id}:`, error)
        }
      }
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugins from backend:', error)
    }

    return plugins
  }

  private async loadPluginFromBackend(manifest: PluginManifest): Promise<Plugin | null> {
    try {
      const mainContent = await invoke<string>('read_plugin_main', { pluginId: manifest.id })
      
      const module = await this.evalPluginModule(mainContent)
      return this.instantiatePluginWithManifest(module, manifest)
    } catch (error) {
      console.error(`[PluginLoader] Failed to load plugin ${manifest.id} from backend:`, error)
      return null
    }
  }

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
      return this.instantiatePlugin(module)
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugin from URL:', error)
      return null
    }
  }

  async loadPluginFromFile(filePath: string): Promise<Plugin | null> {
    try {
      const content = await invoke<string>('read_text_file', { filePath })
      return this.instantiatePluginFromString(content)
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugin from file:', error)
      return null
    }
  }

  async loadPluginFromDirectory(pluginDir: string): Promise<Plugin | null> {
    try {
      const manifestPath = `${pluginDir}/manifest.json`
      const mainPath = `${pluginDir}/main.js`

      const manifestContent = await invoke<string>('read_text_file', { filePath: manifestPath })
      const manifest = JSON.parse(manifestContent) as PluginManifest

      const mainContent = await invoke<string>('read_text_file', { filePath: mainPath })
      return this.instantiatePluginFromSource(mainContent, manifest)
    } catch (error) {
      console.error('[PluginLoader] Failed to load plugin from directory:', error)
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

  private async instantiatePluginFromSource(content: string, manifest: PluginManifest): Promise<Plugin | null> {
    try {
      const module = await this.evalPluginModule(content)
      return this.instantiatePluginWithManifest(module, manifest)
    } catch (error) {
      console.error('[PluginLoader] Failed to evaluate plugin module:', error)
      return null
    }
  }

  private async evalPluginModule(content: string): Promise<any> {
    try {
      let wrappedContent = content
      
      if (content.includes('export default')) {
        wrappedContent = content.replace(
          /export\s+default\s+(\w+)/g,
          'return $1'
        )
        const fn = new Function('Plugin', 'app', wrappedContent)
        return { default: fn(Plugin, app) }
      }
      
      if (content.includes('module.exports')) {
        const module = { exports: {} }
        const fn = new Function('module', 'exports', 'Plugin', 'app', wrappedContent)
        fn(module, module.exports, Plugin, app)
        return module.exports
      }
      
      const module = { exports: {} }
      const fn = new Function('exports', 'require', 'module', '__filename', '__dirname', 'Plugin', 'app', content)
      fn(module.exports, () => {}, module, '', '', Plugin, app)
      return module.exports
    } catch (error) {
      console.error('[PluginLoader] Failed to eval plugin module:', error)
      throw error
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

  private instantiatePluginWithManifest(module: any, manifest: PluginManifest): Plugin | null {
    try {
      const pluginClass = module.default || module
      if (typeof pluginClass !== 'function') {
        console.error('[PluginLoader] Plugin module must export a class')
        return null
      }

      if (!manifest.id) {
        console.error('[PluginLoader] Plugin manifest must have an id')
        return null
      }

      if (this.loadedPlugins.has(manifest.id)) {
        console.warn(`[PluginLoader] Plugin "${manifest.id}" is already loaded`)
        return null
      }

      (pluginClass as any).manifest = manifest

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
      const entries = await invoke<any[]>('list_dir', { dir: directory })

      for (const entry of entries) {
        if (entry.isDirectory) {
          const pluginDir = `${directory}/${entry.name}`
          const manifestPath = `${pluginDir}/manifest.json`
          const mainPath = `${pluginDir}/main.js`

          try {
            await invoke<string>('read_text_file', { filePath: manifestPath })
            await invoke<string>('read_text_file', { filePath: mainPath })

            const plugin = await this.loadPluginFromDirectory(pluginDir)
            if (plugin) {
              plugins.push(plugin)
            }
          } catch {
            continue
          }
        } else if (entry.name.endsWith('.js')) {
          const plugin = await this.loadPluginFromFile(`${directory}/${entry.name}`)
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

  async enablePlugin(pluginId: string): Promise<void> {
    try {
      await invoke('enable_plugin', { pluginId })
      
      const mainContent = await invoke<string>('read_plugin_main', { pluginId })
      const module = await this.evalPluginModule(mainContent)
      
      const pluginClass = module.default || module
      if (typeof pluginClass === 'function') {
        (pluginClass as any).manifest = { id: pluginId }
        app.registerPlugin(pluginClass)
        app.enablePlugin(pluginId)
        
        const pluginInstance = app.getPlugin(pluginId)
        if (pluginInstance && typeof pluginInstance.onload === 'function') {
          await pluginInstance.onload()
        }
        
        this.loadedPlugins.add(pluginId)
      }
      
      console.log(`[PluginLoader] Plugin "${pluginId}" enabled`)
    } catch (error) {
      console.error(`[PluginLoader] Failed to enable plugin ${pluginId}:`, error)
    }
  }

  async disablePlugin(pluginId: string): Promise<void> {
    try {
      const pluginInstance = app.getPlugin(pluginId)
      if (pluginInstance && typeof pluginInstance.onunload === 'function') {
        pluginInstance.onunload()
      }
      
      app.disablePlugin(pluginId)
      this.loadedPlugins.delete(pluginId)
      
      await invoke('disable_plugin', { pluginId })
      
      console.log(`[PluginLoader] Plugin "${pluginId}" disabled`)
    } catch (error) {
      console.error(`[PluginLoader] Failed to disable plugin ${pluginId}:`, error)
    }
  }

  async reloadPlugin(pluginId: string): Promise<Plugin | null> {
    try {
      await invoke('reload_plugin', { pluginId })
      const manifest = await invoke<PluginManifest>('get_plugin_manifest', { pluginId })
      return this.loadPluginFromBackend(manifest)
    } catch (error) {
      console.error(`[PluginLoader] Failed to reload plugin ${pluginId}:`, error)
      return null
    }
  }

  async reloadAllPlugins(): Promise<Plugin[]> {
    try {
      await invoke('reload_all_plugins')
      return this.loadPluginsFromBackend()
    } catch (error) {
      console.error('[PluginLoader] Failed to reload all plugins:', error)
      return []
    }
  }

  unloadPlugin(pluginId: string): void {
    app.disablePlugin(pluginId)
    this.loadedPlugins.delete(pluginId)
  }

  unloadAllPlugins(): void {
    for (const pluginId of this.loadedPlugins) {
      app.disablePlugin(pluginId)
    }
    this.loadedPlugins.clear()
    app.clearPluginResources()
    console.log('[PluginLoader] Unloaded all plugins and cleared resources')
  }

  async reloadPluginsFromDirectory(directory: string): Promise<Plugin[]> {
    this.unloadAllPlugins()
    return this.loadPluginsFromDirectory(directory)
  }

  async reloadPluginsFromBackend(): Promise<Plugin[]> {
    this.unloadAllPlugins()
    return this.loadPluginsFromBackend()
  }

  isLoaded(pluginId: string): boolean {
    return this.loadedPlugins.has(pluginId)
  }

  async getPluginData(pluginId: string): Promise<any> {
    try {
      const data = await invoke<string>('get_plugin_data', { pluginId })
      return JSON.parse(data)
    } catch (error) {
      console.error(`[PluginLoader] Failed to get data for plugin ${pluginId}:`, error)
      return null
    }
  }

  async setPluginData(pluginId: string, data: any): Promise<void> {
    try {
      await invoke('set_plugin_data', { pluginId, data: JSON.stringify(data) })
    } catch (error) {
      console.error(`[PluginLoader] Failed to set data for plugin ${pluginId}:`, error)
    }
  }
}

export const pluginLoader = new PluginLoader()
