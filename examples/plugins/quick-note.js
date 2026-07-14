/**
 * Quick Note Plugin - 第三方插件示例（进阶版）
 * 
 * 功能：
 * - 快速创建笔记
 * - 在当前笔记中插入时间戳
 * - 访问 vault 文件系统
 * - 使用 workspace 打开文件
 * - 监听编辑器事件
 * 
 * 使用方法：
 * 1. 将此文件复制到你的仓库根目录的 .roc/plugins/ 文件夹中
 * 2. 打开 Roc 应用的设置面板
 * 3. 点击「从仓库加载插件」按钮
 * 4. 在插件列表中启用 Quick Note Plugin
 */

class QuickNotePlugin {
  /** 插件清单 */
  static manifest = {
    id: 'quick-note-plugin',
    name: 'Quick Note',
    version: '1.0.0',
    description: '提供快速笔记功能，支持一键创建笔记和插入时间戳'
  }

  /**
   * 构造函数
   * @param {App} app - 应用实例
   */
  constructor(app) {
    this.app = app
    this.config = {
      defaultTitle: 'Quick Note',
      autoOpen: true
    }
  }

  /**
   * 插件加载
   */
  onload() {
    console.log(`[${this.constructor.manifest.name}] 加载中...`)

    // 加载配置
    this.loadData().then(config => {
      if (config) {
        this.config = { ...this.config, ...config }
      }
    })

    // 注册命令 - 创建快速笔记
    this.app.commands.addCommand({
      id: 'quick-note-create',
      name: '创建快速笔记',
      desc: '快速创建一个新笔记',
      callback: () => this.createQuickNote()
    })

    // 注册命令 - 插入时间戳
    this.app.commands.addCommand({
      id: 'quick-note-insert-timestamp',
      name: '插入时间戳',
      desc: '在当前笔记中插入格式化的时间戳',
      callback: () => this.insertTimestamp()
    })

    // 注册命令 - 打开今日笔记
    this.app.commands.addCommand({
      id: 'quick-note-open-today',
      name: '打开今日笔记',
      desc: '打开或创建今天日期命名的笔记',
      callback: () => this.openTodayNote()
    })

    // 注册命令 - 打开设置
    this.app.commands.addCommand({
      id: 'quick-note-settings',
      name: 'Quick Note 设置',
      desc: '配置快速笔记插件',
      callback: () => this.openSettings()
    })

    // 监听应用挂载
    this.registerEvent(this.app.events, 'app:mounted', () => {
      console.log(`[${this.constructor.manifest.name}] 应用已挂载`)
    })

    console.log(`[${this.constructor.manifest.name}] 加载完成`)
  }

  /**
   * 插件卸载
   */
  onunload() {
    this.saveData(this.config)
    console.log(`[${this.constructor.manifest.name}] 已卸载`)
  }

  /**
   * 创建快速笔记
   */
  async createQuickNote() {
    const title = prompt('请输入笔记标题（默认：Quick Note）：', this.config.defaultTitle)
    const noteTitle = (title && title.trim()) || this.config.defaultTitle
    
    try {
      // 使用 vault 创建文件
      const filePath = `${noteTitle}.md`
      const content = `# ${noteTitle}\n\n> 创建于 ${new Date().toLocaleString()}\n\n`
      
      await this.app.vault.create(filePath, content)
      console.log(`[${this.constructor.manifest.name}] 创建笔记: ${filePath}`)

      if (this.config.autoOpen) {
        // 使用 workspace 打开文件
        this.app.workspace.openFile(filePath)
      }

      alert(`已创建笔记: ${filePath}`)
    } catch (error) {
      console.error(`[${this.constructor.manifest.name}] 创建笔记失败:`, error)
      alert('创建笔记失败，请重试')
    }
  }

  /**
   * 在当前笔记插入时间戳
   */
  insertTimestamp() {
    const activeFile = this.app.workspace.activeFilePath
    if (!activeFile) {
      alert('请先打开一个笔记')
      return
    }

    const timestamp = new Date().toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })

    alert(`已在 ${activeFile} 中插入时间戳: ${timestamp}`)
    console.log(`[${this.constructor.manifest.name}] 插入时间戳到: ${activeFile}`)
  }

  /**
   * 打开今日笔记
   */
  async openTodayNote() {
    const today = new Date()
    const dateStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`
    const filePath = `${dateStr}.md`

    try {
      // 检查文件是否存在
      const files = await this.app.vault.getMarkdownFiles()
      
      if (!files.includes(filePath)) {
        // 创建新文件
        const content = `# ${dateStr}\n\n> ${today.toLocaleDateString('zh-CN', { weekday: 'long' })}\n\n## 今日事项\n\n## 笔记\n\n`
        await this.app.vault.create(filePath, content)
        console.log(`[${this.constructor.manifest.name}] 创建今日笔记: ${filePath}`)
      }

      // 打开文件
      this.app.workspace.openFile(filePath)
    } catch (error) {
      console.error(`[${this.constructor.manifest.name}] 打开今日笔记失败:`, error)
      alert('打开今日笔记失败')
    }
  }

  /**
   * 打开设置对话框
   */
  openSettings() {
    const newTitle = prompt('默认笔记标题：', this.config.defaultTitle)
    if (newTitle !== null) {
      this.config.defaultTitle = newTitle.trim() || 'Quick Note'
    }

    const autoOpen = confirm('创建笔记后自动打开？')
    this.config.autoOpen = autoOpen

    this.saveData(this.config)
    alert('设置已保存')
  }

  /**
   * 加载配置
   */
  async loadData() {
    const key = `roc-plugin-data-${this.constructor.manifest.id}`
    const data = localStorage.getItem(key)
    return data ? JSON.parse(data) : null
  }

  /**
   * 保存配置
   */
  async saveData(data) {
    const key = `roc-plugin-data-${this.constructor.manifest.id}`
    localStorage.setItem(key, JSON.stringify(data))
  }

  /**
   * 注册事件监听
   */
  registerEvent(eventSource, eventName, callback) {
    eventSource.on(eventName, callback)
    this.register(() => {
      eventSource.off(eventName, callback)
    })
  }

  /**
   * 注册清理回调
   */
  register(cleanup) {
    if (!this._cleanupFunctions) {
      this._cleanupFunctions = []
    }
    this._cleanupFunctions.push(cleanup)
  }
}

export default QuickNotePlugin