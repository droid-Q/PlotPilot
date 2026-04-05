<template>
  <div class="storyline-panel">
    <header class="panel-header">
      <div class="header-main">
        <div class="title-row">
          <h3 class="panel-title">故事线管理</h3>
          <n-tag size="small" round :bordered="false">Storylines</n-tag>
        </div>
        <p class="panel-lead">
          管理小说的<strong>主线、支线与暗线</strong>，规划故事线的起止章节和关键里程碑。
        </p>
      </div>
      <n-space class="header-actions" :size="8" align="center">
        <n-button size="small" secondary @click="openCreate">
          + 添加故事线
        </n-button>
        <n-button size="small" type="primary" :loading="loading" @click="loadStorylines">
          刷新
        </n-button>
      </n-space>
    </header>

    <div class="panel-content">
      <n-spin :show="loading">
        <n-empty v-if="storylines.length === 0" description="暂无故事线，点击「添加故事线」开始规划">
          <template #icon>
            <span style="font-size: 48px">📖</span>
          </template>
        </n-empty>

        <n-space v-else vertical :size="12">
          <n-card
            v-for="storyline in storylines"
            :key="storyline.id"
            size="small"
            :bordered="true"
            hoverable
          >
            <template #header>
              <div class="storyline-header">
                <n-tag :type="getTypeColor(storyline.storyline_type)" size="small" round>
                  {{ getTypeLabel(storyline.storyline_type) }}
                </n-tag>
                <n-tag :type="getStatusColor(storyline.status)" size="small" round>
                  {{ getStatusLabel(storyline.status) }}
                </n-tag>
              </div>
            </template>

            <n-space vertical :size="8">
              <div class="info-row">
                <n-text depth="3">章节范围:</n-text>
                <n-text>第 {{ storyline.estimated_chapter_start }} - {{ storyline.estimated_chapter_end }} 章</n-text>
              </div>
            </n-space>

            <template #action>
              <n-space :size="8">
                <n-button size="tiny" secondary @click="editStoryline(storyline)">编辑</n-button>
                <n-button size="tiny" type="error" secondary @click="deleteStoryline(storyline.id)">删除</n-button>
              </n-space>
            </template>
          </n-card>
        </n-space>
      </n-spin>
    </div>

    <!-- 创建/编辑故事线模态框 -->
    <n-modal v-model:show="showCreateModal" preset="card" :title="editingStoryline ? '编辑故事线' : '添加故事线'" style="width: 600px">
      <n-form ref="formRef" :model="formData" :rules="formRules" label-placement="left" label-width="120">
        <n-form-item label="故事线类型" path="storyline_type">
          <n-select
            v-model:value="formData.storyline_type"
            :options="typeOptions"
            placeholder="选择故事线类型"
          />
        </n-form-item>

        <n-form-item label="开始章节" path="estimated_chapter_start">
          <n-input-number
            v-model:value="formData.estimated_chapter_start"
            :min="1"
            placeholder="起始章节号"
            style="width: 100%"
          />
        </n-form-item>

        <n-form-item label="结束章节" path="estimated_chapter_end">
          <n-input-number
            v-model:value="formData.estimated_chapter_end"
            :min="1"
            placeholder="结束章节号"
            style="width: 100%"
          />
        </n-form-item>
      </n-form>

      <template #action>
        <n-space justify="end">
          <n-button @click="showCreateModal = false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="handleSubmit">确定</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { workflowApi } from '../../api/workflow'
import type { StorylineDTO } from '../../api/workflow'

interface Props {
  slug: string
}

const props = defineProps<Props>()
const message = useMessage()
const dialog = useDialog()

const loading = ref(false)
const saving = ref(false)
const storylines = ref<StorylineDTO[]>([])
const showCreateModal = ref(false)
const editingStoryline = ref<StorylineDTO | null>(null)

const formData = ref({
  storyline_type: 'main_plot',
  estimated_chapter_start: 1,
  estimated_chapter_end: 10
})

