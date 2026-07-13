<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, onErrorCaptured } from 'vue'
import { Milkdown, useEditor } from '@milkdown/vue'
import { Editor, rootCtx, defaultValueCtx, editorViewOptionsCtx } from '@milkdown/kit/core'
import { commonmark } from '@milkdown/kit/preset/commonmark'
import { gfm } from '@milkdown/kit/preset/gfm'
import { nord } from '@milkdown/theme-nord'
import { listener, listenerCtx } from '@milkdown/kit/plugin/listener'
import { history } from '@milkdown/kit/plugin/history'
import { clipboard } from '@milkdown/kit/plugin/clipboard'
import { cursor } from '@milkdown/kit/plugin/cursor'
import { prism } from '@milkdown/plugin-prism'
import { automd } from '@milkdown/plugin-automd'
import { replaceAll } from '@milkdown/kit/utils'
import type { Ctx } from '@milkdown/kit/ctx'
import { editorViewCtx } from '@milkdown/kit/core'
import { wikilink } from './wikilink'
import { resolveWikilink, listAllNotes } from '../../lib/tauri'
import { useNotesStore } from '../../stores/notes'
import { useEditorStore } from '../../stores/editor'
import type { NoteMeta } from '../../lib/types'

const notesStore = useNotesStore()
const editorStore = useEditorStore()

const containerRef = ref<HTMLElement | null>(null)

/** 自动补全状态 */
const showCompletion = ref(false)
const completionItems = ref<NoteMeta[]>([])
const completionIndex = ref(0)
const completionQuery = ref('')
let completionAnchor: { from: number; to: number } | null = null
let allNotesCache: NoteMeta[] = []

/** 防止编辑器内部更新与外部 watch 循环触发 */
let isInternalChange = false
/** 文件切换标记：确保切换文件时内容一定能同步到编辑器 */
let pendingSwitch = false

/** 捕获子组件（Milkdown/ProseMirror）渲染错误，阻止错误冒泡导致整个布局崩溃 */
onErrorCaptured((err, instance, info) => {
  console.error('[MilkdownEditor] errorCaptured:', err, info)
  return false
})

const { get: getEditor, loading } = useEditor((root: HTMLDivElement) =>
  Editor.make()
    .config((ctx: Ctx) => {
      ctx.set(rootCtx, root)
      ctx.set(defaultValueCtx, notesStore.currentContent)
      ctx.update(editorViewOptionsCtx, (prev) => ({
        ...prev,
        attributes: { class: 'milkdown-body', spellcheck: 'false' },
      }))

      const listenerAPI = ctx.get(listenerCtx)
      listenerAPI.markdownUpdated((_: Ctx, md: string, prevMd: string) => {
        if (md !== prevMd) {
          try {
            isInternalChange = true
            notesStore.currentContent = md
            if (notesStore.currentPath) {
              notesStore.setDirty(notesStore.currentPath)
            }
          } catch (e) {
            console.error('[Milkdown] markdownUpdated error:', e)
          }
        }
      })
    })
    .config(nord)
    .use(commonmark)
    .use(gfm)
    .use(listener)
    .use(history)
    .use(clipboard)
    .use(cursor)
    .use(prism)
    .use(automd)
    .use(wikilink),
)

function syncToEditor(content: string) {
  const editor = getEditor()
  if (!editor) return
  try {
    editor.action(replaceAll(content))
  } catch (e) {
    console.error('[Milkdown] syncToEditor error:', e)
  }
}

/** 获取 ProseMirror EditorView */
function getEditorView(): any | null {
  const editor = getEditor()
  if (!editor) return null
  let view: any = null
  try {
    editor.action((ctx: Ctx) => {
      view = ctx.get(editorViewCtx)
    })
  } catch {
    return null
  }
  return view
}

