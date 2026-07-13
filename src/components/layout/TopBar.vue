<script setup lang="ts">
import { ref, computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  Moon,
  Sunny,
  Monitor,
  Minus,
  FullScreen,
  Close,
} from "@element-plus/icons-vue";
import { useThemeStore, type ThemeMode } from "../../stores/theme";
import VaultSwitcher from "../vault/VaultSwitcher.vue";
const themeStore = useThemeStore();

async function handleDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement).closest(".no-drag")) {
    return;
  }
  const window = getCurrentWindow();
  await window.startDragging();
}

async function minimizeWindow() {
  const window = getCurrentWindow();
  await window.minimize();
}

async function toggleMaximizeWindow() {
  const window = getCurrentWindow();
  await window.toggleMaximize();
}

async function closeWindow() {
  const window = getCurrentWindow();
  await window.close();
}

function handleThemeChange(value: ThemeMode) {
  themeStore.setMode(value);
}
</script>

<template>
  <header class="title-bar" @mousedown="handleDragStart">
    <!-- 选择文件 -->
    <VaultSwitcher class="resource" />

    <div class="title-bar-center">
      <span class="window-title">ROC Notes</span>
    </div>
    <div class="title-bar-right">
      <el-select
        v-model="themeStore.mode"
        class="theme-selector"
        size="small"
        :popper-class="'theme-dropdown'"
        @change="handleThemeChange"
      >
        <el-option label="亮色" value="light">
          <Sunny :size="14" />
        </el-option>
        <el-option label="暗色" value="dark">
          <Moon :size="14" />
        </el-option>
        <el-option label="跟随系统" value="system">
          <Monitor :size="14" />
        </el-option>
      </el-select>
      <button
        class="window-control-btn no-drag"
        @click="minimizeWindow"
        title="Minimize"
      >
        <Minus :size="12" />
      </button>
      <button
        class="window-control-btn no-drag"
        @click="toggleMaximizeWindow"
        title="Maximize"
      >
        <FullScreen :size="12" />
      </button>
      <button
        class="window-control-btn close no-drag"
        @click="closeWindow"
        title="Close"
      >
        <Close :size="12" />
      </button>
    </div>
  </header>
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
  margin-left: 60px;
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

.theme-selector {
  width: 100px;
}

.theme-dropdown {
  background: var(--roc-bg-secondary) !important;
  border: 1px solid var(--roc-border) !important;
}

.theme-dropdown .el-select-dropdown__item {
  color: var(--roc-text-primary);
}

.theme-dropdown .el-select-dropdown__item:hover {
  background: var(--roc-bg-tertiary);
}

.theme-dropdown .el-select-dropdown__item.selected {
  background: rgba(0, 122, 204, 0.15);
  color: var(--roc-accent);
}
</style>
