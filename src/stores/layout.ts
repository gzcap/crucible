import { defineStore } from 'pinia'
import { ref } from 'vue'

const STORAGE_KEY = 'roc-layout'

interface LayoutState {
  leftSidebarWidth: number
  rightSidebarWidth: number
}

/** 面板宽度边界约束 */
const MIN_WIDTH = 160
const MAX_WIDTH = 520
const DEFAULT_LEFT = 240
const DEFAULT_RIGHT = 300

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max)
}

function getSavedState(): LayoutState {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) {
      const parsed = JSON.parse(saved)
      return {
        leftSidebarWidth: clamp(parsed.leftSidebarWidth ?? DEFAULT_LEFT, MIN_WIDTH, MAX_WIDTH),
        rightSidebarWidth: clamp(parsed.rightSidebarWidth ?? DEFAULT_RIGHT, MIN_WIDTH, MAX_WIDTH),
      }
    }
  } catch {
    /* 忽略解析错误，回退默认值 */
  }
  return { leftSidebarWidth: DEFAULT_LEFT, rightSidebarWidth: DEFAULT_RIGHT }
}

export const useLayoutStore = defineStore('layout', () => {
  const saved = getSavedState()
  const leftSidebarWidth = ref(saved.leftSidebarWidth)
  const rightSidebarWidth = ref(saved.rightSidebarWidth)

  /** 侧边栏内容面板可见性（点击活动栏当前项切换） */
  const leftSidebarVisible = ref(true)
  const rightSidebarVisible = ref(false)

  function persist() {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        leftSidebarWidth: leftSidebarWidth.value,
        rightSidebarWidth: rightSidebarWidth.value,
      }),
    )
  }

  /** 设置左侧边栏内容区宽度（含边界约束） */
  function setLeftSidebarWidth(width: number) {
    leftSidebarWidth.value = clamp(width, MIN_WIDTH, MAX_WIDTH)
  }

  /** 设置右侧边栏内容区宽度（含边界约束） */
  function setRightSidebarWidth(width: number) {
    rightSidebarWidth.value = clamp(width, MIN_WIDTH, MAX_WIDTH)
  }

  /** 拖动结束后持久化 */
  function commit() {
    persist()
  }

  return {
    leftSidebarWidth,
    rightSidebarWidth,
    leftSidebarVisible,
    rightSidebarVisible,
    setLeftSidebarWidth,
    setRightSidebarWidth,
    commit,
  }
})
