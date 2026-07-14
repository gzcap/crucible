<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import PanelHeader from '../panels/PanelHeader.vue'

interface Props {
  panel: {
    id: string
    name: string
    icon: string
    component?: any
    render?: (container: HTMLElement) => void
  }
}

const props = defineProps<Props>()
const containerRef = ref<HTMLElement | null>(null)

onMounted(() => {
  renderPanel()
})

watch(() => props.panel, () => {
  renderPanel()
}, { deep: true })

function renderPanel() {
  if (!containerRef.value) return
  
  containerRef.value.innerHTML = ''
  
  if (props.panel.render) {
    props.panel.render(containerRef.value)
  } else if (props.panel.component) {
    if (typeof props.panel.component === 'string') {
      containerRef.value.innerHTML = props.panel.component
    } else {
      try {
        const el = document.createElement('div')
        el.innerHTML = props.panel.component
        if (el.firstChild) {
          containerRef.value.appendChild(el.firstChild)
        }
      } catch {
        containerRef.value.textContent = '无法渲染面板内容'
      }
    }
  }
}
</script>

<template>
  <div class="plugin-panel">
    <PanelHeader :title="panel.name">
      <template #actions>
        <span class="panel-icon">{{ panel.icon }}</span>
      </template>
    </PanelHeader>
    <div ref="containerRef" class="plugin-panel-content"></div>
  </div>
</template>

<style scoped>
.plugin-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.plugin-panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.panel-icon {
  font-size: 16px;
}
</style>