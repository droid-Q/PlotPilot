<template>
  <div class="ce-panel">
    <n-empty v-if="!currentChapterNumber" description="请先从左侧选择一个章节" style="margin-top: 40px" />

    <n-scrollbar v-else class="ce-scroll">
      <n-space vertical :size="12" style="padding: 8px 4px 16px">
        <n-alert v-if="readOnly" type="warning" :show-icon="true" size="small">
          托管运行中：仅可查看
        </n-alert>

        <!-- 本章规划 -->
        <n-card v-if="chapterPlan" size="small" :bordered="true" class="ce-card-plan">
          <template #header>
            <span class="card-title">📋 本章规划</span>
          </template>
          <n-descriptions :column="1" label-placement="left" size="small" label-style="white-space: nowrap">
            <n-descriptions-item label="标题">{{ chapterPlan.title || '—' }}</n-descriptions-item>
            <n-descriptions-item v-if="chapterPlan.outline" label="大纲">
              <n-text style="font-size: 12px; white-space: pre-wrap">{{ chapterPlan.outline }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item v-if="chapterPlan.pov_character_id" label="视角">
              {{ getCharacterName(chapterPlan.pov_character_id) }}
            </n-descriptions-item>
            <n-descriptions-item v-if="chapterPlan.timeline_start || chapterPlan.timeline_end" label="时间线">
              {{ chapterPlan.timeline_start || '—' }} → {{ chapterPlan.timeline_end || '—' }}
            </n-descriptions-item>
            <n-descriptions-item v-if="planMoodLine" label="基调">
              {{ planMoodLine }}
            </n-descriptions-item>
          </n-descriptions>
        </n-card>

        <!-- 节拍规划 -->
        <n-card v-if="showBeatsCard" size="small" :bordered="true">
          <template #header>
            <span class="card-title">🎬 节拍规划</span>
          </template>
          <n-tabs type="segment" size="small" animated>
            <n-tab-pane name="macro" tab="宏观">
              <ol v-if="beatLines.length" class="ce-beat-list">
                <li v-for="(line, bi) in beatLines" :key="bi">{{ line }}</li>
              </ol>
              <n-empty v-else description="暂无宏观节拍" size="small" />
            </n-tab-pane>
            
            <n-tab-pane name="micro" tab="微观">
              <n-space v-if="microBeats.length" vertical :size="8" style="margin-top: 12px">
                <div v-for="(beat, i) in microBeats" :key="i" class="micro-beat-item">
                  <div class="micro-beat-header">
                    <n-tag :type="getBeatTypeColor(beat.focus)" size="small" round>
                      {{ beat.focus }}
                    </n-tag>
                    <n-text strong style="margin-left: 8px">Beat {{ i + 1 }}</n-text>
                    <n-text depth="3" style="margin-left: 8px; font-size: 12px">
                      ({{ beat.target_words }}字)
                    </n-text>
                  </div>
                  <div class="micro-beat-desc">{{ beat.description }}</div>
                </div>
              </n-space>
              <n-empty v-else description="章节生成时自动创建微观节拍" size="small" />
            </n-tab-pane>
          </n-tabs>
        </n-card>

        <!-- 本章总结 -->
        <n-card v-if="hasSummaryBlock" size="small" :bordered="true">
          <template #header>
            <span class="card-title">📝 本章总结</span>
          </template>
          <n-descriptions
            v-if="knowledgeChapter && (knowledgeChapter.summary || knowledgeChapter.key_events || knowledgeChapter.consistency_note)"
            :column="1"
            label-placement="left"
            size="small"
          >
            <n-descriptions-item v-if="knowledgeChapter.summary" label="摘要">
              <n-text style="font-size: 12px; white-space: pre-wrap">{{ knowledgeChapter.summary }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item v-if="knowledgeChapter.key_events" label="关键事件">
              <n-text style="font-size: 12px; white-space: pre-wrap">{{ knowledgeChapter.key_events }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item v-if="knowledgeChapter.consistency_note" label="一致性">
              <n-text style="font-size: 12px; white-space: pre-wrap">{{ knowledgeChapter.consistency_note }}</n-text>
            </n-descriptions-item>
          </n-descriptions>
          <n-text v-else-if="chapterPlan?.description" style="font-size: 12px; white-space: pre-wrap">
            {{ chapterPlan.description }}
          </n-text>
        </n-card>

        <n-alert v-else-if="storyNodeNotFound" type="warning" :show-icon="true">
          未在结构树中找到第 {{ currentChapterNumber }} 章的规划节点
        </n-alert>

        <!-- 人物/地点/道具 -->
        <n-card size="small" :bordered="true" class="ce-card-elements">
          <template #header>
            <div class="ce-card-header-row">
              <span class="card-title">👥 人物 / 地点 / 道具</span>
              <n-space :size="6">
                <n-select
                  v-model:value="filterType"
                  :options="elementTypeOptions"
                  size="tiny"
                  style="width: 80px"
                  clearable
                  placeholder="类型"
                  @update:value="loadElements"
                />
                <n-button size="tiny" secondary :loading="loading" @click="loadElements">刷新</n-button>
              </n-space>
            </div>
          </template>

          <n-spin :show="loading">
            <n-space vertical :size="8">
              <n-space v-if="groupedCharacters.length" vertical :size="6">
                <n-text strong class="ce-group-label">👤 人物</n-text>
                <n-space vertical :size="4">
                  <div v-for="elem in groupedCharacters" :key="elem.id" class="ce-item-readonly">
                    <n-text class="ce-element-name">{{ getElementDisplayName(elem.element_id, 'character') }}</n-text>
                    <n-tag size="tiny" round type="default">{{ relationLabel(elem.relation_type) }}</n-tag>
                    <n-tag :type="getImportanceType(elem.importance)" size="tiny" round>
                      {{ importanceLabel(elem.importance) }}
                    </n-tag>
                    <n-text v-if="elem.notes" depth="3" style="font-size: 12px; margin-left: 8px">
                      {{ elem.notes }}
                    </n-text>
                  </div>
                </n-space>
              </n-space>

              <n-space v-if="groupedLocations.length" vertical :size="6">
                <n-text strong class="ce-group-label">📍 地点</n-text>
                <n-space vertical :size="4">
                  <div v-for="elem in groupedLocations" :key="elem.id" class="ce-item-readonly">
                    <n-text class="ce-element-name">{{ getElementDisplayName(elem.element_id, 'location') }}</n-text>
                    <n-tag size="tiny" round type="default">{{ relationLabel(elem.relation_type) }}</n-tag>
                    <n-tag :type="getImportanceType(elem.importance)" size="tiny" round>
                      {{ importanceLabel(elem.importance) }}
                    </n-tag>
                    <n-text v-if="elem.notes" depth="3" style="font-size: 12px; margin-left: 8px">
                      {{ elem.notes }}
                    </n-text>
                  </div>
                </n-space>
              </n-space>

              <n-space v-if="groupedOther.length" vertical :size="6">
                <n-text strong class="ce-group-label">📦 其他</n-text>
                <n-space vertical :size="4">
                  <div v-for="elem in groupedOther" :key="elem.id" class="ce-item-readonly">
                    <n-tag :type="elemTypeColor(elem.element_type)" size="tiny" round>
                      {{ elemTypeLabel(elem.element_type) }}
                    </n-tag>
                    <n-text class="ce-element-name">{{ getElementDisplayName(elem.element_id, elem.element_type) }}</n-text>
                    <n-tag size="tiny" round type="default">{{ relationLabel(elem.relation_type) }}</n-tag>
                    <n-tag :type="getImportanceType(elem.importance)" size="tiny" round>
                      {{ importanceLabel(elem.importance) }}
                    </n-tag>
                    <n-text v-if="elem.notes" depth="3" style="font-size: 12px; margin-left: 8px">
                      {{ elem.notes }}
                    </n-text>
                  </div>
                </n-space>
              </n-space>

              <n-empty v-if="!loading && elements.length === 0" description="暂无关联元素" size="small" />
            </n-space>
          </n-spin>
        </n-card>

        <!-- 伏笔回收建议 -->
        <n-card size="small" :bordered="true">
          <template #header>
            <span class="card-title">🔗 伏笔回收建议</span>
          </template>
          <ForeshadowChapterSuggestionsPanel
            :slug="slug"
            :current-chapter-number="currentChapterNumber"
            :prefill-outline="chapterPlan?.outline || ''"
            embedded
            compact
            auto-run
          />
        </n-card>

        <!-- 全托管管线摘要 -->
        <n-card
          v-if="autopilotChapterReview && currentChapterNumber === autopilotChapterReview.chapter_number"
          size="small"
          :bordered="true"
        >
          <template #header>
            <span class="card-title">🤖 自动审阅</span>
          </template>
          <n-space vertical :size="8">
            <div class="review-row">
              <n-text depth="3">张力</n-text>
              <div class="tension-bar">
                <div class="tension-fill" :style="{ width: `${autopilotChapterReview.tension * 10}%` }"></div>
                <n-text class="tension-value">{{ autopilotChapterReview.tension }}/10</n-text>
              </div>
            </div>
            <div class="review-row">
              <n-text depth="3">叙事同步</n-text>
              <n-tag
                :type="autopilotChapterReview.narrative_sync_ok ? 'success' : 'warning'"
                size="small"
                round
              >
                {{ autopilotChapterReview.narrative_sync_ok ? '已落库' : '异常' }}
              </n-tag>
            </div>
            <div class="review-row">
              <n-text depth="3">文风相似度</n-text>
              <n-text>
                {{
                  autopilotChapterReview.similarity_score != null
                    ? Number(autopilotChapterReview.similarity_score).toFixed(3)
                    : '—'
                }}
              </n-text>
            </div>
            <div class="review-row">
              <n-text depth="3">漂移告警</n-text>
              <n-tag :type="autopilotChapterReview.drift_alert ? 'error' : 'success'" size="small" round>
                {{ autopilotChapterReview.drift_alert ? '是' : '否' }}
              </n-tag>
            </div>
            <div v-if="autopilotChapterReview.at" class="review-row">
              <n-text depth="3">审阅时间</n-text>
              <n-text depth="3" style="font-size: 12px">{{ formatTime(autopilotChapterReview.at) }}</n-text>
            </div>
          </n-space>
        </n-card>

        <!-- AI 审阅与质检 -->
        <n-card
          v-if="lastWorkflowResult && qcChapterNumber != null"
          size="small"
          :bordered="true"
        >
          <template #header>
            <span class="card-title">✨ AI 生成质检</span>
          </template>
          <n-space vertical :size="10">
            <n-alert
              v-if="currentChapterNumber !== qcChapterNumber"
              type="info"
              size="small"
            >
              为第 {{ qcChapterNumber }} 章质检结果
            </n-alert>

            <ConsistencyReportPanel
              :report="lastWorkflowResult.consistency_report"
              :token-count="lastWorkflowResult.token_count"
              @location-click="onLocationClick"
            />

            <n-collapse
              v-if="lastWorkflowResult.style_warnings && lastWorkflowResult.style_warnings.length > 0"
              class="qc-collapse"
            >
              <n-collapse-item :title="`俗套句式 ${lastWorkflowResult.style_warnings.length} 处`" name="cliche">
                <n-space vertical :size="6">
                  <n-alert
                    v-for="(w, i) in lastWorkflowResult.style_warnings"
                    :key="i"
                    :type="w.severity === 'warning' ? 'warning' : 'info'"
                    :title="w.pattern"
                    size="small"
                  >
                    「{{ w.text }}」
                  </n-alert>
                </n-space>
              </n-collapse-item>
            </n-collapse>

            <n-collapse v-if="ghostAnnotationLines.length > 0" class="qc-collapse">
              <n-collapse-item :title="`冲突批注 ${ghostAnnotationLines.length} 条`" name="ghost">
                <n-space vertical :size="6">
                  <n-alert
                    v-for="(line, gi) in ghostAnnotationLines"
                    :key="gi"
                    type="warning"
                    size="small"
                  >
                    {{ line }}
                  </n-alert>
                </n-space>
              </n-collapse-item>
            </n-collapse>
          </n-space>
        </n-card>
      </n-space>
    </n-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useWorkbenchRefreshStore } from '../../stores/workbenchRefreshStore'
import { useMessage } from 'naive-ui'
import { chapterElementApi } from '../../api/chapterElement'
import type { ChapterElementDTO, ElementType, RelationType, Importance } from '../../api/chapterElement'
import { planningApi } from '../../api/planning'
import type { StoryNode } from '../../api/planning'
import { knowledgeApi } from '../../api/knowledge'
import type { ChapterSummary } from '../../api/knowledge'
import { bibleApi, type CharacterDTO, type LocationDTO } from '../../api/bible'
import type { GenerateChapterWorkflowResponse } from '../../api/workflow'
import type { AutopilotChapterAudit } from './ChapterStatusPanel.vue'
import ForeshadowChapterSuggestionsPanel from './ForeshadowChapterSuggestionsPanel.vue'
import ConsistencyReportPanel from './ConsistencyReportPanel.vue'

const props = withDefaults(
  defineProps<{
    slug: string
    currentChapterNumber?: number | null
    readOnly?: boolean
    lastWorkflowResult?: GenerateChapterWorkflowResponse | null
    qcChapterNumber?: number | null
    autopilotChapterReview?: AutopilotChapterAudit | null
  }>(),
  {
    currentChapterNumber: null,
    readOnly: false,
    lastWorkflowResult: null,
    qcChapterNumber: null,
    autopilotChapterReview: null,
  }
)

const message = useMessage()

const elements = ref<ChapterElementDTO[]>([])
const loading = ref(false)
const storyNodeId = ref<string | null>(null)
const storyNodeNotFound = ref(false)
const chapterPlan = ref<StoryNode | null>(null)
const knowledgeChapter = ref<ChapterSummary | null>(null)
const filterType = ref<ElementType | undefined>(undefined)

// Bible 数据用于 ID -> name 映射
const bibleCharacters = ref<CharacterDTO[]>([])
const bibleLocations = ref<LocationDTO[]>([])

const elementTypeOptions = [
  { label: '人物', value: 'character' },
  { label: '地点', value: 'location' },
  { label: '道具', value: 'item' },
  { label: '组织', value: 'organization' },
  { label: '事件', value: 'event' },
]

const relationTypeOptions = [
  { label: '出场', value: 'appears' },
  { label: '提及', value: 'mentioned' },
  { label: '场景', value: 'scene' },
  { label: '使用', value: 'uses' },
  { label: '参与', value: 'involved' },
  { label: '发生', value: 'occurs' },
]

const importanceOptions = [
  { label: '主要', value: 'major' },
  { label: '一般', value: 'normal' },
  { label: '次要', value: 'minor' },
]

const elemTypeLabel = (t: string) => elementTypeOptions.find(o => o.value === t)?.label ?? t
const elemTypeColor = (t: string): 'error' | 'warning' | 'info' | 'success' | 'default' => {
  const map: Record<string, 'error' | 'warning' | 'info' | 'success' | 'default'> = {
    character: 'error', location: 'success', item: 'warning', organization: 'info', event: 'default'
  }
  return map[t] ?? 'default'
}

const importanceLabel = (i: string) => importanceOptions.find(o => o.value === i)?.label ?? i
const relationLabel = (r: string) => relationTypeOptions.find(o => o.value === r)?.label ?? r

const getImportanceType = (importance: string): 'error' | 'warning' | 'info' | 'success' | 'default' => {
  const map: Record<string, 'error' | 'warning' | 'info' | 'success' | 'default'> = {
    major: 'error',
    normal: 'info',
    minor: 'default'
  }
  return map[importance] || 'default'
}

// 获取元素显示名称（从 Bible 映射）
const getElementDisplayName = (elementId: string, type: string): string => {
  if (type === 'character') {
    const char = bibleCharacters.value.find(c => c.id === elementId)
    if (char) return char.name
  }
  if (type === 'location') {
    const loc = bibleLocations.value.find(l => l.id === elementId)
    if (loc) return loc.name
  }
  // 没找到映射时返回原 ID
  return elementId
}

// 获取人物名称
const getCharacterName = (charId: string): string => {
  const char = bibleCharacters.value.find(c => c.id === charId)
  return char ? char.name : charId
}

const groupedCharacters = computed(() =>
  elements.value.filter(e => e.element_type === 'character')
)
const groupedLocations = computed(() =>
  elements.value.filter(e => e.element_type === 'location')
)
const groupedOther = computed(() =>
  elements.value.filter(e => e.element_type !== 'character' && e.element_type !== 'location')
)

const planMoodLine = computed(() => {
  const m = chapterPlan.value?.metadata
  if (!m || typeof m !== 'object') return ''
  const mood = m.mood ?? m.emotion ?? m.tone
  if (typeof mood === 'string' && mood.trim()) return mood
  if (Array.isArray(m.moods) && m.moods.length) return m.moods.join('、')
  return ''
})

const beatLines = computed(() => {
  const k = knowledgeChapter.value
  if (k?.beat_sections?.length) {
    return k.beat_sections.map(s => String(s || '').trim()).filter(Boolean)
  }
  const ol = chapterPlan.value?.outline?.trim()
  if (!ol) return []
  return ol.split(/\n+/).map(s => s.trim()).filter(s => s.length > 0)
})

const showBeatsCard = computed(() => {
  if (!props.currentChapterNumber) return false
  if (beatLines.value.length > 0) return true
  return !!(chapterPlan.value?.outline?.trim() || knowledgeChapter.value)
})

interface MicroBeat {
  description: string
  target_words: number
  focus: string
}

const microBeats = ref<MicroBeat[]>([])

const getBeatTypeColor = (focus: string): 'success' | 'warning' | 'error' | 'info' | 'default' => {
  const colorMap: Record<string, 'success' | 'warning' | 'error' | 'info' | 'default'> = {
    sensory: 'info',
    dialogue: 'success',
    action: 'warning',
    emotion: 'error',
  }
  return colorMap[focus] || 'default'
}

const hasSummaryBlock = computed(() => {
  if (!props.currentChapterNumber) return false
  const k = knowledgeChapter.value
  if (k && (k.summary?.trim() || k.key_events?.trim() || k.consistency_note?.trim())) return true
  return !!chapterPlan.value?.description?.trim()
})

const ghostAnnotationLines = computed(() => {
  const raw = props.lastWorkflowResult?.ghost_annotations
  if (!raw || !Array.isArray(raw) || raw.length === 0) return []
  const lines: string[] = []
  for (const item of raw) {
    if (item == null) continue
    if (typeof item === 'string') {
      lines.push(item)
      continue
    }
    if (typeof item === 'object') {
      const o = item as Record<string, unknown>
      const msg =
        (typeof o.message === 'string' && o.message) ||
        (typeof o.summary === 'string' && o.summary) ||
        (typeof o.text === 'string' && o.text) ||
        JSON.stringify(o)
      lines.push(msg)
    }
  }
  return lines
})

function formatTime(t: string) {
  try {
    return new Date(t).toLocaleString('zh-CN', {
      month: 'numeric',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return t
  }
}

function findChapterNode(nodes: StoryNode[], num: number): StoryNode | null {
  for (const node of nodes) {
    if (node.node_type === 'chapter' && node.number === num) return node
    if (node.children?.length) {
      const found = findChapterNode(node.children, num)
      if (found) return found
    }
  }
  return null
}

const resolveStoryNode = async () => {
  storyNodeId.value = null
  chapterPlan.value = null
  storyNodeNotFound.value = false
  if (!props.currentChapterNumber) return
  try {
    const res = await planningApi.getStructure(props.slug)
    const roots = res.data?.nodes ?? []
    const node = findChapterNode(roots, props.currentChapterNumber)
    if (node) {
      storyNodeId.value = node.id
      chapterPlan.value = node
    } else {
      storyNodeNotFound.value = true
    }
  } catch {
    storyNodeNotFound.value = true
  }
}

async function loadKnowledgeChapter() {
  knowledgeChapter.value = null
  if (!props.slug || !props.currentChapterNumber) return
  try {
    const k = await knowledgeApi.getKnowledge(props.slug)
    const row = k.chapters?.find(c => c.chapter_id === props.currentChapterNumber)
    knowledgeChapter.value = row ?? null
  } catch {
    knowledgeChapter.value = null
  }
}

const loadElements = async () => {
  if (!storyNodeId.value) return
  loading.value = true
  try {
    const res = await chapterElementApi.getElements(storyNodeId.value, filterType.value)
    elements.value = res.data
  } catch {
    message.error('加载章节元素失败')
  } finally {
    loading.value = false
  }
}

// 加载 Bible 数据用于名称映射
async function loadBible() {
  try {
    const bible = await bibleApi.getBible(props.slug)
    bibleCharacters.value = bible.characters || []
    bibleLocations.value = bible.locations || []
  } catch {
    // Bible 不存在时不报错
    bibleCharacters.value = []
    bibleLocations.value = []
  }
}

function onLocationClick(location: number) {
  message.info(`问题位置约在第 ${location} 字附近`)
}

watch(() => props.slug, async (slug) => {
  if (slug) {
    elements.value = []
    storyNodeId.value = null
    chapterPlan.value = null
    storyNodeNotFound.value = false
    await Promise.all([
      loadBible(),
      resolveStoryNode(),
      loadKnowledgeChapter(),
      loadElements()
    ])
  }
})

watch(() => props.currentChapterNumber, async () => {
  await resolveStoryNode()
  await loadKnowledgeChapter()
  await loadElements()
}, { immediate: false })

const refreshStore = useWorkbenchRefreshStore()
const { deskTick } = storeToRefs(refreshStore)
watch(deskTick, async () => {
  await resolveStoryNode()
  await loadKnowledgeChapter()
  await loadElements()
})

onMounted(async () => {
  await loadBible()
  await resolveStoryNode()
  await loadKnowledgeChapter()
  await loadElements()
})
</script>

<style scoped>
.ce-panel {
  padding: 0;
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ce-scroll {
  flex: 1;
  min-height: 0;
}

.card-title {
  font-size: 13px;
  font-weight: 600;
}

/* 节拍列表 */
.ce-beat-list {
  margin: 8px 0 0;
  padding-left: 1.2em;
  font-size: 12px;
  line-height: 1.8;
}

/* 微观节拍 */
.micro-beat-item {
  padding: 12px 14px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.04) 0%, rgba(139, 92, 246, 0.02) 100%);
  border: 1px solid rgba(99, 102, 241, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.micro-beat-item:hover {
  border-color: rgba(99, 102, 241, 0.2);
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.06) 0%, rgba(139, 92, 246, 0.04) 100%);
}

.micro-beat-header {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.micro-beat-desc {
  margin-top: 6px;
  padding-left: 12px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--n-text-color-2);
  border-left: 2px solid var(--n-border-color);
}

.micro-beat-item:hover .micro-beat-desc {
  border-left-color: var(--n-primary-color);
}

/* 元素分组标签 */
.ce-group-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

/* 元素卡片头部 */
.ce-card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

/* 只读元素项 */
.ce-item-readonly {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 8px;
  background: var(--n-color-modal);
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.ce-item-readonly:hover {
  border-color: var(--n-primary-color);
  background: rgba(99, 102, 241, 0.02);
}

.ce-element-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--n-text-color-1);
  margin-right: 8px;
}

/* 审阅行 */
.review-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

/* 张力进度条 */
.tension-bar {
  position: relative;
  width: 100px;
  height: 20px;
  background: var(--n-color-modal);
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid var(--n-border-color);
}

.tension-fill {
  height: 100%;
  background: linear-gradient(90deg, #10b981, #f59e0b, #ef4444);
  border-radius: 10px;
  transition: width 0.3s ease;
}

.tension-value {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 11px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

/* 质检折叠 */
.qc-collapse :deep(.n-collapse-item__header) {
  font-size: 12px;
}
</style>
