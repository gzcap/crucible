<script setup lang="ts">
import { computed } from 'vue'
import { useVaultStore } from '../../stores/vault'
import { useNotesStore } from '../../stores/notes'

const vaultStore = useVaultStore()
const notesStore = useNotesStore()

const vaultName = computed(() => vaultStore.currentVault?.name || '')
const wordCount = computed(() => {
  if (!notesStore.currentContent) return 0
  return notesStore.currentContent.trim().split(/\s+/).filter(w => w.length > 0).length
})
const charCount = computed(() => {
  if (!notesStore.currentContent) return 0
  return notesStore.currentContent.length
})
const lineCount = computed(() => {
  if (!notesStore.currentContent) return 0
  return notesStore.currentContent.split('\n').length
})
const hasDirty = computed(() => notesStore.tabs.some(tab => tab.dirty))
</script>

<template>
  <footer class="bottom-sidebar">
    <div class="status-left">
      <span class="status-item git-branch">
        <span class="branch-icon">⌘</span>
        <span>main</span>
      </span>
      <span class="status-item sync-status">
        <span>↻</span>
        <span>点击以重新更新</span>
      </span>
      <span class="status-item">Ln {{ lineCount }}, Col 1</span>
      <span class="status-item">字数: {{ wordCount }}</span>
    </div>
    <div class="status-right">
      <span class="status-item">{{ vaultName }}</span>
      <span class="status-item">Markdown</span>
      <span class="status-item">UTF-8</span>
      <span class="status-item">空格: 2</span>
      <span class="status-item">LF</span>
      <span class="status-item">Auto</span>
      <span class="status-item" v-if="hasDirty">⚠</span>
      <span class="status-item settings-icon">⚙️</span>
    </div>
  </footer>
</template>

<style scoped>
.bottom-sidebar {
  width: var(--roc-bottom-sidebar-width);
  height: var(--roc-bottom-sidebar-height);
  background: var(--el-bg-color-overlay);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.status-bar {
  display: flex;
  align-items: center;
  border-top: none;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item {
  color: var(--roc-text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: .8rem;
}

.status-item:hover {
  color: var(--roc-text-primary);
}

</style>