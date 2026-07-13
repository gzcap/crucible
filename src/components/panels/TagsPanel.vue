<script setup lang="ts">
import PanelHeader from "./PanelHeader.vue";
import TagList from "../tags/TagList.vue";
import { useNotesStore } from "../../stores/notes";

const notesStore = useNotesStore();
</script>

<template>
  <div class="content-panel">
    <PanelHeader title="标签" :count="notesStore.allTags.length" />
    <TagList />
  </div>
</template>

<style scoped>
.content-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
