<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Document, FolderAdd, Trash, Refresh } from '@element-plus/icons-vue'

interface MenuItem {
  label: string
  icon?: typeof Document
  action: () => void
  divider?: boolean
}

const props = defineProps<{
  items: MenuItem[]
  x: number
  y: number
}>()

const emit = defineEmits<{
  close: []
}>()

const visible = ref(true)

function handleClick() {
  visible.value = false
  emit('close')
}

function handleOutsideClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.context-menu')) {
    visible.value = false
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick)
})

onUnmounted(() => {
  document.removeEventListener('click', handleOutsideClick)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="context-menu"
      :style="{ left: `${x}px`, top: `${y}px` }"
    >
      <div v-for="(item, index) in items" :key="index" class="menu-item-wrapper">
        <div v-if="item.divider" class="menu-divider" />
        <button
          v-else
          class="menu-item"
          @click="item.action(); handleClick()"
        >
          <component v-if="item.icon" :is="item.icon" :size="14" class="menu-icon" />
          <span class="menu-label">{{ item.label }}</span>
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--roc-bg-primary);
  border: 1px solid var(--roc-border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  padding: 4px;
  min-width: 160px;
}

.menu-item-wrapper {
  display: flex;
  flex-direction: column;
}

.menu-divider {
  height: 1px;
  background: var(--roc-border);
  margin: 4px 8px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--roc-text-primary);
  transition: background 0.15s;
}

.menu-item:hover {
  background: var(--roc-bg-tertiary);
}

.menu-icon {
  flex-shrink: 0;
  color: var(--roc-text-secondary);
}

.menu-label {
  text-align: left;
}
</style>
