<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue'

import type { ResumeUploadRecord } from '#/types/pop-tail'

import { computed, onMounted, ref } from 'vue'

import { Page } from '@vben/common-ui'
import { createIconifyIcon } from '@vben/icons'

import {
  Alert,
  Button,
  Card,
  Col,
  Descriptions,
  Empty,
  List,
  Progress,
  Row,
  Space,
  Table,
  Tag,
} from 'ant-design-vue'

import {
  advanceResumeUploadApi,
  getResumeUploadListApi,
  recoverResumeUploadApi,
} from '#/api/pop-tail/admin'

type EvidenceAction = 'advance' | 'load' | 'resume'

type EvidenceEntry = {
  action: EvidenceAction
  activeTask: null | ResumeUploadRecord
  at: number
  averageProgress: number
  changedCount: number
  completed: number
  inProgress: number
  label: string
  total: number
}

const RefreshIcon = createIconifyIcon('mdi:refresh')
const AdvanceIcon = createIconifyIcon('mdi:skip-next-circle-outline')
const ResumeIcon = createIconifyIcon('mdi:restore')
const BoardIcon = createIconifyIcon('mdi:view-dashboard-outline')
const InsightIcon = createIconifyIcon('mdi:lightbulb-on-outline')

const rows = ref<ResumeUploadRecord[]>([])
const activeTask = ref<null | ResumeUploadRecord>(null)
const notice = ref('')
const error = ref('')
const loading = ref(false)
const acting = ref(false)
const evidenceTrail = ref<EvidenceEntry[]>([])

const suggestions = computed(() => {
  if (!activeTask.value) {
    return ['点击左侧任意任务，可在右侧查看当前判断与下一步建议。']
  }
  return [
    taskConclusion(activeTask.value),
    taskNextStep(activeTask.value),
    activeTask.value.progress >= 100
      ? '该任务已满足收尾验收条件。'
      : '如需继续推进，可先点击“推进进度”，再观察状态变化。',
  ]
})

const verificationChecklist = computed(() => [
  {
    detail: '刷新动作和当前任务快照会写入证据轨迹，便于后续回放。',
    done: evidenceTrail.value.some((item) => item.action === 'load'),
    label: '已记录刷新证据',
  },
  {
    detail: '推进动作应至少产生一次状态或进度变化，用于证明链路可继续前进。',
    done: evidenceTrail.value.some((item) => item.action === 'advance'),
    label: '已记录推进证据',
  },
  {
    detail: '恢复动作会把恢复链路也纳入前端证据，补齐断点续传场景。',
    done: evidenceTrail.value.some((item) => item.action === 'resume'),
    label: '已记录恢复证据',
  },
  {
    detail: '至少有一次交互需要观察到 changedCount > 0，证明状态真的变化。',
    done: evidenceTrail.value.some((item) => item.changedCount > 0),
    label: '已观察到状态变化',
  },
  {
    detail: '若页面中出现完成态，后续 browser smoke 只需截图收尾即可。',
    done:
      rows.value.some((item) => item.progress >= 100)
      || evidenceTrail.value.some((item) => item.completed > 0),
    label: '已看到完成态',
  },
])

const verificationSummary = computed(() => {
  const completed = verificationChecklist.value.filter((item) => item.done).length
  return `${completed}/${verificationChecklist.value.length} 项证据已就绪`
})

const latestEvidence = computed(() => evidenceTrail.value[0] ?? null)
const recentEvidence = computed(() => evidenceTrail.value.slice(0, 4))

const tableColumns: TableColumnsType<ResumeUploadRecord> = [
  {
    dataIndex: 'name',
    key: 'name',
    title: '文件',
  },
  {
    dataIndex: 'progress',
    key: 'progress',
    title: '进度',
    width: 110,
  },
  {
    dataIndex: 'status',
    key: 'status',
    title: '状态',
    width: 150,
  },
  {
    key: 'bar',
    title: '进度条',
    width: 220,
  },
  {
    key: 'advice',
    title: '结论',
  },
  {
    key: 'action',
    title: '操作',
    width: 90,
    fixed: 'right',
  },
]

