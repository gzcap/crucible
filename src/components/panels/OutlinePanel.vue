<script setup lang="ts">
import PanelHeader from "./PanelHeader.vue";
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="大纲" />
    <div class="empty-state">大纲面板</div>
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.empty-state {
  padding: 16px;
  text-align: center;
  color: var(--roc-text-muted);
  font-size: 12px;
}
</style>
