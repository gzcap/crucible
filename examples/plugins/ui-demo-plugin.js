/**
 * UI Demo Plugin - 自定义界面插件示例
 * 
 * 展示如何创建和使用各种自定义界面组件：
 * - 侧边栏面板
 * - 浮动面板 (Popover)
 * - 模态对话框 (Modal)
 * - 设置面板
 * 
 * 使用方法：
 * 1. 将此文件复制到你的仓库根目录的 .roc/plugins/ 文件夹中
 * 2. 打开 Roc 应用的设置面板
 * 3. 点击「从仓库加载插件」按钮
 * 4. 在插件列表中启用 UI Demo Plugin
 */

class UiDemoPlugin {
  /** 插件清单 */
  static manifest = {
    id: 'ui-demo-plugin',
    name: 'UI Demo',
    version: '1.0.0',
    description: '展示自定义界面功能，包括侧边栏面板、浮动面板和模态对话框'
  }

  /**
   * 构造函数
   * @param {App} app - 应用实例
   */
  constructor(app) {
    this.app = app
    this.panelContainer = null
    this.todos = []
  }

  /**
   * 插件加载
   */
  onload() {
    console.log(`[${this.constructor.manifest.name}] 加载中...`)

    this.loadData().then(config => {
      this.todos = config?.todos || []
    })

    // 注册命令 - 打开浮动面板
    this.app.commands.addCommand({
      id: 'ui-demo-open-popover',
      name: '打开浮动面板',
      desc: '展示浮动面板示例',
      callback: () => this.openPopoverDemo()
    })

    // 注册命令 - 打开模态对话框
    this.app.commands.addCommand({
      id: 'ui-demo-open-modal',
      name: '打开模态对话框',
      desc: '展示模态对话框示例',
      callback: () => this.openModalDemo()
    })

    // 注册命令 - 打开侧边栏面板
    this.app.commands.addCommand({
      id: 'ui-demo-open-sidebar',
      name: '打开 Demo 侧边栏',
      desc: '打开插件注册的侧边栏面板',
      callback: () => this.app.workspace.openSidebarPanel('ui-demo-panel')
    })

    // 注册侧边栏面板
    this.registerSidebarPanel()

    // 注册设置面板
    this.registerSettingTab()

    console.log(`[${this.constructor.manifest.name}] 加载完成`)
  }

  /**
   * 插件卸载
   */
  onunload() {
    this.saveData({ todos: this.todos })
    console.log(`[${this.constructor.manifest.name}] 已卸载`)
  }

  /**
   * 注册侧边栏面板
   */
  registerSidebarPanel() {
    this.app.workspace.registerSidebarPanel({
      id: 'ui-demo-panel',
      name: 'Demo Panel',
      icon: '🎨',
      render: (container) => {
        this.renderSidebarPanel(container)
      }
    })
  }

