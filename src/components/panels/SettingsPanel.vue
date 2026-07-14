<script setup lang="ts">
import PanelHeader from "./PanelHeader.vue";
import PluginManager from "../plugins/PluginManager.vue";
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="设置" />
    <div class="settings-content">
      <PluginManager />
    </div>
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.settings-content {
  flex: 1;
  overflow: hidden;
}
</style>
