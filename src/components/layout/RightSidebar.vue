<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import {
  FolderOpened,
  Search,
  Link,
  List,
  Connection,
  Setting,
} from "@element-plus/icons-vue";
import { useLayoutStore } from "../../stores/layout";
import { rocApp } from "../../plugins";
import FilesPanel from "../panels/FilesPanel.vue";
import SearchPanel from "../panels/SearchPanel.vue";
import BacklinksPanel from "../panels/BacklinksPanel.vue";
import OutlinePanel from "../panels/OutlinePanel.vue";
import SettingsPanel from "../panels/SettingsPanel.vue";
import PluginPanel from "../plugins/PluginPanel.vue";

const router = useRouter();
const layoutStore = useLayoutStore();

const activeTab = ref("files");
const pluginPanels = ref<Array<{ id: string; name: string; icon: string; component?: any; render?: (container: HTMLElement) => void }>>([]);

interface NavItem {
  id: string
  icon: any
  iconType?: 'component' | 'emoji'
  label: string
}

const navItems = computed<NavItem[]>(() => {
  const items: NavItem[] = [
    { id: "files", icon: FolderOpened, iconType: 'component', label: "资源管理器" },
    { id: "search", icon: Search, iconType: 'component', label: "搜索" },
    { id: "backlinks", icon: Link, iconType: 'component', label: "反向链接" },
    { id: "outline", icon: List, iconType: 'component', label: "大纲" },
    { id: "graph", icon: Connection, iconType: 'component', label: "图谱" },
    { id: "settings", icon: Setting, iconType: 'component', label: "设置" },
  ];
  
  pluginPanels.value.forEach(panel => {
    items.push({
      id: panel.id,
      icon: panel.icon,
      iconType: 'emoji',
      label: panel.name,
    });
  });
  
  return items;
});

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

function handlePanelRegistered(panel: any) {
  pluginPanels.value.push({
    id: panel.id,
    name: panel.name,
    icon: panel.icon,
    component: panel.component,
    render: panel.render,
  });
}

function getActivePluginPanel() {
  return pluginPanels.value.find(p => p.id === activeTab.value);
}

onMounted(() => {
  rocApp.events.on('workspace:sidebar-panel-registered', handlePanelRegistered);
  const panels = rocApp.workspace.getSidebarPanels();
  panels.forEach(handlePanelRegistered);
});

onUnmounted(() => {
  rocApp.events.off('workspace:sidebar-panel-registered', handlePanelRegistered);
});
</script>

<template>
  <aside class="sidebar">
    <Transition name="slide-right">
      <div v-show="layoutStore.rightSidebarVisible" class="sidebar-content">
        <FilesPanel v-if="activeTab === 'files'" />
        <SearchPanel v-else-if="activeTab === 'search'" />
        <BacklinksPanel v-else-if="activeTab === 'backlinks'" />
        <OutlinePanel v-else-if="activeTab === 'outline'" />
        <SettingsPanel v-else-if="activeTab === 'settings'" />
        <PluginPanel 
          v-else-if="getActivePluginPanel()" 
          :panel="getActivePluginPanel()!" 
        />
      </div>
    </Transition>
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
          <component v-if="item.iconType === 'component'" :is="item.icon" class="activity-icon" />
          <span v-else class="activity-icon">{{ item.icon }}</span>
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
  flex-shrink: 0;
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
  border: none;
  background: transparent;
  cursor: pointer;
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
  flex-shrink: 0;
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