<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, reactive, computed, nextTick, triggerRef } from 'vue'
import { forceSimulation, forceManyBody, forceLink, forceCenter, forceCollide, forceX, forceY, type Simulation } from 'd3-force'
import { select } from 'd3-selection'
import { zoom, zoomIdentity, type ZoomBehavior } from 'd3-zoom'
import { Aim, Refresh, FullScreen, ScaleToOriginal } from '@element-plus/icons-vue'
import { useGraphStore } from '../../stores/graph'
import { useNotesStore } from '../../stores/notes'
import { useThemeStore } from '../../stores/theme'
import type { GraphData } from '../../lib/types'

const graphStore = useGraphStore()
const notesStore = useNotesStore()
const themeStore = useThemeStore()

const svgRef = ref<SVGSVGElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)

let simulation: Simulation<any, any> | null = null
let zoomBehavior: ZoomBehavior<SVGSVGElement, unknown> | null = null
const hoveredNode = ref<any>(null)

/** 拖拽状态 */
const draggingNode = ref<any>(null)
let dragStartPos: { x: number, y: number } | null = null
let hasDragged = false

/** 从 CSS 变量获取主题色 */
const colors = reactive({
  node: '#007acc',
  current: '#e8590c',
  link: '#cccccc',
  text: '#333333',
  bg: '#ffffff',
})

/** SVG 中使用的节点/链接数据（带 x/y 坐标） */
const simNodes = ref<any[]>([])
const simLinks = ref<any[]>([])

/** 节点展示上限 */
const MAX_NODES = 80
/** 过滤信息（用于工具栏提示） */
const filteredInfo = ref<{ shown: number, total: number } | null>(null)

function updateColors() {
  const style = getComputedStyle(document.documentElement)
  colors.node = style.getPropertyValue('--roc-accent').trim() || '#007acc'
  colors.link = style.getPropertyValue('--roc-border').trim() || '#cccccc'
  colors.text = style.getPropertyValue('--roc-text-primary').trim() || '#333333'
  colors.bg = style.getPropertyValue('--roc-bg-primary').trim() || '#ffffff'
  colors.current = '#e8590c'
}

/** hex 转 rgba */
function withAlpha(hex: string, alpha: number): string {
  if (hex.startsWith('#') && hex.length >= 7) {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    return `rgba(${r}, ${g}, ${b}, ${alpha})`
  }
  return hex
}

/** 高亮节点集合（hover 时关联节点） */
const highlightedSet = computed<Set<string> | null>(() => {
  if (!hoveredNode.value) return null
  const set = new Set<string>([hoveredNode.value.id])
  for (const link of simLinks.value) {
    const src = typeof link.source === 'object' ? link.source.id : link.source
    const tgt = typeof link.target === 'object' ? link.target.id : link.target
    if (src === hoveredNode.value.id) set.add(tgt)
    if (tgt === hoveredNode.value.id) set.add(src)
  }
  return set
})

/** 孤立节点数 */
const orphanCount = computed(() => {
  const linked = new Set<string>()
  for (const link of graphStore.data.links as any[]) {
    linked.add(link.source as string)
    linked.add(link.target as string)
  }
  return graphStore.data.nodes.filter(n => !linked.has(n.id)).length
})

/** 节点半径 */
function nodeRadius(node: any): number {
  return Math.max(4, Math.min(14, Math.sqrt(node.link_count || 0) * 3 + 4))
}

/** 判断节点是否高亮 */
function isNodeHighlighted(node: any): boolean {
  return !highlightedSet.value || highlightedSet.value.has(node.id)
}

/** 判断链接是否高亮 */
function isLinkHighlighted(link: any): boolean {
  if (!hoveredNode.value) return true
  const src = typeof link.source === 'object' ? link.source.id : link.source
  const tgt = typeof link.target === 'object' ? link.target.id : link.target
  return src === hoveredNode.value.id || tgt === hoveredNode.value.id
}