function normalizeTask(item: Record<string, any> | ResumeUploadRecord): ResumeUploadRecord {
  return {
    ID: Number(item.ID ?? 0),
    name: String(item.name ?? ''),
    progress: Number(item.progress ?? 0),
    status: String(item.status ?? ''),
  }
}

function normalizeRows(list: Array<Record<string, any> | ResumeUploadRecord>) {
  return list.map((item) => normalizeTask(item))
}

function calculateAverageProgress(list: ResumeUploadRecord[]) {
  if (list.length === 0) return 0
  const total = list.reduce((sum, item) => sum + item.progress, 0)
  return Math.round(total / list.length)
}

function captureEvidence(
  action: EvidenceAction,
  nextRows: ResumeUploadRecord[],
  previousRows: ResumeUploadRecord[],
  currentTask: null | ResumeUploadRecord,
) {
  const previousMap = new Map(previousRows.map((item) => [item.ID, item]))
  const changedCount = nextRows.filter((item) => {
    const previous = previousMap.get(item.ID)
    if (!previous) return true
    return previous.progress !== item.progress || previous.status !== item.status
  }).length

  const labels: Record<EvidenceAction, string> = {
    advance: '推进进度',
    load: '刷新任务',
    resume: '恢复任务',
  }

  evidenceTrail.value = [
    {
      action,
      activeTask: currentTask ? normalizeTask(currentTask) : null,
      at: Date.now(),
      averageProgress: calculateAverageProgress(nextRows),
      changedCount,
      completed: nextRows.filter((item) => item.progress >= 100).length,
      inProgress: nextRows.filter((item) => item.progress < 100).length,
      label: labels[action],
      total: nextRows.length,
    },
    ...evidenceTrail.value,
  ].slice(0, 8)
}

function selectTask(item: Record<string, any> | ResumeUploadRecord) {
  activeTask.value = normalizeTask(item)
}

function taskConclusion(item: Record<string, any> | ResumeUploadRecord) {
  const task = normalizeTask(item)
  if (task.progress >= 100) return '该任务已完成，可做收尾验收。'
  if (task.progress >= 80) return '任务即将完成，建议再推进一次并核对状态。'
  if (task.status.includes('recover') || task.status.includes('resume')) {
    return '任务处于恢复链路，建议继续观察。'
  }
  return '任务仍在处理中，可继续推进或主动恢复。'
}

function taskNextStep(item: Record<string, any> | ResumeUploadRecord) {
  const task = normalizeTask(item)
  if (task.progress >= 100) return '进入上传完成或示例中心继续检查其他链路。'
  if (task.progress === 0) return '优先尝试恢复中断任务，再继续推进进度。'
  return '先推进一次进度，再确认是否需要执行恢复。'
}

function getRowClass(record: ResumeUploadRecord) {
  return record.ID === activeTask.value?.ID ? 'is-active-row' : ''
}

function statusColor(item: Record<string, any> | ResumeUploadRecord) {
  const task = normalizeTask(item)
  if (task.progress >= 100) return 'success'
  if (task.status.includes('recover') || task.status.includes('resume')) return 'processing'
  return 'warning'
}

function formatTime(value: number) {
  return new Date(value).toLocaleString('zh-CN')
}

async function copyText(text: string, successMessage: string) {
  error.value = ''
  try {
    await navigator.clipboard.writeText(text)
    notice.value = successMessage
  } catch {
    if (typeof window !== 'undefined') {
      window.prompt('当前环境不支持自动复制，请手动复制：', text)
    }
    notice.value = `${successMessage}（已切换为手动复制）`
  }
}

async function copyEvidenceSummary() {
  if (evidenceTrail.value.length === 0) return
  const payload = {
    exportedAt: new Date().toISOString(),
    checklist: verificationChecklist.value,
    latestEvidence: latestEvidence.value,
    recentEvidence: recentEvidence.value,
    activeTask: activeTask.value,
    rows: rows.value,
  }
  await copyText(JSON.stringify(payload, null, 2), '已复制断点续传证据摘要。')
}

