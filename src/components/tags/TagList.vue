<script setup lang="ts">
import { useNotesStore } from '../../stores/notes'
import { useSearchStore } from '../../stores/search'

const notesStore = useNotesStore()
const searchStore = useSearchStore()

function getTagCount(tag: string): number {
  return notesStore.notes.filter(n => n.tags.includes(tag)).length
}

function searchTag(tag: string) {
  searchStore.openWithQuery('#' + tag)
}
</script>

<template>
  <div class="tag-list">
    <div
      v-for="tag in notesStore.allTags"
      :key="tag"
      class="tag-item"
      @click="searchTag(tag)"
    >
      <span class="tag-name">#{{ tag }}</span>
      <span class="tag-count">{{ getTagCount(tag) }}</span>
    </div>
    <div v-if="notesStore.allTags.length === 0" class="empty-state">
      没有标签
    </div>
  </div>
</template>