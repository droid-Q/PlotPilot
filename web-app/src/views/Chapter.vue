<template>
  <n-spin :show="pageLoading" class="chapter-spin" description="加载章节…">
  <div class="chapter">
    <header class="chapter-header">
      <n-space align="center" :wrap="false">
        <n-button quaternary round @click="goBack">
          <template #icon>
            <span class="ico-back">←</span>
          </template>
          工作台
        </n-button>
        <n-divider vertical />
        <h2 class="chapter-heading">第 {{ chapterId }} 章</h2>
        <n-tag :type="saveStatus === 'saved' ? 'success' : saveStatus === 'saving' ? 'warning' : 'default'" round size="small">
          {{ saveStatusText }}
        </n-tag>
      </n-space>

      <n-space :size="8" :wrap="false">
        <n-button-group>
          <n-button size="small" @click="prevChapter" :disabled="!canPrev">
            <template #icon>
              <span class="ico-tiny">◀</span>
            </template>
            上一章
          </n-button>
          <n-button size="small" @click="nextChapter" :disabled="!canNext">
            下一章
            <template #icon>
              <span class="ico-tiny">▶</span>
            </template>
          </n-button>
        </n-button-group>

        <n-dropdown :options="toolOptions" @select="handleToolSelect">
          <n-button size="small" secondary>工具</n-button>
        </n-dropdown>

        <n-button size="small" quaternary @click="goCastGraph">关系图</n-button>

        <n-button type="primary" size="small" round :loading="saving" @click="saveContent" :disabled="!contentDirty">
          保存
        </n-button>
      </n-space>
    </header>

    <n-split direction="horizontal" :default-size="0.72" :min="0.55" :max="0.88">
      <template #1>
        <div class="editor-area">
          <n-input
            v-model:value="content"
            type="textarea"
            class="content-editor"
            placeholder="开始写作…&#10;&#10;Ctrl+S 保存 · 自动保存约 30 秒"
            @update:value="onInput"
            :autosize="{ minRows: 22 }"
          />
          <div class="editor-footer">
            <n-space>
              <n-text depth="3">{{ wordCount }} 字</n-text>
              <n-divider vertical />
              <n-text depth="3">{{ lineCount }} 行</n-text>
              <n-divider vertical />
              <n-text depth="3" v-if="lastSaveTime">上次保存 {{ lastSaveTime }}</n-text>
            </n-space>
            <n-button size="small" quaternary @click="showPreview = !showPreview">
              {{ showPreview ? '隐藏预览' : 'Markdown 预览' }}
            </n-button>
          </div>

          <transition name="preview-slide">
            <div v-if="showPreview" class="preview-panel">
              <n-divider title-placement="left">预览</n-divider>
              <div class="preview-content markdown-body md-body" v-html="previewHtml" />
            </div>
          </transition>
        </div>
      </template>

      <template #2>
        <div class="review-panel">
          <n-tabs type="segment" animated class="review-tabs">
            <n-tab-pane name="review" tab="审定">
              <n-form label-placement="top" class="review-form">
                <n-form-item label="状态">
                  <n-radio-group v-model:value="reviewStatus" name="review-status">
                    <n-space>
                      <n-radio value="pending">待阅</n-radio>
                      <n-radio value="ok">已定稿</n-radio>
                      <n-radio value="revise">需修订</n-radio>
                    </n-space>
                  </n-radio-group>
                </n-form-item>
                <n-form-item label="批注">
                  <n-input v-model:value="reviewMemo" type="textarea" :rows="10" placeholder="审读意见…" />
                </n-form-item>
                <n-space vertical :size="8" style="width: 100%">
                  <n-button block :loading="savingAiReview" secondary @click="runAiReview(false)">
                    生成审读意见
                  </n-button>
                  <n-button block :loading="savingAiReview" type="info" secondary @click="runAiReview(true)">
                    生成并写入审定
                  </n-button>
                  <n-text depth="3" style="font-size: 11px; line-height: 1.45">
                    基于合并正文（含 chapters/NNN 下分场景 parts）与大纲一句纲；「生成意见」仅填入上方表单项。
                  </n-text>
                </n-space>
                <n-button type="primary" block round :loading="savingReview" @click="saveReview">保存审定</n-button>
              </n-form>
            </n-tab-pane>

            <n-tab-pane name="info" tab="信息">
              <n-space vertical :size="16" class="info-stats">
                <n-statistic label="字数" :value="wordCount" />
                <n-statistic label="行数" :value="lineCount" />
                <n-statistic label="段落" :value="paragraphCount" />
                <n-divider />
                <div v-if="chapterStructure">
                  <n-text depth="3" class="meta-line"
                    >合并正文约 {{ chapterStructure.composite_char_len }} 字（含分场景拼接）</n-text
                  >
                  <n-text depth="3" class="meta-line meta-mono"
                    >目录 {{ chapterStructure.storage_dir || '（扁平旧版或未建目录）' }}</n-text
                  >
                </div>
                <n-divider />
                <n-text depth="3" class="meta-line">创建：{{ createTime }}</n-text>
                <n-text depth="3" class="meta-line">修改：{{ updateTime }}</n-text>
              </n-space>
            </n-tab-pane>
          </n-tabs>
        </div>
      </template>
    </n-split>
  </div>
  </n-spin>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { bookApi } from '../api/book'
