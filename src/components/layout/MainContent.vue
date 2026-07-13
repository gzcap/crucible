<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import EditorPane from "../editor/EditorPane.vue";
import GraphView from "../graph/GraphView.vue";
import { useNotesStore } from "../../stores/notes";

const route = useRoute();
const notesStore = useNotesStore();

const isGraphView = computed(() => route.name === "Graph");

async function handleDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement).closest(".no-drag")) {
    return;
  }
  const window = getCurrentWindow();
  await window.startDragging();
}
</script>
<template>
  <main class="main-content">
    <div class="tabs-bar" @mousedown="handleDragStart">
      <div
        v-for="tab in notesStore.tabs"
        :key="tab.path"
        class="tab no-drag"
        :class="{ active: tab.path === notesStore.currentPath }"
        @click="notesStore.openNote(tab.path)"
      >
        <span class="tab-title">{{ tab.title }}</span>
        <span v-if="tab.dirty" class="tab-dirty">●</span>
        <button
          class="tab-close no-drag"
          @click.stop="notesStore.closeTab(tab.path)"
        >
          ×
        </button>
      </div>
    </div>

    <div class="editor-area" v-if="notesStore.currentPath && !isGraphView">
      <EditorPane />
    </div>

    <GraphView v-else-if="isGraphView" />

    <div class="empty-state" v-else>
      <p>Open a note or create a new one to get started</p>
    </div>
  </main>
</template>

<style scoped>
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  width: auto;
  height: var(--roc-sidebar-height);
  background: var(--roc-bg-primary);
  border-radius: var(--roc-border-radius);
}

.tabs-bar {
  display: flex;
  height: var(--roc-tabs-height);
  overflow-x: auto;
  overflow-y: hidden;
  padding: 0 0 0 8px;
  align-items: center;
  gap: 4px;
}

.tabs-bar::-webkit-scrollbar {
  display: none;
}

.new-tab-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 16px;
  color: var(--roc-text-secondary);
  transition:
    background 0.15s,
    color 0.15s;
}

.new-tab-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  background: transparent;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition:
    background 0.15s,
    border-color 0.15s;
  color: var(--roc-text-secondary);
  min-width: 100px;
  max-width: 200px;
  border-radius: 4px 4px 0 0;
}

.tab:hover {
  background: var(--roc-bg-tertiary);
}

.tab.active {
  border-bottom-color: var(--roc-accent);
  background: var(--roc-bg-primary);
  color: var(--roc-text-primary);
}

.tab-icon {
  font-size: 13px;
}

.tab-title {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.tab-dirty {
  color: var(--roc-accent);
  font-weight: 700;
  font-size: 10px;
}

.tab-close {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 16px;
  opacity: 0;
  transition:
    opacity 0.15s,
    background 0.15s;
}

.tab:hover .tab-close {
  opacity: 0.5;
}

.tab-close:hover {
  opacity: 1;
  background: var(--roc-bg-hover);
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--roc-bg-secondary);
  border-bottom: 1px solid var(--roc-border);
  height: var(--roc-toolbar-height);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 13px;
  color: var(--roc-text-secondary);
  transition:
    background 0.15s,
    color 0.15s;
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--roc-text-secondary);
}

.breadcrumb-item {
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 3px;
  transition: background 0.15s;
}

.breadcrumb-item:hover {
  background: var(--roc-bg-tertiary);
}

.breadcrumb-item.active {
  color: var(--roc-text-primary);
  font-weight: 500;
}

.editor-area {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.editor-pane-wrapper {
  flex: 1;
  display: flex;
}

.editor-pane-wrapper.split-mode {
  flex-direction: row;
}

.editor-pane-wrapper.split-mode .editor-pane,
.editor-pane-wrapper.split-mode .preview-pane {
  flex: 1;
}

.editor-pane {
  flex: 1;
  height: 100%;
  background: var(--roc-bg-primary);
}

.preview-pane {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  background: var(--roc-bg-primary);
  border-left: 1px solid var(--roc-border);
}

.preview-content {
  padding: 24px 32px;
  max-width: 800px;
  margin: 0 auto;
}

.preview-content :deep(h1) {
  font-size: 28px;
  margin-top: 0;
  margin-bottom: 16px;
  font-weight: 700;
  color: var(--roc-text-primary);
}
.preview-content :deep(h2) {
  font-size: 22px;
  margin-top: 24px;
  margin-bottom: 12px;
  font-weight: 600;
  color: var(--roc-text-primary);
  border-bottom: 1px solid var(--roc-border);
  padding-bottom: 4px;
}
.preview-content :deep(h3) {
  font-size: 18px;
  margin-top: 20px;
  margin-bottom: 10px;
  font-weight: 600;
  color: var(--roc-text-primary);
}
.preview-content :deep(h4),
:deep(h5),
:deep(h6) {
  font-size: 16px;
  margin-top: 16px;
  margin-bottom: 8px;
  color: var(--roc-text-primary);
}
.preview-content :deep(p) {
  margin: 10px 0;
  color: var(--roc-text-secondary);
}
.preview-content :deep(ul),
:deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}
.preview-content :deep(li) {
  margin: 4px 0;
  color: var(--roc-text-secondary);
}
.preview-content :deep(code) {
  background: var(--roc-bg-tertiary);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: "SF Mono", Monaco, Consolas, monospace;
  font-size: 0.9em;
  color: var(--roc-link-color);
}
.preview-content :deep(pre) {
  background: var(--roc-bg-secondary);
  padding: 16px;
  border-radius: 6px;
  overflow-x: auto;
  border: 1px solid var(--roc-border);
}
.preview-content :deep(pre code) {
  background: none;
  padding: 0;
}
.preview-content :deep(blockquote) {
  border-left: 3px solid var(--roc-accent);
  padding-left: 16px;
  color: var(--roc-text-muted);
  margin: 12px 0;
  background: rgba(0, 122, 204, 0.05);
  padding: 8px 16px;
  border-radius: 0 4px 4px 0;
}
.preview-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 12px 0;
}
.preview-content :deep(th),
:deep(td) {
  border: 1px solid var(--roc-border);
  padding: 8px 12px;
  text-align: left;
}
.preview-content :deep(th) {
  background: var(--roc-bg-secondary);
  font-weight: 600;
}
.preview-content :deep(img) {
  max-width: 100%;
  border-radius: 6px;
}

.wikilink {
  color: var(--roc-wikilink-color);
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-color 0.15s;
}

.wikilink:hover {
  border-bottom-color: var(--roc-wikilink-color);
}

.hashtag {
  color: var(--roc-tag-color);
  text-decoration: none;
}

.hashtag:hover {
  text-decoration: underline;
}
</style>
