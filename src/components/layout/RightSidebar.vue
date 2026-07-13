<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import {
  FolderOpened,
  Search,
  CollectionTag,
  Link,
  List,
  Connection,
  Setting,
} from "@element-plus/icons-vue";
import { useLayoutStore } from "../../stores/layout";
import FilesPanel from "../panels/FilesPanel.vue";
import SearchPanel from "../panels/SearchPanel.vue";
import TagsPanel from "../panels/TagsPanel.vue";
import BacklinksPanel from "../panels/BacklinksPanel.vue";
import OutlinePanel from "../panels/OutlinePanel.vue";
import SettingsPanel from "../panels/SettingsPanel.vue";

const router = useRouter();
const layoutStore = useLayoutStore();

const activeTab = ref("files");

const navItems = [
  { id: "files", icon: FolderOpened, label: "资源管理器" },
  { id: "search", icon: Search, label: "搜索" },
  { id: "tags", icon: CollectionTag, label: "标签" },
  { id: "backlinks", icon: Link, label: "反向链接" },
  { id: "outline", icon: List, label: "大纲" },
  { id: "graph", icon: Connection, label: "图谱" },
  { id: "settings", icon: Setting, label: "设置" },
];

function handleNavClick(id: string) {
  if (activeTab.value === id) {
    layoutStore.rightSidebarVisible = !layoutStore.rightSidebarVisible;
  } else {
    activeTab.value = id;
    layoutStore.rightSidebarVisible = true;
  }
  if (id === "graph") {
    router.push("/graph");
  } else {
    router.push("/");
  }
}
</script>

<template>
  <aside class="sidebar">
    <!-- 右侧工具栏具体内容 -->
    <Transition name="slide-right">
      <div v-show="layoutStore.rightSidebarVisible" class="sidebar-content">
        <FilesPanel v-if="activeTab === 'files'" />
        <SearchPanel v-else-if="activeTab === 'search'" />
        <TagsPanel v-else-if="activeTab === 'tags'" />
        <BacklinksPanel v-else-if="activeTab === 'backlinks'" />
        <OutlinePanel v-else-if="activeTab === 'outline'" />
        <SettingsPanel v-else-if="activeTab === 'settings'" />
      </div>
    </Transition>
    <!-- 右侧工具栏 -->
    <div class="sidebar-right">
      <nav class="activity-bar">
        <button
          v-for="item in navItems"
          :key="item.id"
          class="activity-item"
          :class="{ active: activeTab === item.id }"
          @click="handleNavClick(item.id)"
          :title="item.label"
        >
          <component :is="item.icon" class="activity-icon" />
        </button>
      </nav>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: row;
  gap: 0;
  width: var(--roc-sidebar-width);
  height: var(--roc-sidebar-height);
}

.sidebar-right {
  width: 39px;
  height: var(--roc-sidebar-height);
  background: var(--roc-bg-secondary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.activity-bar {
  width: var(--roc-activity-bar-width);
  background: var(--roc-bg-secondary);
  display: flex;
  padding: 8px 0 0 0;
  flex-direction: column;
  align-items: center;
  gap: 15px;
  flex-shrink: 0;
}

.activity-item {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 18px;
  color: var(--roc-text-secondary);
  transition:
    background 0.15s,
    color 0.15s;
}

.activity-item:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

.activity-item.active {
  background: var(--roc-bg-active);
  color: var(--roc-accent);
}

.activity-icon {
  font-size: 20px;
}

.sidebar-content {
  width: var(--roc-right-sidebar-content-width, var(--roc-sidebar-content-width));
  height: var(--roc-sidebar-height);
  border-radius: var(--roc-border-radius);
  background: var(--roc-bg-primary);
}

.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.slide-right-enter-from,
.slide-right-leave-to {
  width: 0;
  opacity: 0;
}
</style>
