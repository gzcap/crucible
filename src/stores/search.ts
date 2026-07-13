import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SearchResult, SearchMode } from '../lib/types'
import { search, getAllTags, listAllNotes } from '../lib/tauri'

export const useSearchStore = defineStore('search', () => {
  const isOpen = ref(false)
  const query = ref('')
  const mode = ref<SearchMode>('fulltext')
  const results = ref<SearchResult[]>([])
  const tags = ref<string[]>([])
  const notes = ref<{ path: string; title: string }[]>([])

  async function loadTags() {
    tags.value = await getAllTags()
  }

  async function loadNotes() {
    const allNotes = await listAllNotes()
    notes.value = allNotes.map(n => ({ path: n.path, title: n.title }))
  }

  async function performSearch(queryStr: string, searchMode?: SearchMode) {
    if (!queryStr.trim()) {
      results.value = []
      return
    }

    mode.value = searchMode || mode.value
    results.value = await search(queryStr.trim(), mode.value, 20)
  }

  function open() {
    isOpen.value = true
    query.value = ''
    results.value = []
    loadTags()
    loadNotes()
  }

  function openWithQuery(queryStr: string) {
    isOpen.value = true
    query.value = queryStr
    loadTags()
    loadNotes()
    performSearch(queryStr)
  }

  function close() {
    isOpen.value = false
    query.value = ''
    results.value = []
  }

  function toggle() {
    if (isOpen.value) {
      close()
    } else {
      open()
    }
  }

  return {
    isOpen,
    query,
    mode,
    results,
    tags,
    notes,
    performSearch,
    open,
    openWithQuery,
    close,
    toggle
  }
})