<script setup lang="ts">
import { useNotesStore } from '../../stores/notes'
import { useEditorStore } from '../../stores/editor'

const notesStore = useNotesStore()
const editorStore = useEditorStore()

const modeLabels = {
  source: 'Source',
  preview: 'Preview',
  split: 'Split'
}
</script>

<template>
  <div class="editor-toolbar">
    <div class="toolbar-left">
      <button class="toolbar-btn" @click="notesStore.saveNote(notesStore.currentPath!, notesStore.currentContent)">
        <span class="btn-icon">💾</span>
        Save
      </button>
    </div>
    
    <div class="toolbar-center">
      <button
        v-for="(label, mode) in modeLabels"
        :key="mode"
        class="toolbar-btn mode-btn"
        :class="{ active: editorStore.mode === mode }"
        @click="editorStore.setMode(mode as any)"
      >
        {{ label }}
      </button>
    </div>
    
    <div class="toolbar-right">
      <button class="toolbar-btn" @click="notesStore.loadBacklinks(notesStore.currentPath!)">
        <span class="btn-icon">🔗</span>
        Refresh Backlinks
      </button>
    </div>
  </div>
</template>