import { useStatsStore } from '../stores/statsStore'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const statsStore = useStatsStore()

const slug = route.params.slug as string
const chapterId = computed(() => {
  const id = Number(route.params.id as string)
  if (isNaN(id) || id <= 0) {
    message.error('无效的章节ID')
    return null
  }
  return id
})

const goHome = () => {
  router.push(`/book/${slug}/workbench`)
}

watch(chapterId, (newId) => {
  if (newId === null) {
    goHome()
  }
}, { immediate: true })

const content = ref('')
const saving = ref(false)
const saveStatus = ref<'unsaved' | 'saving' | 'saved'>('saved')
const lastSaveTime = ref('')
const saveTimer = ref<ReturnType<typeof setTimeout> | null>(null)

const reviewStatus = ref('pending')
const reviewMemo = ref('')
const savingReview = ref(false)
const savingAiReview = ref(false)
const chapterStructure = ref<{
  composite_char_len: number
  storage_dir: string | null
} | null>(null)

const showPreview = ref(false)
const chapterIds = ref<number[]>([])
const pageLoading = ref(true)

const wordCount = computed(() => content.value.replace(/\s/g, '').length)
const lineCount = computed(() => (content.value ? content.value.split('\n').length : 0))
const paragraphCount = computed(() =>
  content.value ? content.value.split(/\n\s*\n/).filter(p => p.trim()).length : 0
)

const previewHtml = ref<string>('')
const markdownDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null)

const updatePreview = (debounce = false) => {
  const parseMarkdown = () => {
    const html = marked.parse(content.value, { breaks: true, async: false }) as string
    const sanitizedHtml = DOMPurify.sanitize(html)
    previewHtml.value = sanitizedHtml
  }

  if (debounce) {
    if (markdownDebounceTimer.value) clearTimeout(markdownDebounceTimer.value)
    markdownDebounceTimer.value = window.setTimeout(parseMarkdown, 300)
  } else {
    parseMarkdown()
  }
}

const saveStatusText = computed(() => {
  const map = { unsaved: '未保存', saving: '保存中…', saved: '已保存' }
  return map[saveStatus.value]
})

const contentDirty = computed(() => saveStatus.value === 'unsaved')

const currentChapterIndex = computed(() => {
  return chapterIds.value.indexOf(chapterId.value)
})

const canPrev = computed(() => {
  const i = currentChapterIndex.value
  return i > 0
})

