<script setup lang="ts">
import { inject, type Ref } from 'vue'
import { useNotesStore } from '../../stores/notes'

interface FileTreeNode {
  name: string
  path: string
  isDir: boolean
  children?: FileTreeNode[]
}

defineProps<{
  nodes: FileTreeNode[]
  level?: number
}>()

const emit = defineEmits<{
  toggle: [path: string]
  open: [path: string]
}>()

defineOptions({ name: 'FileTreeNode' })

const notesStore = useNotesStore()
const expandedDirs = inject<Ref<Set<string>>>('expandedDirs')!

function isExpanded(path: string): boolean {
  return expandedDirs.value.has(path)
}

function isActive(path: string): boolean {
  return notesStore.currentPath === path
}

function sortNodes(a: FileTreeNode, b: FileTreeNode): number {
  if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
  return a.name.localeCompare(b.name)
}
</script>

<template>
  <div class="file-tree-children" :style="{ paddingLeft: level ? '16px' : '0' }">
    <div v-for="node in [...nodes].sort(sortNodes)" :key="node.path">
      <div class="tree-node" :class="{ active: !node.isDir && isActive(node.path) }" @click="node.isDir ? emit('toggle', node.path) : emit('open', node.path)">
        <button v-if="node.isDir" class="dir-toggle" @click.stop="emit('toggle', node.path)">
          {{ isExpanded(node.path) ? '▼' : '▶' }}
        </button>
        <span v-else class="dir-spacer" />
        <span class="node-icon">{{ node.isDir ? '📁' : '📝' }}</span>
        <span class="node-name">{{ node.name }}</span>
      </div>
      <FileTreeNode
        v-if="node.isDir && isExpanded(node.path) && node.children"
        :nodes="node.children"
        :level="(level ?? 0) + 1"
        @toggle="(p) => emit('toggle', p)"
        @open="(p) => emit('open', p)"
      />
    </div>
  </div>
</template>

<style scoped>
.file-tree-children {
  font-size: 13px;
  color: var(--roc-text-primary);
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.tree-node:hover {
  background: var(--roc-bg-tertiary);
}

.tree-node.active {
  background: rgba(0, 122, 204, 0.15);
  color: var(--roc-accent);
}

.dir-toggle {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: var(--roc-text-muted);
  flex-shrink: 0;
}

.dir-spacer {
  width: 16px;
  flex-shrink: 0;
}

.node-icon {
  flex-shrink: 0;
  font-size: 14px;
}

.node-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
