import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Tauri 2 桌面应用的 Vite 配置
export default defineConfig({
  plugins: [vue()],
  // 不清屏，便于在 Tauri CLI 中查看日志
  clearScreen: false,
  server: {
    // Tauri 默认期望的开发服务器端口
    port: 1420,
    strictPort: true,
    host: '0.0.0.0',
  },
  // 允许在前端代码中通过 import.meta.env 访问的变量前缀
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2021',
    minify: 'esbuild',
    sourcemap: false,
  },
})
