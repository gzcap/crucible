<script setup lang="ts">
import { ref } from 'vue'
import { ElIcon, ElTooltip } from 'element-plus'
import { ArrowRight, Document, Location } from '@element-plus/icons-vue'
import { useNotesStore } from '../../stores/notes'
import { useSearchStore } from '../../stores/search'
import { getTagDetails } from '../../lib/tauri'
import type { TagDetail, TagOccurrence } from '../../lib/types'

const notesStore = useNotesStore()
const searchStore = useSearchStore()

const expandedTag = ref<string | null>(null)
const tagDetails = ref<Map<string, TagDetail>>(new Map())
const loadingTags = ref<Set<string>>(new Set())

function getTagCount(tag: string): number {
  return notesStore.notes.filter(n => n.tags.includes(tag)).length
}

async function toggleTag(tag: string) {
  if (expandedTag.value === tag) {
    expandedTag.value = null
    return
  }
  
  expandedTag.value = tag
  
  if (!tagDetails.value.has(tag)) {
    loadingTags.value.add(tag)
    try {
      const details = await getTagDetails(tag)
      tagDetails.value.set(tag, details)
    } catch {
      tagDetails.value.set(tag, { name: tag, count: getTagCount(tag), occurrences: [] })
    } finally {
      loadingTags.value.delete(tag)
    }
  }
}

function searchTag(tag: string) {
  searchStore.openWithQuery('#' + tag)
}

function openNote(occurrence: TagOccurrence) {
  notesStore.openNote(occurrence.path)
}
</script>

<template>
  <div class="tag-list">
    <div v-for="tag in notesStore.allTags" :key="tag" class="tag-group">
      <div class="tag-item" @click="toggleTag(tag)">
        <ElIcon :size="14" class="tag-arrow" :class="{ expanded: expandedTag === tag }">
          <ArrowRight />
        </ElIcon>
        <span class="tag-name">#{{ tag }}</span>
        <span class="tag-count">{{ getTagCount(tag) }}</span>
      </div>
      
      <div v-if="expandedTag === tag" class="tag-details">
        <div v-if="loadingTags.has(tag)" class="loading">加载中...</div>
        <div v-else-if="tagDetails.get(tag)?.occurrences.length === 0" class="empty-occurrences">
          未找到使用位置
        </div>
        <div
          v-for="(occurrence, idx) in tagDetails.get(tag)?.occurrences"
          :key="idx"
          class="occurrence-item"
          @click="openNote(occurrence)"
        >
          <ElIcon :size="12" class="occurrence-icon">
            <Document />
          </ElIcon>
          <div class="occurrence-info">
            <span class="occurrence-title">{{ occurrence.title }}</span>
            <ElTooltip :content="occurrence.path" placement="top">
              <span class="occurrence-path">{{ occurrence.path }}</span>
            </ElTooltip>
          </div>
          <ElIcon :size="12" class="line-icon">
            <Location />
          </ElIcon>
          <span class="occurrence-line">{{ occurrence.line }}</span>
        </div>
        <div v-if="tagDetails.get(tag)?.occurrences.length" class="tag-snippets">
          <div
            v-for="(occurrence, idx) in tagDetails.get(tag)?.occurrences"
            :key="'snippet-' + idx"
            class="snippet-item"
          >
            <span class="snippet-line-num">{{ occurrence.line }}</span>
            <span class="snippet-text">{{ occurrence.snippet }}</span>
          </div>
        </div>
      </div>
    </div>
    
    <div v-if="notesStore.allTags.length === 0" class="empty-state">
      没有标签
    </div>
  </div>
</template>

<style scoped>
.tag-list {
  padding: 6px;
}

.tag-group {
  margin-bottom: 2px;
}

.tag-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.tag-item:hover {
  background: var(--roc-bg-tertiary);
}

.tag-arrow {
  color: var(--roc-text-muted);
  transition: transform 0.15s;
  flex-shrink: 0;
}

.tag-arrow.expanded {
  transform: rotate(90deg);
}

.tag-name {
  font-size: 13px;
  color: var(--roc-accent);
  flex: 1;
}

.tag-count {
  font-size: 11px;
  color: var(--roc-text-muted);
  background: var(--roc-bg-tertiary);
  padding: 1px 6px;
  border-radius: 10px;
  flex-shrink: 0;
}

.tag-details {
  padding-left: 26px;
  margin-top: 4px;
  border-left: 2px solid var(--roc-border);
}

.loading {
  font-size: 12px;
  color: var(--roc-text-muted);
  padding: 8px;
}

.empty-occurrences {
  font-size: 12px;
  color: var(--roc-text-muted);
  padding: 8px;
}

.occurrence-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}

.occurrence-item:hover {
  background: var(--roc-bg-tertiary);
}

.occurrence-icon {
  color: var(--roc-text-muted);
  flex-shrink: 0;
}

.occurrence-info {
  flex: 1;
  min-width: 0;
}

.occurrence-title {
  font-size: 12px;
  color: var(--roc-text-primary);
  display: block;
}

.occurrence-path {
  font-size: 11px;
  color: var(--roc-text-muted);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.line-icon {
  color: var(--roc-text-muted);
  flex-shrink: 0;
}

.occurrence-line {
  font-size: 11px;
  color: var(--roc-accent);
  flex-shrink: 0;
}

.tag-snippets {
  margin-top: 8px;
  border-top: 1px solid var(--roc-border);
  padding-top: 8px;
}

.snippet-item {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  margin-bottom: 2px;
}

.snippet-line-num {
  font-size: 11px;
  color: var(--roc-text-muted);
  flex-shrink: 0;
  min-width: 24px;
  text-align: right;
}

.snippet-text {
  font-size: 12px;
  color: var(--roc-text-secondary);
  word-break: break-all;
}

.empty-state {
  text-align: center;
  color: var(--roc-text-muted);
  padding: 40px 20px;
  font-size: 13px;
}
</style>
