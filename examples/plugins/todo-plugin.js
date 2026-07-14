/**
 * Todo Plugin - 第三方插件示例
 * 
 * 功能：
 * - 在命令面板中添加待办事项命令
 * - 监听文件变化事件
 * - 提供数据持久化配置
 * - 注册快捷键
 * 
 * 使用方法：
 * 1. 将此文件复制到你的仓库根目录的 .roc/plugins/ 文件夹中
 * 2. 打开 Roc 应用的设置面板
 * 3. 点击「从仓库加载插件」按钮
 * 4. 在插件列表中启用 Todo Plugin
 */

class TodoPlugin {
  /** 插件清单 - 必须定义 */
  static manifest = {
    id: 'todo-plugin',
    name: 'Todo Plugin',
    version: '1.0.0',
    description: '提供待办事项管理功能，支持添加、查看和完成待办任务'
  }

  /**
   * 插件构造函数
   * @param {App} app - 应用实例，提供核心 API 访问
   */
  constructor(app) {
    this.app = app
    this.todos = []
  }

  /**
   * 插件加载时调用
   * 在此方法中注册所有功能
   */
  onload() {
    console.log(`[${this.constructor.manifest.name}] 插件加载中...`)

    // 加载持久化配置
    this.loadData().then(config => {
      this.todos = config?.todos || []
      console.log(`[${this.constructor.manifest.name}] 已加载 ${this.todos.length} 个待办事项`)
    })

    // 注册命令 - 添加待办
    this.app.commands.addCommand({
      id: 'todo-plugin-add',
      name: '添加待办事项',
      desc: '添加一个新的待办任务',
      callback: () => this.addTodo()
    })

    // 注册命令 - 查看待办
    this.app.commands.addCommand({
      id: 'todo-plugin-list',
      name: '查看待办事项',
      desc: '显示所有待办任务列表',
      callback: () => this.listTodos()
    })

    // 注册命令 - 清除已完成
    this.app.commands.addCommand({
      id: 'todo-plugin-clear',
      name: '清除已完成待办',
      desc: '移除所有已完成的待办任务',
      callback: () => this.clearCompleted()
    })

    // 监听应用挂载事件
    this.registerEvent(this.app.events, 'app:mounted', () => {
      console.log(`[${this.constructor.manifest.name}] 应用已挂载`)
    })

    // 监听文件变化事件
    this.registerEvent(this.app.events, 'roc://file-changed', (event) => {
      console.log(`[${this.constructor.manifest.name}] 文件变化: ${event.kind} - ${event.path}`)
    })

    // 注册定时器 - 每分钟保存一次数据
    this.registerInterval(() => {
      this.saveData({ todos: this.todos })
    }, 60000)

    console.log(`[${this.constructor.manifest.name}] 插件加载完成`)
  }

  /**
   * 插件卸载时调用
   * 清理资源（自动注册的资源会自动清理）
   */
  onunload() {
    // 保存数据
    this.saveData({ todos: this.todos })
    console.log(`[${this.constructor.manifest.name}] 插件已卸载`)
  }

  /**
   * 添加待办事项
   */
  addTodo() {
    const text = prompt('请输入待办事项内容：')
    if (text && text.trim()) {
      this.todos.push({
        id: Date.now(),
        text: text.trim(),
        completed: false,
        createdAt: new Date().toISOString()
      })
      this.saveData({ todos: this.todos })
      alert(`已添加待办: ${text}`)
    }
  }

  /**
   * 显示待办事项列表
   */
  listTodos() {
    if (this.todos.length === 0) {
      alert('暂无待办事项')
      return
    }

    const completedCount = this.todos.filter(t => t.completed).length
    const pendingCount = this.todos.length - completedCount

    let message = `待办事项列表 (${pendingCount} 待完成 / ${completedCount} 已完成)\n\n`
    this.todos.forEach((todo, index) => {
      const status = todo.completed ? '[✓]' : '[ ]'
      message += `${index + 1}. ${status} ${todo.text}\n`
    })

    alert(message)
  }

  /**
   * 清除已完成的待办事项
   */
  clearCompleted() {
    const completed = this.todos.filter(t => t.completed)
    if (completed.length === 0) {
      alert('没有已完成的待办事项')
      return
    }

    if (confirm(`确定要清除 ${completed.length} 个已完成的待办事项吗？`)) {
      this.todos = this.todos.filter(t => !t.completed)
      this.saveData({ todos: this.todos })
      alert(`已清除 ${completed.length} 个待办事项`)
    }
  }

  /**
   * 加载插件数据
   * @returns {Promise<Object|null>} 插件配置数据
   */
  async loadData() {
    const key = `roc-plugin-data-${this.constructor.manifest.id}`
    const data = localStorage.getItem(key)
    return data ? JSON.parse(data) : null
  }

  /**
   * 保存插件数据
   * @param {Object} data - 要保存的数据
   */
  async saveData(data) {
    const key = `roc-plugin-data-${this.constructor.manifest.id}`
    localStorage.setItem(key, JSON.stringify(data))
  }

  /**
   * 注册事件监听（自动清理）
   * @param {Object} eventSource - 事件源
   * @param {string} eventName - 事件名称
   * @param {Function} callback - 回调函数
   */
  registerEvent(eventSource, eventName, callback) {
    eventSource.on(eventName, callback)
    this.register(() => {
      eventSource.off(eventName, callback)
    })
  }

  /**
   * 注册定时器（自动清理）
   * @param {Function} callback - 回调函数
   * @param {number} delay - 延迟时间（毫秒）
   * @returns {number} 定时器 ID
   */
  registerInterval(callback, delay) {
    const id = window.setInterval(callback, delay)
    this.register(() => {
      window.clearInterval(id)
    })
    return id
  }

  /**
   * 注册清理回调
   * @param {Function} cleanup - 清理函数
   */
  register(cleanup) {
    if (!this._cleanupFunctions) {
      this._cleanupFunctions = []
    }
    this._cleanupFunctions.push(cleanup)
  }
}

// 必须导出插件类作为 default
export default TodoPlugin