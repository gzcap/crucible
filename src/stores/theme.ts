import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

function getSavedMode(): ThemeMode {
  const saved = localStorage.getItem('roc-theme-mode') as ThemeMode
  return saved || 'system'
}

function getSystemTheme(): ThemeMode {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>(getSavedMode())

  const actualTheme = computed<ThemeMode>(() => {
    if (mode.value === 'system') {
      return getSystemTheme()
    }
    return mode.value
  })

  function setMode(newMode: ThemeMode) {
    mode.value = newMode
    localStorage.setItem('roc-theme-mode', newMode)
    applyTheme()
  }

  function toggleMode() {
    const modes: ThemeMode[] = ['light', 'dark', 'system']
    const currentIndex = modes.indexOf(mode.value)
    setMode(modes[(currentIndex + 1) % modes.length])
  }

  function updateActualTheme() {
    applyTheme()
  }

  function applyTheme() {
    const root = document.documentElement
    root.classList.remove('light', 'dark')
    root.classList.add(actualTheme.value)
  }

  watch(actualTheme, () => {
    applyTheme()
  })

  return {
    mode,
    actualTheme,
    setMode,
    toggleMode,
    updateActualTheme,
    applyTheme,
  }
})