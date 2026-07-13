<script setup lang="ts">
import { ref, onErrorCaptured } from "vue";
import LeftSidebar from "./LeftSidebar.vue";

import MainContent from "./MainContent.vue";
import RightSidebar from "./RightSidebar.vue";
import BottomBar from "./BottomBar.vue";
import TopBar from "./TopBar.vue";
import Resizer from "./Resizer.vue";
import { useLayoutStore } from "../../stores/layout";

const showBacklinks = ref(true);
const layoutStore = useLayoutStore();

/** 捕获所有子组件渲染错误，防止错误冒泡导致整个布局被卸载 */
onErrorCaptured((err, instance, info) => {
  console.error('[AppLayout] errorCaptured:', err, info)
  return false
})

// 拖动起始时记录基准宽度，移动时基于基准 + 偏移量计算
const baseLeftWidth = ref(0);
const baseRightWidth = ref(0);

function onLeftResizeStart() {
  baseLeftWidth.value = layoutStore.leftSidebarWidth;
}

function onLeftResize(delta: number) {
  // 左侧面板：向右拖动（delta > 0）增加宽度
  layoutStore.setLeftSidebarWidth(baseLeftWidth.value + delta);
}

function onLeftResizeEnd() {
  layoutStore.commit();
}

function onRightResizeStart() {
  baseRightWidth.value = layoutStore.rightSidebarWidth;
}

function onRightResize(delta: number) {
  // 右侧面板：向左拖动（delta < 0）增加宽度，因此取反
  layoutStore.setRightSidebarWidth(baseRightWidth.value - delta);
}

function onRightResizeEnd() {
  layoutStore.commit();
}
</script>

<template>
  <div class="app-layout">
    <!-- 顶部栏 -->
    <TopBar />
    <div
      class="main-layout"
      :style="{
        '--roc-sidebar-content-width': layoutStore.leftSidebarWidth + 'px',
        '--roc-right-sidebar-content-width': layoutStore.rightSidebarWidth + 'px',
      }"
    >
      <!-- 左侧 -->
      <LeftSidebar />
      <!-- 左侧拖动条 -->
      <Resizer
        v-if="layoutStore.leftSidebarVisible"
        @resize-start="onLeftResizeStart"
        @resize="onLeftResize"
        @resize-end="onLeftResizeEnd"
      />
      <!-- 中间 -->
      <MainContent />
      <!-- 右侧拖动条 -->
      <Resizer
        v-if="showBacklinks && layoutStore.rightSidebarVisible"
        @resize-start="onRightResizeStart"
        @resize="onRightResize"
        @resize-end="onRightResizeEnd"
      />
      <!-- 右侧 -->
      <RightSidebar v-if="showBacklinks" />
    </div>
    <!-- 底部边栏 -->
    <BottomBar />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background-color: var(--roc-bg-secondary);
}
</style>
