<script setup lang="ts">
import { ref, computed } from "vue";
import { Moon, Sunny, Monitor, Check } from "@element-plus/icons-vue";
import { useThemeStore, type ThemeMode } from "../../stores/theme";

const themeStore = useThemeStore();
const showMenu = ref(false);
const currentIcon = computed(() => {
  switch (themeStore.mode) {
    case "dark":
      return Moon;
    case "light":
      return Sunny;
    case "system":
      return Monitor;
    default:
      return Sunny;
  }
});

const options: { value: ThemeMode; label: string; icon: any }[] = [
  { value: "light", label: "亮色", icon: Sunny },
  { value: "dark", label: "暗色", icon: Moon },
  { value: "system", label: "跟随系统", icon: Monitor },
];

function selectTheme(option: any, e: MouseEvent) {
  const transition = document.startViewTransition(() => {
    themeStore.setMode(option.value);
    showMenu.value = false;
  });
  transition.ready.then(() => {
    const { clientX, clientY } = e;
    const radius = Math.hypot(
      Math.max(clientX, innerWidth - clientX),
      Math.max(clientY, innerHeight - clientY),
    );
    document.documentElement.animate(
      {
        clipPath: [
          `circle(0% at ${clientX}px ${clientY}px)`,
          `circle(${radius}px at ${clientX}px ${clientY}px)`,
        ],
      },
      {
        duration: 400,
        easing: "cubic-bezier(0.4, 0, 0.2, 1)",
        pseudoElement: "::view-transition-new(root)",
      },
    );
  });
}

function toggleMenu() {
  showMenu.value = !showMenu.value;
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".theme-toggle")) {
    showMenu.value = false;
  }
}

if (typeof window !== "undefined") {
  window.addEventListener("click", handleClickOutside);
}
</script>

<template>
  <div class="theme-toggle no-drag">
    <button
      ref="switchTheme"
      class="theme-btn"
      @click="toggleMenu"
      title="切换主题"
    >
      <component :is="currentIcon" :size="15" style="width: 16px; height: 16px;" />
    </button>
    <Transition name="menu">
      <div v-if="showMenu" class="theme-menu">
        <button
          v-for="option in options"
          :key="option.value"
          class="menu-item"
          :class="{ active: themeStore.mode === option.value }"
          @click="selectTheme(option, $event)"
        >
          <component :is="option.icon" :size="14" class="item-icon" />
          <span class="item-label">{{ option.label }}</span>
          <Check
            v-if="themeStore.mode === option.value"
            :size="14"
            class="item-check"
          />
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.theme-toggle {
  position: relative;
}

.theme-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 6px;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-text-primary);
}

.theme-btn:active {
  transform: scale(0.95);
}

.theme-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  min-width: 140px;
  background: var(--roc-bg-secondary);
  border: 1px solid var(--roc-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px;
  z-index: 100;
}

.menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: none;
  background: transparent;
  border-radius: 4px;
  color: var(--roc-text-primary);
  cursor: pointer;
  transition: background 0.15s;
}

.menu-item:hover {
  background: var(--roc-bg-tertiary);
}

.menu-item.active {
  background: rgba(0, 122, 204, 0.1);
  color: var(--roc-accent);
}

.item-icon {
  flex-shrink: 0;
}

.item-label {
  flex: 1;
  text-align: left;
  font-size: 12px;
}

.item-check {
  flex-shrink: 0;
}

.menu-enter-active,
.menu-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}
</style>
