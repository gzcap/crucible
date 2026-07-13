<script setup lang="ts">
import { Refresh, MoreFilled } from "@element-plus/icons-vue";
import PanelHeader from "./PanelHeader.vue";
import FileTree from "../explorer/FileTree.vue";
import { useNotesStore } from "../../stores/notes";

const notesStore = useNotesStore();
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="资源管理器">
      <template #actions>
        <button class="panel-btn" title="新建文件">📄</button>
        <button class="panel-btn" title="新建文件夹">📁</button>
        <button class="panel-btn" @click="notesStore.loadNotes()" title="刷新">
          <Refresh :size="14" />
        </button>
        <button class="panel-btn" title="更多">
          <MoreFilled :size="14" />
        </button>
      </template>
    </PanelHeader>
    <FileTree />
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.panel-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 12px;
  color: var(--roc-text-muted);
  transition:
    background 0.15s,
    color 0.15s;
}

.panel-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}
</style>
