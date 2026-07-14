import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import '@milkdown/theme-nord/style.css'
import './styles/main.css'

import App from './App.vue'
import router from './router'
import { useThemeStore } from './stores/theme'
import { rocApp, registerBuiltinPlugins, enableBuiltinPlugins } from './plugins'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(ElementPlus)

// 注册并启用内置插件
registerBuiltinPlugins()
enableBuiltinPlugins()

// 将插件系统的 app 对象注入到 Vue 应用中，方便组件访问
app.provide('rocApp', rocApp)

// 应用初始主题，并在系统主题变化时同步（仅当用户选择“跟随系统”时生效）
const themeStore = useThemeStore()
themeStore.applyTheme()
window
  .matchMedia('(prefers-color-scheme: dark)')
  .addEventListener('change', () => {
    if (themeStore.mode === 'system') {
      themeStore.updateActualTheme()
    }
  })

app.mount('#app')

// 应用挂载后，通知所有插件
rocApp.events.emit('app:mounted')