function exportEvidenceSummary() {
  if (typeof window === 'undefined' || evidenceTrail.value.length === 0) {
    notice.value = '当前环境不支持导出证据，或尚未产生证据轨迹。'
    return
  }
  const content = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      checklist: verificationChecklist.value,
      trail: evidenceTrail.value,
      activeTask: activeTask.value,
      rows: rows.value,
    },
    null,
    2,
  )
  const blob = new Blob([content], { type: 'application/json;charset=utf-8' })
  const url = window.URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `breakpoint-evidence-${Date.now()}.json`
  link.click()
  window.URL.revokeObjectURL(url)
  notice.value = `已导出 ${evidenceTrail.value.length} 条断点续传证据。`
}

async function load() {
  loading.value = true
  error.value = ''
  const previousRows = [...rows.value]
  try {
    const result = await getResumeUploadListApi()
    const nextRows = normalizeRows(result.List)
    rows.value = nextRows
    activeTask.value = !activeTask.value || !nextRows.some((item) => item.ID === activeTask.value?.ID) ? nextRows[0] ?? null : nextRows.find((item) => item.ID === activeTask.value?.ID) ?? nextRows[0] ?? null;
    captureEvidence('load', nextRows, previousRows, activeTask.value)
    notice.value = '已刷新断点续传任务。'
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : '获取续传任务失败'
  } finally {
    loading.value = false
  }
}

async function advance() {
  acting.value = true
  error.value = ''
  const previousRows = [...rows.value]
  try {
    const result = await advanceResumeUploadApi()
    const nextRows = normalizeRows(result.List)
    rows.value = nextRows
    activeTask.value =
      nextRows.find((item) => item.ID === activeTask.value?.ID) ?? nextRows[0] ?? null
    captureEvidence('advance', nextRows, previousRows, activeTask.value)
    notice.value = '已向后端请求推进续传进度。'
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : '推进续传失败'
  } finally {
    acting.value = false
  }
}

async function resume() {
  acting.value = true
  error.value = ''
  const previousRows = [...rows.value]
  try {
    const result = await recoverResumeUploadApi()
    const nextRows = normalizeRows(result.List)
    rows.value = nextRows
    activeTask.value =
      nextRows.find((item) => item.progress < 100) ?? nextRows[0] ?? null
    captureEvidence('resume', nextRows, previousRows, activeTask.value)
    notice.value = '已向后端请求恢复中断任务。'
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : '恢复续传失败'
  } finally {
    acting.value = false
  }
}

onMounted(() => {
  void load()
})
</script>

