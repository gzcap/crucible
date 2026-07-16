<script setup lang="ts">
import { onMounted, onErrorCaptured, watch } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "./stores/vault";
import { useNotesStore } from "./stores/notes";
import { pluginLoader } from "./plugins";

const router = useRouter();
const vaultStore = useVaultStore();
const notesStore = useNotesStore();

async function loadThirdPartyPlugins() {
  const vault = vaultStore.currentVault;
  if (!vault) return;

  const pluginDir = `${vault.path}/.roc/plugins`;
  try {
    await pluginLoader.reloadPluginsFromDirectory(pluginDir);
    console.log(`[App] Reloaded third-party plugins from ${pluginDir}`);
  } catch (error) {
    console.warn(`[App] Failed to load plugins from ${pluginDir}:`, error);
  }
}

async function handleVaultChange() {
  const vault = vaultStore.currentVault;
  if (vault) {
    await notesStore.loadNotes();
    await loadThirdPartyPlugins();
  }
}

onMounted(async () => {
  await notesStore.setupEventListeners();
  await vaultStore.init();

  if (vaultStore.currentVault) {
    await loadThirdPartyPlugins();
    await router.push("/");
  } else {
    await router.push("/vault-picker");
  }
});

watch(
  () => vaultStore.currentVault,
  (newVault, oldVault) => {
    if (newVault && newVault.id !== oldVault?.id) {
      handleVaultChange();
    }
  }
);

onErrorCaptured((err, _instance, info) => {
  console.error("[App] errorCaptured:", err, info);
  return false;
});
</script>

<template>
  <router-view />
</template>
