<script setup lang="ts">
import { computed } from 'vue'

interface FileTreeNode {
  name: string
  path: string
  isDir: boolean
  children?: FileTreeNode[]
}

const props = defineProps<{
  node: FileTreeNode
  expandedDirs: Set<string>
}>()

const emit = defineEmits<{
  toggle: [path: string]
  open: [path: string]
}>()

function isExpanded(path: string): boolean {
  return props.expandedDirs.has(path)
}

function toggleDir(path: string) {
  emit('toggle', path)
}

function openNote(path: string) {
  emit('open', path)
}

function sortNodes(a: FileTreeNode, b: FileTreeNode): number {
  if (a.isDir !== b.isDir) {
    return a.isDir ? -1 : 1
  }
  return a.name.localeCompare(b.name)
}

const sortedChildren = computed(() => {
  return props.node.children?.sort(sortNodes) || []
})
</script>

<template>
  <div v-if="node.isDir" class="tree-node">
    <button class="dir-toggle" @click="toggleDir(node.path)">
      {{ isExpanded(node.path) ? '▼' : '▶' }}
    </button>
    <span class="node-icon dir-icon">📁</span>
    <span class="node-name" @click="toggleDir(node.path)">{{ node.name }}</span>
    
    <div v-if="isExpanded(node.path)" class="children">
      <FileTreeNodeComponent
        v-for="child in sortedChildren"
        :key="child.path"
        :node="child"
        :expanded-dirs="expandedDirs"
        @toggle="toggleDir"
        @open="openNote"
      />
    </div>
  </div>

  <div v-else class="tree-node file-node">
    <span class="node-icon file-icon">📝</span>
    <span
      class="node-name"
      @click="openNote(node.path)"
    >
      {{ node.name }}
    </span>
  </div>
</template>