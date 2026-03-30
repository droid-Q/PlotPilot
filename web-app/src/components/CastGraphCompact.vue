<template>
  <div class="cgc-root">
    <div class="cgc-toolbar">
      <n-text depth="3" class="cgc-hint">
        与全页「人物关系网」同源（<code>cast_graph.json</code>）· 侧栏只读预览 · 点节点进入全页编辑
      </n-text>
      <n-space :size="8">
        <n-button size="small" quaternary :loading="loading" @click="reload">同步数据</n-button>
        <n-button size="small" secondary @click="goFull">完整编辑页</n-button>
      </n-space>
    </div>
    <div v-if="emptyHint" class="cgc-empty">
      <n-empty description="尚无人物节点，可在完整页添加" size="small" />
    </div>
    <div v-else class="cgc-canvas">
      <GraphChart :nodes="nodes" :links="links" height="100%" @node-click="handleNodeClick" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { bookApi } from '../api/book'
import GraphChart from './charts/GraphChart.vue'
import { convertGraph, type VisNode, type VisEdge } from '../utils/visToEcharts'
import type { EChartsNode, EChartsLink } from '../utils/visToEcharts'

const props = defineProps<{ slug: string }>()
const router = useRouter()

interface CastCharacter {
  id: string
  name: string
  aliases: string[]
  role: string
  traits: string
  note: string
  story_events?: unknown[]
}

interface CastRelationship {
  id: string
  source_id: string
  target_id: string
  label: string
  note: string
  directed: boolean
  story_events?: unknown[]
}

const loading = ref(false)
const graph = ref<{ characters: CastCharacter[]; relationships: CastRelationship[] }>({
  characters: [],
  relationships: [],
})
let requestId = 0

const emptyHint = computed(() => graph.value.characters.length === 0 && !loading.value)

const graphData = computed(() => {
  const visNodes: VisNode[] = graph.value.characters.map(c => {
    const ne = (c.story_events || []).length
    const base = [c.name, ...(c.aliases || []), c.traits, c.note].filter(Boolean).join('\n')
    return {
      id: c.id,
      label: c.name + (c.role ? `\n${c.role}` : '') + (ne ? `\n·${ne}事件` : ''),
      title: ne ? `${base}\n—\n人物线事件 ${ne} 条` : base,
      color: { background: '#c7d2fe', border: '#6366f1' },
      font: { size: 14 },
      shape: 'box',
      borderWidth: 2,
    }
  })

  const visEdges: VisEdge[] = graph.value.relationships.map(r => {
    const ne = (r.story_events || []).length
    const base = [r.label, r.note].filter(Boolean).join('\n')
    return {
      id: r.id,
      from: r.source_id,
      to: r.target_id,
      label: (r.label || '') + (ne ? ` ·${ne}` : ''),
      title: ne ? `${base || '关系'}\n—\n共同经历 ${ne} 条` : base || undefined,
      arrows: r.directed ? 'to' : undefined,
      font: { size: 11, align: 'middle' },
    }
  })

  return convertGraph(visNodes, visEdges)
})

const nodes = computed(() => graphData.value.nodes)
const links = computed(() => graphData.value.links)

const reload = async () => {
  const currentRequestId = ++requestId

  loading.value = true
  try {
    const data = await bookApi.getCast(props.slug)

    // Only update if this is still the latest request
    if (currentRequestId === requestId) {
      graph.value = {
        characters: data.characters || [],
        relationships: data.relationships || [],
      }
    }
  } catch (error) {
    console.error('Failed to load cast data:', error)
    if (currentRequestId === requestId) {
      window.$message?.error('加载人物关系失败，请稍后重试')
    }
  } finally {
    if (currentRequestId === requestId) {
      loading.value = false
    }
  }
}

const handleNodeClick = (node: EChartsNode) => {
  router.push({ path: `/book/${props.slug}/cast`, query: { focus: node.id } })
}

const goFull = () => {
  router.push(`/book/${props.slug}/cast`)
}

watch(
  () => props.slug,
  () => {
    void reload()
  }
)

onMounted(async () => {
  await nextTick()
  await reload()
})
</script>

<style scoped>
.cgc-root {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
  background: #fafafa;
  border-radius: 10px;
  border: 1px solid rgba(148, 163, 184, 0.25);
  overflow: hidden;
}

.cgc-toolbar {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.2);
  background: #fff;
}

.cgc-hint {
  font-size: 11px;
  line-height: 1.45;
  max-width: min(100%, 380px);
}

.cgc-hint code {
  font-size: 10px;
  padding: 0 4px;
  border-radius: 4px;
  background: rgba(79, 70, 229, 0.08);
  color: #4338ca;
}

.cgc-canvas {
  flex: 1;
  min-height: 220px;
  width: 100%;
}

.cgc-empty {
  position: absolute;
  left: 0;
  right: 0;
  top: 48px;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 1;
}
</style>