/** 检查光标前是否有 [[ 触发补全 */
function checkCompletion() {
  const editorView = getEditorView()
  if (!editorView) return

  const state = editorView.state
  const pos = state.selection.head
  const textBefore = state.doc.textBetween(Math.max(0, pos - 100), pos, '\n', '\0')

  // 匹配 [[ 后面的内容（不含 ]]
  const match = textBefore.match(/\[\[([^\]\n|]*?)$/)
  if (match) {
    completionQuery.value = match[1]
    const from = pos - match[0].length
    completionAnchor = { from, to: pos }

    // 过滤笔记列表
    const q = completionQuery.value.toLowerCase()
    completionItems.value = allNotesCache
      .filter(n => !q || n.title.toLowerCase().includes(q) || n.path.toLowerCase().includes(q))
      .slice(0, 10)

    if (completionItems.value.length > 0) {
      showCompletion.value = true
      completionIndex.value = 0
      return
    }
  }

  showCompletion.value = false
}

/** 插入选中的 wikilink */
function insertCompletion(note: NoteMeta) {
  const editorView = getEditorView()
  if (!editorView || !completionAnchor) return

  const stem = note.path.replace(/\.md$/, '').split('/').pop() || note.title
  const insertText = `[[${stem}]]`

  editorView.dispatch({
    changes: {
      from: completionAnchor.from,
      to: completionAnchor.to,
      insert: insertText,
    },
  })

  showCompletion.value = false
  completionAnchor = null
  editorView.focus()
}

/** 加载笔记列表缓存 */
async function refreshNoteCache() {
  try {
    allNotesCache = await listAllNotes()
  } catch {
    allNotesCache = []
  }
}

/** 键盘导航补全列表 */
function handleKeydown(e: KeyboardEvent) {
  if (!showCompletion.value) {
    // 检查是否输入了 [[ 后需要触发
    if (e.key !== 'Escape' && e.key !== 'ArrowUp' && e.key !== 'ArrowDown' && e.key !== 'Enter') {
      setTimeout(checkCompletion, 0)
    }
    return
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    completionIndex.value = (completionIndex.value + 1) % completionItems.value.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    completionIndex.value = (completionIndex.value - 1 + completionItems.value.length) % completionItems.value.length
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (completionItems.value[completionIndex.value]) {
      insertCompletion(completionItems.value[completionIndex.value])
    }
  } else if (e.key === 'Escape') {
    e.preventDefault()
    showCompletion.value = false
  } else {
    setTimeout(checkCompletion, 0)
  }
}

/** 点击 wikilink 时打开对应笔记 */
async function handleWikilinkClick(e: MouseEvent) {
  const target = (e.target as HTMLElement).closest('.roc-wikilink')
  if (!target) return
  e.preventDefault()
  e.stopPropagation()
  const noteName = (target as HTMLElement).getAttribute('data-target')
  if (!noteName) return
  try {
    const path = await resolveWikilink(noteName)
    if (path) {
      notesStore.openNote(path)
    }
  } catch {
    // 笔记不存在则忽略
  }
}

/** 外部内容变化（切换文件 / 文件被外部修改）时同步到编辑器 */
watch(
  () => notesStore.currentContent,
  (newContent) => {
    // 文件切换时强制同步，不受 isInternalChange 影响
    if (isInternalChange && !pendingSwitch) {
      isInternalChange = false
      return
    }
    isInternalChange = false
    pendingSwitch = false
    syncToEditor(newContent)
  },
)

/** 编辑器创建完成后同步内容（解决异步创建期间内容丢失问题） */
watch(
  loading,
  (isLoading) => {
    if (!isLoading) {
      syncToEditor(notesStore.currentContent)
    }
  },
  { immediate: true },
)

/** 全局保存快捷键（capture 阶段捕获，确保在编辑器处理前触发） */
function onGlobalSave(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    if (notesStore.currentPath) {
      e.preventDefault()
      notesStore.saveNote(notesStore.currentPath, notesStore.currentContent)
    }
  }
}

onMounted(async () => {
  document.addEventListener('keydown', onGlobalSave, true)
  await refreshNoteCache()
  if (containerRef.value) {
    containerRef.value.addEventListener('click', handleWikilinkClick, true)
    containerRef.value.addEventListener('keyup', handleKeydown)
  }
  editorStore.setEditorView(getEditor())
})

onUnmounted(() => {
  document.removeEventListener('keydown', onGlobalSave, true)
  if (containerRef.value) {
    containerRef.value.removeEventListener('click', handleWikilinkClick, true)
    containerRef.value.removeEventListener('keyup', handleKeydown)
  }
})

// 切换笔记时刷新缓存，并标记强制同步
watch(() => notesStore.currentPath, () => {
  pendingSwitch = true
  isInternalChange = false
  refreshNoteCache()
  // 兜底：若新文件内容与旧文件相同（watch 不触发），延迟强制同步
  setTimeout(() => {
    if (pendingSwitch) {
      pendingSwitch = false
      syncToEditor(notesStore.currentContent)
    }
  }, 150)
})
</script>

<template>
  <div ref="containerRef" class="milkdown-wrapper">
    <Milkdown class="milkdown-pane" />

    <!-- wikilink 自动补全下拉框 -->
    <div v-if="showCompletion" class="wikilink-completion">
      <div
        v-for="(note, idx) in completionItems"
        :key="note.path"
        class="completion-item"
        :class="{ active: idx === completionIndex }"
        @mousedown.prevent="insertCompletion(note)"
        @mouseenter="completionIndex = idx"
      >
        <span class="completion-title">{{ note.title }}</span>
        <span class="completion-path">{{ note.path }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.milkdown-wrapper {
  flex: 1;
  height: 100%;
  overflow: hidden;
  position: relative;
}

.milkdown-wrapper :deep(.roc-wikilink) {
  color: var(--roc-accent);
  cursor: pointer;
  text-decoration: none;
  border-bottom: 1px dashed var(--roc-accent);
  padding: 0 2px;
  border-radius: 3px;
  transition: background 0.15s, opacity 0.15s;
}

.milkdown-wrapper :deep(.roc-wikilink:hover) {
  background: rgba(0, 122, 204, 0.08);
  opacity: 0.9;
}

/* 自动补全下拉框 */
.wikilink-completion {
  position: absolute;
  z-index: 1000;
  background: var(--roc-bg-secondary);
  border: 1px solid var(--roc-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  max-height: 280px;
  overflow-y: auto;
  min-width: 240px;
  bottom: 20px;
  left: 40px;
}

.completion-item {
  display: flex;
  flex-direction: column;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.1s;
  gap: 2px;
}

.completion-item.active {
  background: var(--roc-bg-tertiary);
}

.completion-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--roc-text-primary);
}

.completion-path {
  font-size: 11px;
  color: var(--roc-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
