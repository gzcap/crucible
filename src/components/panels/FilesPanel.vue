<script setup lang="ts">
import { ref } from 'vue'
import { ElIcon } from 'element-plus'
import { Refresh, MoreFilled, Document, FolderAdd, Search } from "@element-plus/icons-vue";
import PanelHeader from "./PanelHeader.vue";
import FileTree from "../explorer/FileTree.vue";
import { useNotesStore } from "../../stores/notes";

const notesStore = useNotesStore();
const searchQuery = ref('')

function handleNewNote() {
  notesStore.createNewNote();
}

function handleNewFolder() {
  notesStore.createNewFolder();
}

function handleSearch() {
  notesStore.setSearchQuery(searchQuery.value)
}

function clearSearch() {
  searchQuery.value = ''
  notesStore.setSearchQuery('')
}
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="资源管理器">
      <template #actions>
        <el-tooltip content="新建笔记" placement="bottom">
          <button class="panel-btn" @click="handleNewNote">
            <ElIcon :size="14">
              <Document />
            </ElIcon>
          </button>
        </el-tooltip>
        <el-tooltip content="新建文件夹" placement="bottom">
          <button class="panel-btn" @click="handleNewFolder">
            <ElIcon :size="14">
              <FolderAdd />
            </ElIcon>
          </button>
        </el-tooltip>
        <el-tooltip content="刷新" placement="bottom">
          <button class="panel-btn" @click="notesStore.loadNotes()">
            <ElIcon :size="14">
              <Refresh />
            </ElIcon>
          </button>
        </el-tooltip>
        <el-tooltip content="更多" placement="bottom">
          <button class="panel-btn">
            <ElIcon :size="14">
              <MoreFilled />
            </ElIcon>
          </button>
        </el-tooltip>
      </template>
    </PanelHeader>
    
    <div class="search-bar">
      <div class="search-input-wrapper">
        <ElIcon :size="13" class="search-icon">
          <Search />
        </ElIcon>
        <input
          type="text"
          v-model="searchQuery"
          placeholder="搜索文件..."
          class="search-input"
          @input="handleSearch"
          @keyup.enter="handleSearch"
        />
        <button v-if="searchQuery" class="search-clear" @click="clearSearch">×</button>
      </div>
    </div>

    <FileTree :search-query="searchQuery" />
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.search-bar {
  padding: 8px;
  border-bottom: 1px solid var(--roc-border);
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  background: var(--roc-bg-secondary);
  border-radius: 6px;
  padding: 4px 8px;
  border: 1px solid transparent;
  transition: border-color 0.2s;
}

.search-input-wrapper:focus-within {
  border-color: var(--roc-accent);
}

.search-icon {
  color: var(--roc-text-muted);
  margin-right: 6px;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  font-size: 12px;
  color: var(--roc-text-primary);
}

.search-input::placeholder {
  color: var(--roc-text-muted);
}

.search-clear {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 50%;
  cursor: pointer;
  color: var(--roc-text-muted);
  font-size: 14px;
  transition: background 0.15s, color 0.15s;
}

.search-clear:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
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