function initGraph() {
  if (!svgRef.value || !containerRef.value) return

  const rect = containerRef.value.getBoundingClientRect()
  const w = Math.max(rect.width, 300)
  const h = Math.max(rect.height, 300)

  // 节点过多时按 link_count 降序取前 MAX_NODES 个
  const allNodes = graphStore.data.nodes
  let activeNodes: any[]
  if (allNodes.length > MAX_NODES) {
    activeNodes = [...allNodes]
      .sort((a, b) => (b.link_count || 0) - (a.link_count || 0))
      .slice(0, MAX_NODES)
    filteredInfo.value = { shown: MAX_NODES, total: allNodes.length }
  } else {
    activeNodes = allNodes
    filteredInfo.value = null
  }
  const visibleIds = new Set(activeNodes.map(n => n.id))

  // 过滤链接：两端节点都需可见
  const activeLinks = graphStore.data.links.filter(l => {
    const src = typeof l.source === 'object' ? (l.source as any).id : l.source
    const tgt = typeof l.target === 'object' ? (l.target as any).id : l.target
    return visibleIds.has(src) && visibleIds.has(tgt)
  })

  // 准备数据副本（force 会修改对象的 x/y），初始位置在可视区域内随机分布
  const nodes = activeNodes.map(n => ({ ...n, x: Math.random() * w, y: Math.random() * h }))
  const links = activeLinks.map(l => ({ ...l }))
  simNodes.value = nodes
  simLinks.value = links

  // 创建力导向模拟（velocityDecay 较低 → 运动更流畅灵动）
  simulation = forceSimulation(nodes as any)
    .force('charge', forceManyBody().strength(-200))
    .force('link', forceLink(links).id((d: any) => d.id).distance(70).strength(0.3))
    .force('center', forceCenter(w / 2, h / 2).strength(0.06))
    .force('x', forceX(w / 2).strength(0.04))
    .force('y', forceY(h / 2).strength(0.04))
    .force('collide', forceCollide((d: any) => nodeRadius(d) + 5))
    .alpha(1)
    .alphaDecay(0.018)
    .velocityDecay(0.3)

  if (!simulation) return

  // tick 时触发响应式更新，让 Vue 重新渲染 SVG 元素
  simulation.on('tick', () => {
    triggerRef(simNodes)
    triggerRef(simLinks)
  })

  // 缩放支持
  zoomBehavior = zoom<SVGSVGElement, unknown>()
    .scaleExtent([0.02, 5])
    .on('zoom', (event) => {
      const g = svgRef.value?.querySelector('.graph-content') as SVGGElement
      if (g) {
        select(g).attr('transform', event.transform.toString())
      }
      currentScale.value = event.transform.k
    })

  select(svgRef.value).call(zoomBehavior as any)
}

/** 适配视图 */
function fitView() {
  if (!svgRef.value || !zoomBehavior || simNodes.value.length === 0) return
  const rect = containerRef.value?.getBoundingClientRect()
  if (!rect) return

  const padding = 60
  const xs = simNodes.value.map(n => n.x)
  const ys = simNodes.value.map(n => n.y)
  const minX = Math.min(...xs), maxX = Math.max(...xs)
  const minY = Math.min(...ys), maxY = Math.max(...ys)
  const graphW = maxX - minX || 1
  const graphH = maxY - minY || 1
  const scale = Math.min((rect.width - padding * 2) / graphW, (rect.height - padding * 2) / graphH, 2)
  const tx = rect.width / 2 - (minX + graphW / 2) * scale
  const ty = rect.height / 2 - (minY + graphH / 2) * scale

  select(svgRef.value).call(zoomBehavior.transform, zoomIdentity.translate(tx, ty).scale(scale))
}

/** 重启布局模拟 */
function reheat() {
  if (simulation) {
    simulation.alpha(1).restart()
  }
}

/** 节点点击 */
function handleNodeClick(node: any) {
  notesStore.openNote(node.id)
}

/** 将屏幕坐标转换为 SVG 内容坐标（考虑 zoom transform） */
function screenToSvgCoords(clientX: number, clientY: number): { x: number, y: number } {
  const svg = svgRef.value
  if (!svg) return { x: 0, y: 0 }
  const pt = svg.createSVGPoint()
  pt.x = clientX
  pt.y = clientY
  const ctm = (svg.querySelector('.graph-content') as SVGGraphicsElement | null)?.getScreenCTM()
  if (!ctm) return { x: 0, y: 0 }
  const transformed = pt.matrixTransform(ctm.inverse())
  return { x: transformed.x, y: transformed.y }
}

/** 节点拖拽开始 */
function onNodeDragStart(event: MouseEvent, node: any) {
  event.preventDefault()
  event.stopPropagation()
  dragStartPos = { x: event.clientX, y: event.clientY }
  hasDragged = false
  draggingNode.value = node
  node.fx = node.x
  node.fy = node.y
  simulation?.alphaTarget(0.3).restart()

  document.addEventListener('mousemove', onNodeDragMove)
  document.addEventListener('mouseup', onNodeDragEnd)
}

/** 节点拖拽移动 */
function onNodeDragMove(event: MouseEvent) {
  if (!draggingNode.value || !dragStartPos) return
  const dx = event.clientX - dragStartPos.x
  const dy = event.clientY - dragStartPos.y
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    hasDragged = true
  }
  if (hasDragged) {
    const { x, y } = screenToSvgCoords(event.clientX, event.clientY)
    draggingNode.value.fx = x
    draggingNode.value.fy = y
    triggerRef(simNodes)
  }
}

