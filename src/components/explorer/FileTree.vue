<script setup lang="ts">
import { ref, computed, provide } from 'vue'
import FileTreeNode from './FileTreeNode.vue'
import ContextMenu from './ContextMenu.vue'
import { useNotesStore } from '../../stores/notes'
import { Document, FolderAdd, FolderOpened, Folder } from '@element-plus/icons-vue'

const props = defineProps<{
  searchQuery?: string
}>()

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

function handleContextMenu(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('.tree-node')) {
    return
  }
  event.preventDefault()
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY
  }
}

function closeContextMenu() {
  contextMenu.value.visible = false
}

function expandAll() {
  const visited = new Set<string>()
  function visit(node: any) {
    if (!node || visited.has(node.path)) return
    visited.add(node.path)
    if (node.isDir && node.children && node.children.length > 0) {
      expandedDirs.value.add(node.path)
      for (const child of node.children) {
        visit(child)
      }
    }
  }
  visit(notesStore.fileTree)
}

function collapseAll() {
  expandedDirs.value.clear()
}

const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
}>({ visible: false, x: 0, y: 0 })

const rootNodes = computed(() => {
  const tree = props.searchQuery ? notesStore.filteredTree : notesStore.fileTree
  return tree?.children || []
})
</script>

<template>
  <div class="file-tree" @contextmenu="handleContextMenu">
    <div class="tree-toolbar" v-if="rootNodes.length > 0">
      <button class="tree-tool-btn" @click="expandAll" title="展开所有">
        <FolderOpened :size="12" />
      </button>
      <button class="tree-tool-btn" @click="collapseAll" title="折叠所有">
        <Folder :size="12" />
      </button>
    </div>

    <FileTreeNode
      v-if="rootNodes.length > 0"
      :nodes="rootNodes"
      @toggle="toggleDir"
      @open="openNote"
    />
    <div v-else class="empty-tree">
      <p>{{ searchQuery ? '未找到匹配的文件' : '暂无文件' }}</p>
      <p class="empty-hint" v-if="!searchQuery">右键点击空白处新建文件或文件夹</p>
    </div>
  </div>

  <ContextMenu
    v-if="contextMenu.visible"
    :items="[
      {
        label: '新建文件',
        icon: Document,
        action: () => notesStore.createNewNote()
      },
      {
        label: '新建文件夹',
        icon: FolderAdd,
        action: () => notesStore.createNewFolder()
      }
    ]"
    :x="contextMenu.x"
    :y="contextMenu.y"
    @close="closeContextMenu"
  />
</template>

<style scoped>
.file-tree {
  padding: 4px;
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.tree-toolbar {
  display: flex;
  gap: 2px;
  padding: 4px 6px;
  border-bottom: 1px solid var(--roc-border);
  margin-bottom: 4px;
}

.tree-tool-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--roc-text-muted);
  transition: background 0.15s, color 0.15s;
}

.tree-tool-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

.empty-tree {
  padding: 16px;
  text-align: center;
  color: var(--roc-text-muted);
  font-size: 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.empty-hint {
  font-size: 11px;
  opacity: 0.7;
  margin-top: 4px;
}
</style>
