<script setup lang="ts">
import { ref, onUnmounted } from 'vue'

/**
 * 可复用的拖动分隔条组件
 * 在鼠标按下时捕获起始位置，移动时持续发出相对于起始点的累计偏移量 deltaX
 */
const emit = defineEmits<{
  (e: 'resize-start'): void
  (e: 'resize', deltaX: number): void
  (e: 'resize-end'): void
}>()

const isDragging = ref(false)
let startX = 0

function onMouseDown(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  isDragging.value = true
  startX = e.clientX
  emit('resize-start')
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  emit('resize', e.clientX - startX)
}

function onMouseUp() {
  if (!isDragging.value) return
  isDragging.value = false
  emit('resize-end')
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
})
</script>

<template>
  <div
    class="resizer"
    :class="{ active: isDragging }"
    @mousedown="onMouseDown"
  >
    <div class="resizer-handle" />
  </div>
</template>

<style scoped>
.resizer {
  flex-shrink: 0;
  width: 6px;
  height: 100%;
  position: relative;
  cursor: col-resize;
  background: transparent;
  z-index: 10;
}

.resizer::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  background: transparent;
  transition: background 0.15s, width 0.15s;
}

.resizer:hover::before,
.resizer.active::before {
  width: 2px;
  background: var(--roc-accent);
}

.resizer-handle {
  position: absolute;
  inset: 0;
}
</style>