/** 节点拖拽结束 */
function onNodeDragEnd() {
  if (!draggingNode.value) return
  simulation?.alphaTarget(0)
  if (!hasDragged) {
    // 没有拖拽 → 视为点击
    handleNodeClick(draggingNode.value)
  }
  // 释放后取消固定，让节点回归力布局
  draggingNode.value.fx = null
  draggingNode.value.fy = null
  draggingNode.value = null
  dragStartPos = null
  hasDragged = false
  document.removeEventListener('mousemove', onNodeDragMove)
  document.removeEventListener('mouseup', onNodeDragEnd)
}

watch(() => graphStore.data, (newData: GraphData) => {
  if (simulation) {
    simulation.stop()
    simulation = null
  }
  if (svgRef.value) {
    initGraph()
  }
}, { deep: true })

/** 主题切换时更新颜色 */
watch(() => themeStore.actualTheme, () => {
  updateColors()
})

onMounted(async () => {
  await graphStore.loadGraph()
  updateColors()
  await nextTick()
  waitForContainer()
})

/** 轮询等待容器有非零尺寸后初始化图谱 */
function waitForContainer(retries = 20) {
  if (!containerRef.value) {
    if (retries > 0) {
      setTimeout(() => waitForContainer(retries - 1), 50)
    }
    return
  }
  const rect = containerRef.value.getBoundingClientRect()
  if (rect.width > 0 && rect.height > 0) {
    initGraph()
    setTimeout(fitView, 1500)
  } else if (retries > 0) {
    setTimeout(() => waitForContainer(retries - 1), 50)
  }
}

onUnmounted(() => {
  if (simulation) {
    simulation.stop()
    simulation = null
  }
})

/** 缩放级别（控制标签显示） */
const currentScale = ref(1)
</script>

<template>
  <div class="graph-view">
    <!-- 工具栏 -->
    <div class="graph-toolbar">
      <div class="toolbar-left">
        <Aim :size="16" class="toolbar-icon" />
        <span class="toolbar-title">关系图谱</span>
        <div class="toolbar-stats">
          <template v-if="filteredInfo">
            <span class="stat stat-filtered">显示 {{ filteredInfo.shown }}/{{ filteredInfo.total }} 节点</span>
          </template>
          <template v-else>
            <span class="stat">{{ graphStore.data.nodes.length }} 节点</span>
          </template>
          <span class="stat-sep">·</span>
          <span class="stat">{{ simLinks.length }} 链接</span>
        </div>
      </div>
      <div class="toolbar-right">
        <el-tooltip content="适配视图" placement="bottom">
          <button class="graph-btn" @click="fitView">
            <FullScreen :size="15" />
          </button>
        </el-tooltip>
        <el-tooltip content="重新布局" placement="bottom">
          <button class="graph-btn" @click="reheat">
            <ScaleToOriginal :size="15" />
          </button>
        </el-tooltip>
        <el-tooltip content="刷新数据" placement="bottom">
          <button class="graph-btn" @click="graphStore.loadGraph()">
            <Refresh :size="15" />
          </button>
        </el-tooltip>
      </div>
    </div>

    <!-- 图谱容器 -->
    <div class="graph-canvas-wrapper" ref="containerRef">
      <svg
        ref="svgRef"
        class="graph-svg"
      >
        <g class="graph-content">
          <!-- 链接 -->
          <g class="links-group">
            <line
              v-for="(link, i) in simLinks"
              :key="i"
              class="graph-link"
              :x1="(link.source as any)?.x ?? 0"
              :y1="(link.source as any)?.y ?? 0"
              :x2="(link.target as any)?.x ?? 0"
              :y2="(link.target as any)?.y ?? 0"
              :stroke="isLinkHighlighted(link) ? withAlpha(colors.link, 0.6) : withAlpha(colors.link, 0.1)"
              :stroke-width="isLinkHighlighted(link) ? 1.5 : 0.5"
            />
          </g>

          <!-- 节点 -->
          <g class="nodes-group">
            <g
              v-for="node in simNodes"
              :key="node.id"
              class="node-group"
              :class="{ 'node-dragging': draggingNode?.id === node.id }"
              :transform="`translate(${node.x},${node.y})`"
              @mouseenter="hoveredNode = node"
              @mouseleave="hoveredNode = null"
              @mousedown="onNodeDragStart($event, node)"
              :style="{ cursor: draggingNode?.id === node.id ? 'grabbing' : 'grab' }"
            >
              <!-- 当前笔记光晕 -->
              <circle
                v-if="node.id === notesStore.currentPath"
                :r="nodeRadius(node) + 6"
                :fill="withAlpha(colors.current, 0.15)"
              />
              <!-- 节点圆 -->
              <circle
                :r="nodeRadius(node)"
                :fill="node.id === notesStore.currentPath || hoveredNode?.id === node.id
                  ? colors.current
                  : isNodeHighlighted(node) ? colors.node : withAlpha(colors.node, 0.2)"
                :stroke="node.id === notesStore.currentPath ? colors.current : withAlpha(colors.link, 0.6)"
                :stroke-width="node.id === notesStore.currentPath ? 2.5 : 0.8"
              />
              <!-- 标签 -->
              <text
                v-if="currentScale >= 0.3 || hoveredNode?.id === node.id || node.id === notesStore.currentPath"
                :y="nodeRadius(node) + 10"
                :fill="isNodeHighlighted(node) ? colors.text : withAlpha(colors.text, 0.3)"
                font-size="9"
                font-family='-apple-system, "PingFang SC", sans-serif'
                text-anchor="middle"
                class="node-label"
              >{{ node.name.length > 22 ? node.name.slice(0, 20) + '…' : node.name }}</text>
            </g>
          </g>
        </g>
      </svg>

      <!-- 加载状态 -->
      <div v-if="graphStore.isLoading" class="graph-overlay">
        <div class="loading-spinner"></div>
        <p>加载图谱中...</p>
      </div>

      <!-- 空状态 -->
      <div v-else-if="graphStore.data.nodes.length === 0" class="graph-overlay">
        <Aim :size="48" class="empty-icon" />
        <p class="empty-title">暂无图谱数据</p>
        <p class="empty-desc">打开仓库后，笔记间的链接将在此可视化</p>
      </div>
    </div>

    <!-- 图例 -->
    <div class="graph-legend" v-if="graphStore.data.nodes.length > 0">
      <div class="legend-item">
        <span class="legend-dot legend-dot--normal"></span>
        <span>笔记</span>
      </div>
      <div class="legend-item">
        <span class="legend-dot legend-dot--current"></span>
        <span>当前笔记</span>
      </div>
      <div class="legend-item">
        <span class="legend-line"></span>
        <span>链接</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.graph-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--roc-bg-primary);
}

