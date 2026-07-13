<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useSearchStore } from '../../stores/search'
import { useNotesStore } from '../../stores/notes'

const searchStore = useSearchStore()
const notesStore = useNotesStore()

const searchInput = ref<HTMLInputElement | null>(null)
const selectedIndex = ref(0)

const modeLabels = {
  fulltext: 'Text',
  tag: 'Tag',
  link: 'Link'
}

const displayResults = computed(() => {
  if (!searchStore.query.trim()) {
    return searchStore.notes.map(n => ({
      type: 'note' as const,
      path: n.path,
      title: n.title,
      snippet: ''
    }))
  }
  return searchStore.results.map(r => ({
    type: 'result' as const,
    path: r.path,
    title: r.title,
    snippet: r.snippet
  }))
})

watch(() => searchStore.isOpen, async (isOpen) => {
  if (isOpen) {
    await nextTick()
    searchInput.value?.focus()
    selectedIndex.value = 0
  }
})

watch(() => searchStore.query, () => {
  selectedIndex.value = 0
})

watch(() => searchStore.results, () => {
  selectedIndex.value = 0
})

async function handleInput() {
  await searchStore.performSearch(searchStore.query)
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, displayResults.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    if (displayResults.value[selectedIndex.value]) {
      selectResult(displayResults.value[selectedIndex.value])
    }
  } else if (e.key === 'Escape') {
    searchStore.close()
  }
}

function selectResult(result: { path: string; title: string }) {
  notesStore.openNote(result.path)
  searchStore.close()
}

function selectMode(mode: string) {
  searchStore.mode = mode as any
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.command-palette')) {
    searchStore.close()
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>

<template>
  <div v-if="searchStore.isOpen" class="command-palette-overlay">
    <div class="command-palette">
      <div class="palette-header">
        <input
          ref="searchInput"
          type="text"
          class="search-input"
          v-model="searchStore.query"
          @input="handleInput"
          @keydown="handleKeydown"
          placeholder="Search notes..."
        />
        <div class="mode-selector">
          <button
            v-for="(label, mode) in modeLabels"
            :key="mode"
            class="mode-btn"
            :class="{ active: searchStore.mode === mode }"
            @click="selectMode(mode)"
          >
            {{ label }}
          </button>
        </div>
      </div>
      
      <div class="results-list">
        <div
          v-for="(result, index) in displayResults"
          :key="result.path"
          class="result-item"
          :class="{ selected: index === selectedIndex }"
          @click="selectResult(result)"
          @mouseenter="selectedIndex = index"
        >
          <span class="result-icon">{{ result.type === 'note' ? '📄' : '🔍' }}</span>
          <div class="result-content">
            <span class="result-title">{{ result.title }}</span>
            <span class="result-path">{{ result.path }}</span>
            <span v-if="result.snippet" class="result-snippet">{{ result.snippet }}</span>
          </div>
        </div>

        <div v-if="displayResults.length === 0" class="empty-results">
          <p>No results found</p>
        </div>
      </div>
    </div>
  </div>
</template>