const formRules = {
  storyline_type: { required: true, message: '请选择故事线类型', trigger: 'change' },
  estimated_chapter_start: { required: true, type: 'number', message: '请输入开始章节', trigger: 'blur' },
  estimated_chapter_end: { required: true, type: 'number', message: '请输入结束章节', trigger: 'blur' }
}

const typeOptions = [
  { label: '主线', value: 'main_plot' },
  { label: '爱情线', value: 'romance' },
  { label: '复仇线', value: 'revenge' },
  { label: '悬疑线', value: 'mystery' },
  { label: '成长线', value: 'growth' },
  { label: '政治线', value: 'political' },
  { label: '冒险线', value: 'adventure' },
  { label: '家庭线', value: 'family' },
  { label: '友情线', value: 'friendship' }
]

const getTypeLabel = (type: string) => {
  const option = typeOptions.find(o => o.value === type)
  return option?.label || type
}

const getTypeColor = (type: string) => {
  const colors: Record<string, any> = {
    main_plot: 'primary',
    romance: 'error',
    revenge: 'warning',
    mystery: 'info',
    growth: 'success'
  }
  return colors[type] || 'default'
}

const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    active: '进行中',
    completed: '已完成',
    abandoned: '已废弃'
  }
  return labels[status] || status
}

const getStatusColor = (status: string) => {
  const colors: Record<string, any> = {
    active: 'success',
    completed: 'info',
    abandoned: 'default'
  }
  return colors[status] || 'default'
}

const loadStorylines = async () => {
  loading.value = true
  try {
    storylines.value = await workflowApi.getStorylines(props.slug)
  } catch (error: any) {
    message.error(error?.response?.data?.detail || '加载故事线失败')
  } finally {
    loading.value = false
  }
}

const openCreate = () => {
  editingStoryline.value = null
  formData.value = { storyline_type: 'main_plot', estimated_chapter_start: 1, estimated_chapter_end: 10 }
  showCreateModal.value = true
}

const handleSubmit = async () => {
  if (formData.value.estimated_chapter_end < formData.value.estimated_chapter_start) {
    message.error('结束章节必须大于等于开始章节')
    return
  }

  saving.value = true
  try {
    if (editingStoryline.value) {
      await workflowApi.updateStoryline(props.slug, editingStoryline.value.id, formData.value)
      message.success('故事线已更新')
    } else {
      await workflowApi.createStoryline(props.slug, formData.value)
      message.success('故事线创建成功')
    }
    showCreateModal.value = false
    await loadStorylines()
  } catch (error: any) {
    message.error(error?.response?.data?.detail || (editingStoryline.value ? '更新失败' : '创建失败'))
  } finally {
    saving.value = false
  }
}

const editStoryline = (storyline: StorylineDTO) => {
  editingStoryline.value = storyline
  formData.value = {
    storyline_type: storyline.storyline_type,
    estimated_chapter_start: storyline.estimated_chapter_start,
    estimated_chapter_end: storyline.estimated_chapter_end,
  }
  showCreateModal.value = true
}

const deleteStoryline = (id: string) => {
  dialog.warning({
    title: '确认删除',
    content: '删除后无法恢复，确定吗？',
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await workflowApi.deleteStoryline(props.slug, id)
        message.success('已删除')
        await loadStorylines()
      } catch (error: any) {
        message.error(error?.response?.data?.detail || '删除失败')
      }
    },
  })
}

watch(() => props.slug, (slug) => {
  if (slug) loadStorylines()
})

onMounted(() => {
  loadStorylines()
})
</script>

<style scoped>
.storyline-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--aitext-panel-muted);
}

.panel-header {
  padding: 16px;
  border-bottom: 1px solid var(--aitext-split-border);
  background: var(--app-surface);
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.header-main {
  flex: 1;
  min-width: 0;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.panel-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color-1);
}

.panel-lead {
  margin: 0;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-color-3);
}

.header-actions {
  flex-shrink: 0;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.storyline-header {
  display: flex;
  gap: 8px;
  align-items: center;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
</style>
