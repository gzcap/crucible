import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { NoteMeta, Backlink, EditorTab, FileChangeEvent } from '../lib/types'
import {
  listAllNotes,
  getBacklinks,
  onFileChanged,
  onVaultOpened,
  onVaultClosed,
  createNote
} from '../lib/tauri'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { useVaultStore } from './vault'

export const useNotesStore = defineStore('notes', () => {
  const vaultStore = useVaultStore()
  const notes = ref<NoteMeta[]>([])
  const currentPath = ref<string | null>(null)
  const currentContent = ref('')
  const backlinks = ref<Backlink[]>([])
  const tabs = ref<EditorTab[]>([])
  const dirtyPaths = ref<Set<string>>(new Set())
  const fileTree = ref<FileTreeNode | null>(null)
  const savingPaths = new Set<string>()
  let openNoteRequestId = 0

  interface FileTreeNode {
    name: string
    path: string
    isDir: boolean
    children?: FileTreeNode[]
  }

  const currentTab = computed(() => {
    return tabs.value.find(t => t.path === currentPath.value)
  })

  /** 将笔记相对路径解析为绝对路径（基于当前仓库路径） */
  function resolveNotePath(relPath: string): string {
    const base = vaultStore.currentVault?.path
    if (!base) throw new Error('No vault open')
    return `${base.replace(/\/$/, '')}/${relPath}`
  }

  const allTags = computed(() => {
    const tagSet = new Set<string>()
    for (const note of notes.value) {
      for (const tag of note.tags) {
        tagSet.add(tag)
      }
    }
    return Array.from(tagSet).sort()
  })

  async function createNewNote() {
    const result = await createNote()
    if (result) {
      await loadNotes()
      await openNote(result.path)
    }
  }

  async function loadNotes() {
    try {
      const result = await listAllNotes()
      notes.value = result
      buildFileTree()
    } catch (e) {
      console.error('[notes] loadNotes error:', e)
    }
  }

  function buildFileTree() {
    const root: FileTreeNode = { name: '', path: '', isDir: true, children: [] }

    for (const note of notes.value) {
      const parts = note.path.split('/')
      let current = root

      for (let i = 0; i < parts.length; i++) {
        const part = parts[i]
        const fullPath = parts.slice(0, i + 1).join('/')
        const isLast = i === parts.length - 1

        let child = current.children?.find(c => c.name === part)
        if (!child) {
          child = {
            name: part,
            path: fullPath,
            isDir: !isLast,
            children: !isLast ? [] : undefined
          }
          current.children?.push(child)
        }
        current = child
      }
    }

    fileTree.value = root
  }

  async function openNote(path: string) {
    const requestId = ++openNoteRequestId

    currentPath.value = path

    if (!tabs.value.find(t => t.path === path)) {
      const note = notes.value.find(n => n.path === path)
      tabs.value.push({
        path,
        title: note?.title || path,
        dirty: false,
        type: 'note'
      })
    }

    try {
      const content = await readTextFile(resolveNotePath(path))
      // 仅当没有更新的请求时才应用结果
      if (requestId === openNoteRequestId) {
        currentContent.value = content
      }
    } catch {
      if (requestId === openNoteRequestId) {
        currentContent.value = ''
      }
    }

    // 加载反向链接（同样检查请求 ID，防止竞态覆盖）
    getBacklinks(path).then((result) => {
      if (requestId === openNoteRequestId) {
        backlinks.value = result
      }
    }).catch(() => {
      if (requestId === openNoteRequestId) {
        backlinks.value = []
      }
    })
  }

  function openGraph() {
    const graphPath = '__graph__'
    currentPath.value = graphPath

    if (!tabs.value.find(t => t.path === graphPath)) {
      tabs.value.push({
        path: graphPath,
        title: '关系图谱',
        dirty: false,
        type: 'graph'
      })
    }
  }

  async function saveNote(path: string, content: string) {
    savingPaths.add(path)
    // Milkdown commonmark 序列化器会将 [[ 转义为 \[\[，这里还原以确保 wikilink 格式正确
    const cleaned = content.replace(/\\?\[\\?\[/g, '[[').replace(/\\?\]\\?\]/g, ']]')
    await writeTextFile(resolveNotePath(path), cleaned)
    dirtyPaths.value.delete(path)

    const tab = tabs.value.find(t => t.path === path)
    if (tab) {
      tab.dirty = false
    }

    // 等待 watcher 事件窗口过后再清除标记（watcher 有 300ms 防抖，留余量）
    setTimeout(() => {
      savingPaths.delete(path)
    }, 1000)

    // 保存后刷新反向链接
    setTimeout(() => {
      if (currentPath.value) {
        loadBacklinks(currentPath.value)
      }
    }, 500)
  }

  async function loadBacklinks(path: string) {
    backlinks.value = await getBacklinks(path)
  }

  function closeTab(path: string) {
    const index = tabs.value.findIndex(t => t.path === path)
    if (index > -1) {
      tabs.value.splice(index, 1)
    }

    if (currentPath.value === path) {
      currentPath.value = tabs.value[0]?.path || null
      if (currentPath.value) {
        openNote(currentPath.value)
      }
    }
  }

  function setDirty(path: string) {
    dirtyPaths.value.add(path)
    const tab = tabs.value.find(t => t.path === path)
    if (tab) {
      tab.dirty = true
    }
  }

  function handleFileChange(event: FileChangeEvent) {
    if (event.kind === 'delete') {
      notes.value = notes.value.filter(n => n.path !== event.path)
      tabs.value = tabs.value.filter(t => t.path !== event.path)
      if (currentPath.value === event.path) {
        currentPath.value = tabs.value[0]?.path || null
      }
    } else if (event.kind === 'create' || event.kind === 'modify') {
      loadNotes()
      // 如果是当前正在保存的文件触发的 modify，不重新加载内容（避免光标重置）
      if (currentPath.value === event.path && !savingPaths.has(event.path)) {
        openNote(event.path)
      }
    } else if (event.kind === 'rename') {
      notes.value = notes.value.map(n =>
        n.path === event.path ? { ...n, path: event.new_path! } : n
      )
      tabs.value = tabs.value.map(t =>
        t.path === event.path ? { ...t, path: event.new_path! } : t
      )
      if (currentPath.value === event.path) {
        currentPath.value = event.new_path ?? null
      }
    }
    buildFileTree()
  }

  async function setupEventListeners() {
    await onFileChanged(handleFileChange)
    await onVaultOpened(() => loadNotes())
    await onVaultClosed(() => {
      notes.value = []
      currentPath.value = null
      currentContent.value = ''
      backlinks.value = []
      tabs.value = []
      fileTree.value = null
    })
  }

  return {
    notes,
    currentPath,
    currentContent,
    backlinks,
    tabs,
    currentTab,
    fileTree,
    allTags,
    loadNotes,
    openNote,
    openGraph,
    saveNote,
    loadBacklinks,
    closeTab,
    createNewNote,
    setDirty,
    setupEventListeners
  }
})