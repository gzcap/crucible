<script setup lang="ts">
import { onErrorCaptured } from 'vue'
import { MilkdownProvider } from '@milkdown/vue'
import MilkdownEditor from './MilkdownEditor.vue'

/** 捕获 Milkdown 编辑器错误，阻止冒泡到 MainContent/AppLayout 导致整个布局崩溃 */
onErrorCaptured((err, instance, info) => {
  console.error('[EditorPane] errorCaptured:', err, info)
  return false
})
</script>

<template>
  <MilkdownProvider>
    <MilkdownEditor />
  </MilkdownProvider>
</template>

<style>
/* ============ Milkdown WYSIWYG 主题适配 ============
 * DOM 结构:
 *   .editor-pane > [data-milkdown-root].milkdown-pane > .ProseMirror.milkdown-body
 * nord 主题注入 prose 类，本样式覆盖颜色映射到 --roc-* 变量以跟随亮/暗主题
 */

.editor-pane {
  flex: 1;
  height: 100%;
  overflow: hidden;
  background: var(--roc-bg-primary);
}

.milkdown-pane {
  height: 100%;
  background: var(--roc-bg-primary);
  overflow-y: auto;
}

/* ProseMirror 编辑区域 */
.milkdown-pane .ProseMirror {
  outline: none;
  color: var(--roc-text-primary);
  caret-color: var(--roc-accent);
  font-size: 15px;
  line-height: 1.7;
  padding: 24px 32px;
  max-width: 820px;
  margin: 0 auto;
  min-height: 100%;
  background: var(--roc-bg-primary);
}

.milkdown-pane .ProseMirror p {
  margin: 0.6em 0;
}

.milkdown-pane .ProseMirror h1 {
  font-size: 1.9em;
  font-weight: 700;
  margin: 0.8em 0 0.4em;
  color: var(--roc-text-primary);
  border-bottom: 1px solid var(--roc-border);
  padding-bottom: 0.2em;
}

.milkdown-pane .ProseMirror h2 {
  font-size: 1.5em;
  font-weight: 600;
  margin: 0.7em 0 0.35em;
  color: var(--roc-text-primary);
}

.milkdown-pane .ProseMirror h3 {
  font-size: 1.25em;
  font-weight: 600;
  margin: 0.6em 0 0.3em;
  color: var(--roc-text-primary);
}

.milkdown-pane .ProseMirror h4,
.milkdown-pane .ProseMirror h5,
.milkdown-pane .ProseMirror h6 {
  font-size: 1.1em;
  font-weight: 600;
  margin: 0.5em 0 0.25em;
  color: var(--roc-text-primary);
}

.milkdown-pane .ProseMirror a {
  color: var(--roc-accent);
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-color 0.15s;
}

.milkdown-pane .ProseMirror a:hover {
  border-bottom-color: var(--roc-accent);
}

.milkdown-pane .ProseMirror strong {
  font-weight: 700;
  color: var(--roc-text-primary);
}

.milkdown-pane .ProseMirror em {
  font-style: italic;
}

.milkdown-pane .ProseMirror code {
  background: var(--roc-bg-tertiary);
  color: var(--roc-accent);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: "SF Mono", Monaco, Consolas, monospace;
  font-size: 0.88em;
}

.milkdown-pane .ProseMirror pre {
  background: var(--roc-bg-secondary);
  border: 1px solid var(--roc-border);
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
  margin: 1em 0;
}

.milkdown-pane .ProseMirror pre code {
  background: none;
  padding: 0;
  color: var(--roc-text-primary);
  font-size: 0.9em;
}

.milkdown-pane .ProseMirror blockquote {
  border-left: 3px solid var(--roc-accent);
  background: rgba(0, 122, 204, 0.05);
  padding: 8px 16px;
  margin: 1em 0;
  border-radius: 0 6px 6px 0;
  color: var(--roc-text-secondary);
}

.milkdown-pane .ProseMirror ul,
.milkdown-pane .ProseMirror ol {
  padding-left: 24px;
  margin: 0.6em 0;
}

.milkdown-pane .ProseMirror li {
  margin: 0.25em 0;
}

.milkdown-pane .ProseMirror img {
  max-width: 100%;
  border-radius: 8px;
  margin: 0.8em 0;
}

.milkdown-pane .ProseMirror hr {
  border: none;
  border-top: 1px solid var(--roc-border);
  margin: 1.5em 0;
}

/* 表格 */
.milkdown-pane .ProseMirror table {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  overflow: hidden;
}

.milkdown-pane .ProseMirror th,
.milkdown-pane .ProseMirror td {
  border: 1px solid var(--roc-border);
  padding: 8px 12px;
  text-align: left;
}

.milkdown-pane .ProseMirror th {
  background: var(--roc-bg-secondary);
  font-weight: 600;
  color: var(--roc-text-primary);
}

/* 选中区域 */
.milkdown-pane .ProseMirror ::selection {
  background: rgba(0, 122, 204, 0.2);
}

/* 滚动条 */
.milkdown-pane::-webkit-scrollbar {
  width: 8px;
}

.milkdown-pane::-webkit-scrollbar-thumb {
  background: var(--roc-bg-tertiary);
  border-radius: 4px;
}

.milkdown-pane::-webkit-scrollbar-thumb:hover {
  background: var(--roc-text-muted);
}

/* 焦点态 */
.milkdown-pane .ProseMirror-focused {
  outline: none;
}
</style>
