<script setup lang="ts">
import PanelHeader from "./PanelHeader.vue";
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="设置" />
    <div class="empty-state">设置面板</div>
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
