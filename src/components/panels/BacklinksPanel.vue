<script setup lang="ts">
import { computed } from "vue";
import { Link, Document } from "@element-plus/icons-vue";
import PanelHeader from "./PanelHeader.vue";
import { useNotesStore } from "../../stores/notes";

const notesStore = useNotesStore();

/** 当前笔记名（去掉 .md 后缀） */
const currentNoteName = computed(() => {
  if (!notesStore.currentPath) return "";
  const parts = notesStore.currentPath.split("/");
  return parts[parts.length - 1].replace(/\.md$/, "");
});

/** 提取来源的文件夹路径 */
function sourceFolder(source: string): string {
  const idx = source.lastIndexOf("/");
  return idx > -1 ? source.slice(0, idx) : "";
}
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="反向链接" :count="notesStore.backlinks.length" />

    <div class="backlinks-body">
      <!-- 当前笔记标识 -->
      <div v-if="currentNoteName" class="current-note">
        <Document :size="13" />
        <span class="current-note-name">{{ currentNoteName }}</span>
      </div>

      <!-- 反向链接列表 -->
      <div v-if="notesStore.backlinks.length > 0" class="backlinks-list">
        <div
          v-for="backlink in notesStore.backlinks"
          :key="backlink.source"
          class="backlink-item"
          @click="notesStore.openNote(backlink.source)"
        >
          <div class="backlink-header">
            <Link :size="12" class="backlink-icon" />
            <span class="backlink-title">{{ backlink.source_title }}</span>
          </div>
          <div v-if="sourceFolder(backlink.source)" class="backlink-path">
            {{ sourceFolder(backlink.source) }}
          </div>
          <div v-if="backlink.snippet" class="backlink-snippet">
            {{ backlink.snippet }}
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <Link :size="32" class="empty-icon" />
        <p class="empty-title">暂无反向链接</p>
        <p class="empty-desc">
          其他笔记中使用 <code>[[{{ currentNoteName || "笔记名" }}]]</code> 链接到此处时将显示
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.backlinks-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 4px;
}

/* 当前笔记标识 */
.current-note {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  margin-bottom: 4px;
  font-size: 12px;
  color: var(--roc-accent);
  background: rgba(0, 122, 204, 0.06);
  border-radius: 4px;
}

.current-note-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 反向链接列表 */
.backlinks-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.backlink-item {
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  border-left: 2px solid transparent;
}

.backlink-item:hover {
  background: var(--roc-bg-tertiary);
  border-left-color: var(--roc-accent);
}

.backlink-header {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-bottom: 3px;
}

.backlink-icon {
  color: var(--roc-text-muted);
  flex-shrink: 0;
}

.backlink-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--roc-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.backlink-path {
  font-size: 11px;
  color: var(--roc-text-muted);
  margin-bottom: 4px;
  margin-left: 17px;
}

.backlink-snippet {
  font-size: 12px;
  line-height: 1.5;
  color: var(--roc-text-secondary);
  margin-left: 17px;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  text-align: center;
}

.empty-icon {
  color: var(--roc-text-muted);
  opacity: 0.3;
  margin-bottom: 12px;
}

.empty-title {
  font-size: 13px;
  color: var(--roc-text-secondary);
  margin: 0 0 6px 0;
}

.empty-desc {
  font-size: 11px;
  color: var(--roc-text-muted);
  line-height: 1.5;
  margin: 0;
}

.empty-desc code {
  background: var(--roc-bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 10px;
  color: var(--roc-accent);
}
</style>
