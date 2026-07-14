<script setup lang="ts">
import { inject, type Ref, ref, computed } from 'vue'
import { ElMessageBox } from 'element-plus'
import { ArrowDown, ArrowRight, Folder, FolderOpened, Document, FolderAdd } from '@element-plus/icons-vue'
import { useNotesStore } from '../../stores/notes'
import ContextMenu from './ContextMenu.vue'

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
const contextMenu = ref<{
  visible: boolean
  x: number
  y: number
  node: FileTreeNode | null
}>({ visible: false, x: 0, y: 0, node: null })

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

function getChildCount(node: FileTreeNode): number {
  if (!node.isDir || !node.children) return 0
  let count = 0
  const stack = [...node.children]
  while (stack.length > 0) {
    const child = stack.pop()!
    if (child.isDir && child.children) {
      stack.push(...child.children)
    } else if (!child.isDir) {
      count++
    }
  }
  return count
}

function handleContextMenu(event: MouseEvent, node: FileTreeNode) {
  event.preventDefault()
  event.stopPropagation()
  
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    node
  }
}

function closeContextMenu() {
  contextMenu.value.visible = false
}

async function deleteNode(node: FileTreeNode) {
  const typeText = node.isDir ? '文件夹' : '文件'
  const title = `确认删除${typeText}`
  const message = `确定要删除 ${typeText} "${node.name}" 吗？此操作无法撤销。`

  try {
    await ElMessageBox.confirm(message, title, {
      confirmButtonText: '确认删除',
      cancelButtonText: '取消',
      type: 'warning',
      confirmButtonClass: 'el-button--danger'
    })

    if (node.isDir) {
      await notesStore.removeFolder(node.path)
    } else {
      await notesStore.removeNote(node.path)
    }
  } catch {
  }
}

async function createNoteInFolder(path: string) {
  await notesStore.createNewNote()
}

async function createFolderInFolder(path: string) {
  await notesStore.createNewFolder()
}
</script>

<template>
  <div class="file-tree-children" :style="{ paddingLeft: level ? '16px' : '0' }">
    <div v-for="node in [...nodes].sort(sortNodes)" :key="node.path">
      <div
        class="tree-node"
        :class="{ active: !node.isDir && isActive(node.path), dragging: false }"
        @click="node.isDir ? emit('toggle', node.path) : emit('open', node.path)"
        @contextmenu="(e) => handleContextMenu(e, node)"
      >
        <span v-if="node.isDir" class="dir-toggle" @click.stop="emit('toggle', node.path)">
          <ArrowDown v-if="isExpanded(node.path)" :size="12" />
          <ArrowRight v-else :size="12" />
        </span>
        <span v-else class="dir-spacer" />
        
        <span class="node-icon">
          <FolderOpened v-if="node.isDir && isExpanded(node.path)" :size="14" class="icon-dir-open" />
          <Folder v-else-if="node.isDir" :size="14" class="icon-dir" />
          <Document v-else :size="13" class="icon-file" />
        </span>
        
        <span class="node-name">
          {{ node.name }}
          <span v-if="node.isDir && getChildCount(node) > 0" class="node-count">
            {{ getChildCount(node) }}
          </span>
        </span>
        
        <span class="node-actions">
          <button 
            v-if="node.isDir" 
            class="node-action-btn" 
            title="新建文件"
            @click.stop="createNoteInFolder(node.path)"
          >
            <Document :size="12" />
          </button>
          <button 
            v-if="node.isDir" 
            class="node-action-btn" 
            title="新建文件夹"
            @click.stop="createFolderInFolder(node.path)"
          >
            <FolderAdd :size="12" />
          </button>
        </span>
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

  <ContextMenu
    v-if="contextMenu.visible && contextMenu.node"
    :items="[
      {
        label: '新建文件',
        icon: Document,
        action: () => createNoteInFolder(contextMenu.node!.path)
      },
      {
        label: '新建文件夹',
        icon: FolderAdd,
        action: () => createFolderInFolder(contextMenu.node!.path)
      },  
      {
        label: contextMenu.node!.isDir ? '删除文件夹' : '删除文件',
        icon: Document,
        action: () => deleteNode(contextMenu.node!)
      }
    ]"
    :x="contextMenu.x"
    :y="contextMenu.y"
    @close="closeContextMenu"
  />
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
  padding: 4px 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s, padding 0.15s;
  position: relative;
}

.tree-node:hover {
  background: var(--roc-bg-tertiary);
}

.tree-node:hover .node-actions {
  opacity: 1;
}

.tree-node.active {
  background: rgba(0, 122, 204, 0.15);
  color: var(--roc-accent);
}

.tree-node.active .icon-file {
  color: var(--roc-accent);
}

.dir-toggle {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--roc-text-muted);
  flex-shrink: 0;
  transition: transform 0.15s;
}

.dir-toggle:hover {
  color: var(--roc-text-primary);
}

.dir-spacer {
  width: 16px;
  flex-shrink: 0;
}

.node-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s;
}

.icon-dir {
  color: var(--roc-text-muted);
}

.icon-dir-open {
  color: var(--roc-accent);
}

.icon-file {
  color: var(--roc-text-secondary);
}

.node-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 6px;
}

.node-count {
  font-size: 11px;
  color: var(--roc-text-muted);
  background: var(--roc-bg-tertiary);
  padding: 1px 5px;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
}

.node-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
  flex-shrink: 0;
}

.node-action-btn {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  color: var(--roc-text-muted);
  transition: background 0.15s, color 0.15s;
}

.node-action-btn:hover {
  background: var(--roc-bg-secondary);
  color: var(--roc-text-primary);
}

.tree-node.dragging {
  opacity: 0.5;
  background: var(--roc-accent) !important;
}
</style>
