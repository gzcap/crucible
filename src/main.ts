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

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(ElementPlus)

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
