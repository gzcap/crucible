import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { EditorMode } from '../lib/types'

export const useEditorStore = defineStore('editor', () => {
  const mode = ref<EditorMode>('wysiwyg')
  const editorView = ref<any>(null)

  function setMode(newMode: EditorMode) {
    mode.value = newMode
  }

  function setEditorView(view: any) {
    editorView.value = view
  }

  function toggleMode() {
    const modes: EditorMode[] = ['wysiwyg', 'source']
    const currentIndex = modes.indexOf(mode.value)
    mode.value = modes[(currentIndex + 1) % modes.length]
  }

  return {
    mode,
    editorView,
    setMode,
    setEditorView,
    toggleMode
  }
})