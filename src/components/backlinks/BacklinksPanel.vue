<script setup lang="ts">
import { ref, computed } from 'vue'
import { useNotesStore } from '../../stores/notes'

const notesStore = useNotesStore()

const activeTab = ref<'links' | 'potential'>('links')

const backlinks = computed(() => notesStore.backlinks)
const unresolvedLinks = computed(() => {
  return notesStore.currentPath ? notesStore.notes.filter(n => 
    !notesStore.notes.find(n2 => n2.path === n.path)
  ) : []
})
</script>

<template>
  <aside class="backlinks-panel">
    <div class="panel-header">
      <h3>当前笔记中的链接</h3>
      <span class="count">{{ backlinks.length }}</span>
    </div>

    <div class="panel-tabs">
      <button
        class="panel-tab"
        :class="{ active: activeTab === 'links' }"
        @click="activeTab = 'links'"
      >
        当前笔记中的链接
      </button>
      <button
        class="panel-tab"
        :class="{ active: activeTab === 'potential' }"
        @click="activeTab = 'potential'"
      >
        当前笔记中潜在的链接
      </button>
    </div>

    <div class="backlinks-list" v-if="activeTab === 'links'">
      <div
        v-for="backlink in backlinks"
        :key="backlink.source"
        class="backlink-item"
        @click="notesStore.openNote(backlink.source)"
      >
        <div class="backlink-title">{{ backlink.source_title }}</div>
        <div class="backlink-snippet">{{ backlink.snippet }}</div>
      </div>

      <div v-if="backlinks.length === 0" class="empty-backlinks">
        <p>未找到反向链接</p>
      </div>
    </div>

    <div class="backlinks-list" v-if="activeTab === 'potential'">
      <div v-if="unresolvedLinks.length === 0" class="empty-backlinks">
        <p>没有潜在链接</p>
      </div>
    </div>
  </aside>
</template>
