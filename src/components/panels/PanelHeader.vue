<script setup lang="ts">
/**
 * 通用面板头部组件
 * - title: 面板标题
 * - count: 可选的数量徽标（与 actions 互斥显示）
 * - actions 插槽: 可选的操作按钮区域
 */
defineProps<{
  title: string
  count?: number
}>()
</script>

<template>
  <div class="panel-header">
    <span class="panel-title">{{ title }}</span>
    <span v-if="count !== undefined" class="panel-count">{{ count }}</span>
    <div v-else-if="$slots.actions" class="panel-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--roc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--roc-border);
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-count {
  font-size: 11px;
  color: var(--roc-text-muted);
  background: var(--roc-bg-tertiary);
  padding: 1px 6px;
  border-radius: 10px;
  font-weight: 400;
}

.panel-actions {
  display: flex;
  gap: 4px;
}
</style>
