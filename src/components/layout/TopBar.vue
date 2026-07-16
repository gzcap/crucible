<script setup lang="ts">
import { ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Setting } from "@element-plus/icons-vue";
import SettingsModal from "./SettingsModal.vue";
import ThemeToggle from "./ThemeToggle.vue";
import VaultSwitcher from "../vault/VaultSwitcher.vue";
const showSettings = ref(false);

function openSettings() {
  showSettings.value = true;
}

function closeSettings() {
  showSettings.value = false;
}

async function handleDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement).closest(".no-drag")) {
    return;
  }
  const window = getCurrentWindow();
  await window.startDragging();
}
</script>

<template>
  <header class="title-bar" @mousedown="handleDragStart">
    <!-- 左 -->
    <!-- 选择文件 -->
    <VaultSwitcher class="resource" />
    <!-- 中 -->
    <div class="title-bar-center">
      <span class="window-title"></span>
    </div>
    <!-- 右 -->
    <div class="title-bar-right">
      <!-- 主题切换 -->
      <ThemeToggle />
      <!-- 设置按钮 -->
      <button
        class="settings-btn no-drag"
        @click="openSettings"
        title="设置"
      >
        <el-icon  :size="18">
          <Setting />
        </el-icon>
      </button>
      
    </div>
  </header>
  
  <SettingsModal :visible="showSettings" @close="closeSettings" />
</template>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--roc-titlebar-height);
  width: 100%;
  background: var(--roc-bg-secondary);
}

.resource {
  margin-left: 66px;
}

.title-bar-center {
  flex: 1;
  text-align: center;
}

.window-title {
  font-size: 12px;
  color: var(--roc-text-secondary);
}

.title-bar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--roc-border);
  background: transparent;
  margin-right: 8px;
  border-radius: 50%;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.settings-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

</style>