<template>
  <Page
    title="断点续传工作台"
  >
    <div class="space-y-4 p-1">
      <Card>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-base font-medium">
              <BoardIcon class="text-[18px] text-primary" />
              <span>统一工作台骨架</span>
            </div>
            <p class="mb-0 text-text-secondary">
              通过摘要卡、任务表格、任务解读、建议区和证据轨迹，把断点续传的浏览器验收信息提前收口到页面里。
            </p>
          </div>
          <Space wrap>
            <Button :loading="loading" @click="load">
              <template #icon>
                <RefreshIcon />
              </template>
              刷新任务
            </Button>
            <Button :loading="acting" @click="advance">
              <template #icon>
                <AdvanceIcon />
              </template>
              推进进度
            </Button>
            <Button :loading="acting" type="primary" @click="resume">
              <template #icon>
                <ResumeIcon />
              </template>
              恢复任务
            </Button>
            <Button :disabled="evidenceTrail.length === 0" @click="copyEvidenceSummary">复制证据</Button>
            <Button :disabled="evidenceTrail.length === 0" @click="exportEvidenceSummary">导出证据</Button>
          </Space>
        </div>
      </Card>

      <Alert v-if="error" :message="error" show-icon type="error" />
      <Alert v-else-if="notice" :message="notice" show-icon type="success" />

      <Row :gutter="[16, 16]">
        <Col :lg="15" :span="24">
          <Card title="任务列表">
            <Table
              :columns="tableColumns"
              :data-source="rows"
              :loading="loading"
              :pagination="false"
              :row-class-name="getRowClass"
              row-key="ID"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'progress'">
                  {{ record.progress }}%
                </template>
                <template v-else-if="column.key === 'status'">
                  <Tag :color="statusColor(record)">{{ record.status }}</Tag>
                </template>
                <template v-else-if="column.key === 'bar'">
                  <Progress :percent="record.progress" size="small" />
                </template>
                <template v-else-if="column.key === 'advice'">
                  {{ taskConclusion(record) }}
                </template>
                <template v-else-if="column.key === 'action'">
                  <Button size="small" type="link" @click="selectTask(record)">查看</Button>
                </template>
              </template>
            </Table>
          </Card>
        </Col>

        <Col :lg="9" :span="24">
          <div class="space-y-4">
            <Card title="任务解读">
              <Descriptions v-if="activeTask" :column="1" size="small">
                <Descriptions.Item label="当前任务">{{ activeTask.name }}</Descriptions.Item>
                <Descriptions.Item label="任务状态">
                  <Tag :color="statusColor(activeTask)">{{ activeTask.status }}</Tag>
                </Descriptions.Item>
                <Descriptions.Item label="当前进度">
                  <Progress :percent="activeTask.progress" size="small" />
                </Descriptions.Item>
                <Descriptions.Item label="处理建议">
                  {{ taskConclusion(activeTask) }}
                </Descriptions.Item>
                <Descriptions.Item label="下一步">
                  {{ taskNextStep(activeTask) }}
                </Descriptions.Item>
              </Descriptions>
              <Empty v-else description="暂无选中任务" />
            </Card>

            <Card title="验收看板">
              <div class="evidence-list">
                <div v-for="item in verificationChecklist" :key="item.label" class="evidence-item">
                  <div class="evidence-head">
                    <strong>{{ item.label }}</strong>
                    <Tag :color="item.done ? 'success' : 'warning'">
                      {{ item.done ? '已就绪' : '待补充' }}
                    </Tag>
                  </div>
                  <p>{{ item.detail }}</p>
                </div>
              </div>
              <p class="evidence-summary">{{ verificationSummary }}</p>
            </Card>

            <Card>
              <template #title>
                <div class="flex items-center gap-2">
                  <InsightIcon class="text-[18px] text-primary" />
                  <span>建议区</span>
                </div>
              </template>
              <List :data-source="suggestions" size="small">
                <template #renderItem="{ item }">
                  <List.Item>{{ item }}</List.Item>
                </template>
              </List>
            </Card>

            <Card title="证据轨迹">
              <div v-if="recentEvidence.length > 0" class="evidence-list">
                <div v-for="item in recentEvidence" :key="`${item.action}-${item.at}`" class="evidence-item">
                  <div class="evidence-head">
                    <strong>{{ item.label }}</strong>
                    <span>{{ formatTime(item.at) }}</span>
                  </div>
                  <p>
                    总任务 {{ item.total }}，进行中 {{ item.inProgress }}，已完成 {{ item.completed }}，平均进度
                    {{ item.averageProgress }}%，变化任务 {{ item.changedCount }}。
                  </p>
                  <p v-if="item.activeTask">
                    当前焦点：{{ item.activeTask.name }} · {{ item.activeTask.progress }}% ·
                    {{ item.activeTask.status }}
                  </p>
                </div>
              </div>
              <Empty v-else description="先执行刷新 / 推进 / 恢复动作，页面会自动积累证据。" />
            </Card>
          </div>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<style scoped>
:deep(.is-active-row > td) {
  background: rgb(24 144 255 / 8%) !important;
}

.evidence-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.evidence-item {
  border: 1px solid var(--ant-color-border-secondary);
  border-radius: 12px;
  padding: 12px 14px;
  background: var(--ant-color-fill-quaternary);
}

.evidence-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.evidence-item p,
.evidence-summary {
  margin: 8px 0 0;
  color: var(--ant-color-text-description);
  font-size: 13px;
}
</style>
