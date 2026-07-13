import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GraphData } from '../lib/types'
import { getGraph } from '../lib/tauri'

export const useGraphStore = defineStore('graph', () => {
  const data = ref<GraphData>({ nodes: [], links: [] })
  const isLoading = ref(false)
  const tagFilter = ref<string | null>(null)

  async function loadGraph() {
    isLoading.value = true
    try {
      data.value = await getGraph()
    } finally {
      isLoading.value = false
    }
  }

  const filteredData = ref<GraphData>({ nodes: [], links: [] })

  function applyFilters() {
    if (!tagFilter.value) {
      filteredData.value = { ...data.value }
      return
    }

    filteredData.value = {
      nodes: data.value.nodes.filter(n => n.tag_count > 0),
      links: data.value.links
    }
  }

  function setTagFilter(tag: string | null) {
    tagFilter.value = tag
    applyFilters()
  }

  return {
    data,
    isLoading,
    tagFilter,
    filteredData,
    loadGraph,
    setTagFilter,
    applyFilters
  }
})