  /**
   * 渲染侧边栏面板内容
   */
  renderSidebarPanel(container) {
    this.panelContainer = container
    
    container.innerHTML = `
      <div style="padding: 12px;">
        <h4 style="margin: 0 0 12px 0; font-size: 14px; font-weight: 600;">🎯 Todo List</h4>
        <div id="todo-list" style="margin-bottom: 12px; min-height: 40px;"></div>
        <input 
          type="text" 
          id="todo-input"
          placeholder="添加新任务..."
          style="
            width: 100%;
            padding: 6px 8px;
            border: 1px solid var(--roc-border);
            border-radius: 4px;
            background: var(--roc-bg-secondary);
            color: var(--roc-text-primary);
            font-size: 13px;
            box-sizing: border-box;
          "
        />
        <button 
          id="add-todo-btn"
          style="
            width: 100%;
            margin-top: 8px;
            padding: 6px;
            background: var(--roc-accent);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 13px;
          "
        >
          添加
        </button>
      </div>
    `

    this.updateTodoList()

    const input = container.querySelector('#todo-input')
    const btn = container.querySelector('#add-todo-btn')

    const handleAdd = () => {
      const text = input.value.trim()
      if (text) {
        this.todos.push({
          id: Date.now(),
          text,
          completed: false
        })
        input.value = ''
        this.updateTodoList()
        this.saveData({ todos: this.todos })
      }
    }

    btn.addEventListener('click', handleAdd)
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') handleAdd()
    })

    this.register(() => {
      btn.removeEventListener('click', handleAdd)
      input.removeEventListener('keydown', handleAdd)
    })
  }

  /**
   * 更新 Todo 列表
   */
  updateTodoList() {
    if (!this.panelContainer) return
    
    const list = this.panelContainer.querySelector('#todo-list')
    if (!list) return

    if (this.todos.length === 0) {
      list.innerHTML = `<p style="color: var(--roc-text-muted); font-size: 12px; text-align: center; margin: 0;">暂无任务</p>`
      return
    }

    list.innerHTML = this.todos.map((todo, index) => `
      <div 
        style="
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px;
          margin-bottom: 4px;
          border-radius: 4px;
          background: var(--roc-bg-secondary);
        "
      >
        <input 
          type="checkbox" 
          ${todo.completed ? 'checked' : ''}
          onchange="window.__ROC_PLUGINS__.uiDemo.toggleTodo(${todo.id})"
          style="cursor: pointer;"
        />
        <span style="flex: 1; font-size: 13px; ${todo.completed ? 'text-decoration: line-through; color: var(--roc-text-muted);' : ''}">
          ${index + 1}. ${todo.text}
        </span>
        <button 
          onclick="window.__ROC_PLUGINS__.uiDemo.deleteTodo(${todo.id})"
          style="
            border: none;
            background: transparent;
            color: var(--roc-text-muted);
            cursor: pointer;
            font-size: 14px;
          "
        >
          ×
        </button>
      </div>
    `).join('')
  }

  /**
   * 切换 Todo 状态
   */
  toggleTodo(id) {
    const todo = this.todos.find(t => t.id === id)
    if (todo) {
      todo.completed = !todo.completed
      this.updateTodoList()
      this.saveData({ todos: this.todos })
    }
  }

  /**
   * 删除 Todo
   */
  deleteTodo(id) {
    this.todos = this.todos.filter(t => t.id !== id)
    this.updateTodoList()
    this.saveData({ todos: this.todos })
  }

  /**
   * 打开浮动面板演示
   */
  openPopoverDemo() {
    const popover = this.app.workspace.createPopover({
      title: '浮动面板示例',
      content: `
        <div style="text-align: center;">
          <p style="margin: 8px 0;">这是一个浮动面板示例</p>
          <p style="font-size: 12px; color: var(--roc-text-muted);">可以显示提示信息或快捷操作</p>
        </div>
      `,
      closable: true,
      onClose: () => {
        console.log('[UI Demo] Popover closed')
      }
    })
  }

  /**
   * 打开模态对话框演示
   */
  openModalDemo() {
    this.app.workspace.createModal({
      title: '模态对话框示例',
      content: (container) => {
        container.innerHTML = `
          <div>
            <p style="margin: 0 0 12px 0;">这是一个自定义模态对话框</p>
            <p style="font-size: 12px; color: var(--roc-text-muted);">可以包含表单、列表或其他复杂内容</p>
            <input 
              type="text" 
              placeholder="输入一些内容..."
              style="
                width: 100%;
                margin-top: 12px;
                padding: 8px;
                border: 1px solid var(--roc-border);
                border-radius: 4px;
                background: var(--roc-bg-secondary);
                color: var(--roc-text-primary);
                box-sizing: border-box;
              "
            />
          </div>
        `
      },
      width: '450px',
      confirmText: '确认',
      cancelText: '取消',
      onConfirm: () => {
        alert('确认按钮被点击！')
      },
      onCancel: () => {
        console.log('[UI Demo] Modal cancelled')
      }
    })
  }

  /**
   * 注册设置面板
   */
  registerSettingTab() {
    this.app.workspace.registerSettingTab({
      id: 'ui-demo-settings',
      name: 'UI Demo 设置',
      icon: '⚙️',
      render: (container) => {
        container.innerHTML = `
          <div style="padding: 16px;">
            <h3 style="margin: 0 0 16px 0;">UI Demo 插件设置</h3>
            
            <div style="margin-bottom: 16px;">
              <label style="display: block; margin-bottom: 4px; font-size: 13px;">
                <input type="checkbox" checked /> 启用浮动面板
              </label>
            </div>
            
            <div style="margin-bottom: 16px;">
              <label style="display: block; margin-bottom: 4px; font-size: 13px;">
                <input type="checkbox" checked /> 启用模态对话框
              </label>
            </div>
            
            <div>
              <label style="display: block; margin-bottom: 4px; font-size: 13px;">主题颜色</label>
              <select style="
                padding: 6px;
                border: 1px solid var(--roc-border);
                border-radius: 4px;
                background: var(--roc-bg-secondary);
                color: var(--roc-text-primary);
              ">
                <option>蓝色</option>
                <option>绿色</option>
                <option>紫色</option>
              </select>
            </div>
          </div>
        `
      }
    })
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
   * 注册清理回调
   */
  register(cleanup) {
    if (!this._cleanupFunctions) {
      this._cleanupFunctions = []
    }
    this._cleanupFunctions.push(cleanup)
  }
}

// 暴露插件实例到全局，以便在 HTML 中调用
window.__ROC_PLUGINS__ = window.__ROC_PLUGINS__ || {}
window.__ROC_PLUGINS__.uiDemo = null

// 插件加载时保存实例引用
const originalOnload = UiDemoPlugin.prototype.onload
UiDemoPlugin.prototype.onload = function() {
  window.__ROC_PLUGINS__.uiDemo = this
  originalOnload.call(this)
}

export default UiDemoPlugin