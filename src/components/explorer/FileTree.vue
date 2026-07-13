<script setup lang="ts">
import { ref, computed, provide } from 'vue'
import FileTreeNode from './FileTreeNode.vue'
import { useNotesStore } from '../../stores/notes'

const notesStore = useNotesStore()

const expandedDirs = ref<Set<string>>(new Set())

provide('expandedDirs', expandedDirs)

function toggleDir(path: string) {
  if (expandedDirs.value.has(path)) {
    expandedDirs.value.delete(path)
  } else {
    expandedDirs.value.add(path)
  }
}

function openNote(path: string) {
  notesStore.openNote(path)
}

const rootNodes = computed(() => notesStore.fileTree?.children || [])
</script>

<template>
  <div class="file-tree">
    <FileTreeNode
      v-if="rootNodes.length > 0"
      :nodes="rootNodes"
      @toggle="toggleDir"
      @open="openNote"
    />
    <div v-else class="empty-tree">
      <p>暂无文件</p>
    </div>
  </div>
</template>

<style scoped>
.file-tree {
  padding: 4px;
  flex: 1;
  overflow-y: auto;
}

.empty-tree {
  padding: 16px;
  text-align: center;
  color: var(--roc-text-muted);
  font-size: 12px;
}
</style>