const canNext = computed(() => {
  const i = currentChapterIndex.value
  return i >= 0 && i < chapterIds.value.length - 1
})

const createTime = ref('—')
const updateTime = ref('—')

const toolOptions = [
  { label: '复制全文', key: 'copy' },
  { label: '清空正文', key: 'clear' },
]

const handleToolSelect = (key: string) => {
  if (key === 'copy') {
    void navigator.clipboard.writeText(content.value).then(
      () => message.success('已复制'),
      () => message.error('复制失败')
    )
  }
  if (key === 'clear') {
    content.value = ''
    onInput()
    updatePreview(false)
  }
}

const onInput = () => {
  saveStatus.value = 'unsaved'
  if (saveTimer.value) clearTimeout(saveTimer.value)
  saveTimer.value = window.setTimeout(() => {
    void saveContent()
  }, 30000)
  updatePreview(true)
}

const saveContent = async () => {
  if (saving.value) return
  saving.value = true
  saveStatus.value = 'saving'

  try {
    await bookApi.saveChapterBody(slug, chapterId.value, content.value)
    saveStatus.value = 'saved'
    lastSaveTime.value = new Date().toLocaleTimeString('zh-CN', { hour12: false })
    updateTime.value = new Date().toLocaleString('zh-CN', { hour12: false })
    message.success('已保存')
    // Refresh book stats after successful save
    statsStore.onChapterSaved(slug, chapterId.value)
  } catch (error) {
    console.error('Failed to save content:', error)
    saveStatus.value = 'unsaved'
    message.error('保存失败，请稍后重试')
  } finally {
    saving.value = false
  }
}

const saveReview = async () => {
  savingReview.value = true
  try {
    await bookApi.saveChapterReview(slug, chapterId.value, reviewStatus.value, reviewMemo.value)
    message.success('审定已保存')
    // Refresh book stats after successful save
    statsStore.onChapterSaved(slug, chapterId.value)
  } catch (error) {
    console.error('Failed to save review:', error)
    message.error('保存失败，请稍后重试')
  } finally {
    savingReview.value = false
  }
}

const runAiReview = async (save: boolean) => {
  savingAiReview.value = true
  try {
    const r = await bookApi.reviewChapterAi(slug, chapterId.value, save)
    reviewStatus.value = r.status
    reviewMemo.value = r.memo
    message.success(save ? '已写入审定意见' : '已填入审读意见')
  } catch (e: any) {
    message.error(e?.response?.data?.detail || '生成失败')
  } finally {
    savingAiReview.value = false
  }
}

const goBack = () => {
  router.push(`/book/${slug}/workbench`)
}

const goCastGraph = () => {
  router.push({ path: `/book/${slug}/cast`, query: { chapter: String(chapterId.value) } })
}

const prevChapter = () => {
  const i = currentChapterIndex.value
  if (i > 0) router.push(`/book/${slug}/chapter/${chapterIds.value[i - 1]}`)
}

const nextChapter = () => {
  const i = currentChapterIndex.value
  if (i >= 0 && i < chapterIds.value.length - 1) {
    router.push(`/book/${slug}/chapter/${chapterIds.value[i + 1]}`)
  }
}

const onKeySave = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    void saveContent()
  }
}

