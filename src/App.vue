<script setup lang="ts">
import { onMounted, onErrorCaptured } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "./stores/vault";
import { useNotesStore } from "./stores/notes";

const router = useRouter();
const vaultStore = useVaultStore();
const notesStore = useNotesStore();

onMounted(async () => {
  await notesStore.setupEventListeners();
  await vaultStore.init();

  if (vaultStore.currentVault) {
    await notesStore.loadNotes();
    await router.push("/");
  } else {
    await router.push("/vault-picker");
  }
});

onErrorCaptured((err, _instance, info) => {
  console.error("[App] errorCaptured:", err, info);
  return false;
});
</script>

<template>
  <router-view />
</template>
