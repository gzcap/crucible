<script setup lang="ts">
import { ref } from "vue";
import { Search } from "@element-plus/icons-vue";
import PanelHeader from "./PanelHeader.vue";
import { useSearchStore } from "../../stores/search";

const searchStore = useSearchStore();
const searchQuery = ref("");
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="搜索" />
    <div class="search-bar">
      <el-input
        v-model="searchQuery"
        placeholder="搜索文件..."
        size="small"
        :prefix-icon="Search"
        @input="searchStore.performSearch(searchQuery)"
        autofocus
      />
    </div>
    <div v-if="searchQuery.length > 0" class="search-results">
      <div
        v-for="(result, index) in searchStore.results"
        :key="index"
        class="search-result-item"
      >
        <span class="search-result-icon">📄</span>
        <span class="search-result-title">{{ result.title }}</span>
      </div>
      <div v-if="searchStore.results.length === 0" class="empty-state">
        未找到匹配结果
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

.search-bar {
  padding: 8px;
}

.search-results {
  padding: 4px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.search-result-item:hover {
  background: var(--roc-bg-tertiary);
}

.search-result-icon {
  font-size: 14px;
}

.search-result-title {
  font-size: 13px;
  color: var(--roc-text-primary);
}

.empty-state {
  padding: 16px;
  text-align: center;
  color: var(--roc-text-muted);
  font-size: 12px;
}
</style>