const loadChapter = async () => {
  const cid = chapterId.value
  if (cid === null) {
    return
  }

  // Parallel execution of independent API calls
  const [desk, body, rev, structureResult] = await Promise.allSettled([
    bookApi.getDesk(slug),
    bookApi.getChapterBody(slug, cid),
    bookApi.getChapterReview(slug, cid),
    bookApi.getChapterStructure(slug, cid)
  ])

  // Handle desk API result
  if (desk.status === 'fulfilled') {
    chapterIds.value = desk.value.chapters.map(c => c.id).sort((a, b) => a - b)
  } else {
    console.error('Failed to load desk:', desk.reason)
  }

  // Handle body API result
  if (body.status === 'fulfilled') {
    content.value = body.value.content || ''
    if (content.value) {
      createTime.value = new Date().toLocaleString('zh-CN', { hour12: false })
      updateTime.value = createTime.value
    }
    updatePreview(false)
  } else {
    console.error('Failed to load chapter body:', body.reason)
  }

  // Handle review API result
  if (rev.status === 'fulfilled') {
    reviewStatus.value = rev.value.status
    reviewMemo.value = rev.value.memo
  }

  // Handle structure API result (this one is optional, can fail gracefully)
  if (structureResult.status === 'fulfilled') {
    chapterStructure.value = {
      composite_char_len: structureResult.value.composite_char_len,
      storage_dir: structureResult.value.storage_dir ?? null,
    }
  } else {
    console.warn('Failed to load chapter structure:', structureResult.reason)
    chapterStructure.value = null
  }

  saveStatus.value = 'saved'
}

watch(
  () => route.params.id,
  async () => {
    if (route.name !== 'Chapter') return
    if (saveTimer.value) {
      clearTimeout(saveTimer.value)
      saveTimer.value = null
    }
    pageLoading.value = true
    try {
      await loadChapter()
    } catch (error) {
      console.error('Failed to load chapter:', error)
      message.error('加载章节失败')
    } finally {
      pageLoading.value = false
    }
  }
)

onMounted(async () => {
  window.addEventListener('keydown', onKeySave)
  try {
    await loadChapter()
  } catch (error) {
    console.error('Failed to load chapter:', error)
    message.error('加载章节失败')
  } finally {
    pageLoading.value = false
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeySave)
  if (saveTimer.value) clearTimeout(saveTimer.value)
  if (markdownDebounceTimer.value) clearTimeout(markdownDebounceTimer.value)
})
</script>

<style scoped>
.chapter-spin {
  height: 100vh;
  min-height: 0;
}

.chapter-spin :deep(.n-spin-content) {
  min-height: 100%;
  height: 100%;
}

.chapter {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--app-page-bg, #f0f2f8);
}

.chapter :deep(.n-split) {
  flex: 1;
  min-height: 0;
}

.chapter-header {
  flex-shrink: 0;
  padding: 12px 18px;
  border-bottom: 1px solid var(--app-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  background: var(--app-surface, #fff);
}

.chapter-heading {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
}

.ico-back {
  font-size: 15px;
}

.ico-tiny {
  font-size: 10px;
  opacity: 0.8;
}

.editor-area {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 14px 16px;
  background: var(--app-surface);
}

.content-editor {
  flex: 1;
  min-height: 0;
  font-size: 16px;
  line-height: 1.85;
}

.content-editor :deep(textarea) {
  font-family: 'Source Han Serif SC', 'Noto Serif SC', Georgia, serif;
  line-height: 1.85;
}

.editor-footer {
  padding: 10px 0 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.preview-slide-enter-active,
.preview-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.preview-slide-enter-from,
.preview-slide-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.preview-panel {
  margin-top: 8px;
  padding: 12px 14px;
  border-radius: 12px;
  background: #f8fafc;
  border: 1px solid var(--app-border);
  max-height: 42vh;
  overflow: auto;
}

.preview-content {
  font-size: 14px;
}

.review-panel {
  height: 100%;
  min-height: 0;
  padding: 12px 14px;
  background: linear-gradient(180deg, #f8fafc 0%, #eef2ff 100%);
  border-left: 1px solid var(--app-border);
}

.review-tabs {
  height: 100%;
}

.review-tabs :deep(.n-tab-pane) {
  padding-top: 12px;
}

.review-form {
  max-width: 100%;
}

.info-stats {
  padding: 8px 0;
}

.meta-line {
  font-size: 13px;
}

.meta-mono {
  font-family: ui-monospace, Consolas, monospace;
  font-size: 12px;
  word-break: break-all;
}
</style>