/* 工具栏 */
.graph-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--roc-border);
  background: var(--roc-bg-secondary);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-icon {
  color: var(--roc-accent);
}

.toolbar-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--roc-text-primary);
}

.toolbar-stats {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 8px;
}

.stat {
  font-size: 11px;
  color: var(--roc-text-muted);
}

.stat-filtered {
  color: var(--roc-accent);
  font-weight: 500;
}

.stat-sep {
  color: var(--roc-text-muted);
  font-size: 11px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.graph-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--roc-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.graph-btn:hover {
  background: var(--roc-bg-tertiary);
  color: var(--roc-accent);
}

/* 图谱容器 */
.graph-canvas-wrapper {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.graph-svg {
  width: 100%;
  height: 100%;
  display: block;
  background: var(--roc-bg-primary);
}

.node-label {
  pointer-events: none;
  user-select: none;
}

.node-group {
  transition: opacity 0.15s;
}

.node-group circle {
  transition: r 0.18s ease, fill 0.15s, stroke-width 0.15s;
}

.node-dragging {
  opacity: 1 !important;
}

.node-group:hover {
  opacity: 1;
}

/* 遮罩层（加载/空状态） */
.graph-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--roc-bg-primary);
  z-index: 10;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--roc-border);
  border-top-color: var(--roc-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.graph-overlay p {
  margin: 0;
  color: var(--roc-text-secondary);
}

.graph-overlay .loading-spinner + p {
  font-size: 13px;
}

.empty-icon {
  color: var(--roc-text-muted);
  opacity: 0.3;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 15px;
  font-weight: 500;
  color: var(--roc-text-secondary);
  margin-bottom: 6px !important;
}

.empty-desc {
  font-size: 12px;
  color: var(--roc-text-muted);
}

/* 图例 */
.graph-legend {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 6px 16px;
  border-top: 1px solid var(--roc-border);
  background: var(--roc-bg-secondary);
  flex-shrink: 0;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--roc-text-muted);
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.legend-dot--normal {
  background: var(--roc-accent);
}

.legend-dot--current {
  background: #e8590c;
  box-shadow: 0 0 0 3px rgba(232, 89, 12, 0.2);
}

.legend-line {
  width: 16px;
  height: 1.5px;
  background: var(--roc-text-muted);
  opacity: 0.5;
}
</style>
