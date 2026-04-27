<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue'
import type {
  UploadChangeParam,
  UploadFile,
} from 'ant-design-vue/es/upload/interface'

import { computed, onMounted, ref } from 'vue'

import { Page } from '@vben/common-ui'
import { createIconifyIcon } from '@vben/icons'

import {
  Alert,
  Button,
  Card,
  Col,
  List,
  Popconfirm,
  Row,
  Space,
  Table,
  Tag,
  Upload,
} from 'ant-design-vue'

import {
  completeUploadQueueApi,
  getUploadQueueApi,
  saveUploadQueueApi,
} from '#/api/pop-tail/admin'
import type { UploadFileRecord } from '#/types/pop-tail'

type QueueDraftFile = {
  name: string
  size: string
}

const RefreshIcon = createIconifyIcon('mdi:refresh')
const CopyIcon = createIconifyIcon('mdi:content-copy')
const UploadIcon = createIconifyIcon('mdi:tray-arrow-up')
const ReplaceIcon = createIconifyIcon('mdi:playlist-remove')
const ConfirmIcon = createIconifyIcon('mdi:check-decagram-outline')
const ClearIcon = createIconifyIcon('mdi:broom')

const rows = ref<UploadFileRecord[]>([])
const selectedFiles = ref<UploadFile[]>([])
const notice = ref('')
const error = ref('')
const saving = ref(false)

const pendingSelection = computed<QueueDraftFile[]>(() =>
  uniqueBySignature(
    selectedFiles.value.map((item) => ({
      name: item.name,
      size: formatBytes(item.size ?? item.originFileObj?.size ?? 0),
    })),
  ),
)
const pendingCount = computed(
  () => rows.value.filter((item) => item.status !== '上传完成').length,
)
const completedCount = computed(
  () => rows.value.filter((item) => item.status === '上传完成').length,
)
const totalBytes = computed(() =>
  rows.value.reduce((sum, item) => sum + parseSize(item.size), 0),
)
const formattedTotalSize = computed(() => formatBytes(totalBytes.value))
const duplicateCount = computed(() => {
  const counter = new Map<string, number>()
  for (const item of rows.value) {
    const key = `${item.name}__${item.size}`
    counter.set(key, (counter.get(key) ?? 0) + 1)
  }
  return [...counter.values()].filter((count) => count > 1).length
})
const statusText = computed(() => {
  if (saving.value) return '正在同步后台上传队列…'
  if (pendingSelection.value.length) {
    return `已选 ${pendingSelection.value.length} 个文件，等待写入队列`
  }
  if (!rows.value.length) return '当前队列为空，可先选择文件后追加或覆盖'
  return `队列中 ${rows.value.length} 项，待上传 ${pendingCount.value} 项`
})
const queueHealth = computed(() => {
  if (!rows.value.length) return '空队列'
  if (!pendingCount.value) return '已全部完成'
  if (duplicateCount.value) return '需清理重复项'
  return '待继续上传'
})

const suggestions = computed(() => [
  '追加：保留既有队列，并把本次选择文件补充进去，适合连续补交。',
  '覆盖：按本次选择重建队列，适合重新提交批次或移除旧队列污染。',
  '确认全部上传：调用完成接口，把全部项标记为上传完成，便于示例链路收口。',
])

const tableColumns: TableColumnsType<UploadFileRecord> = [
  {
    dataIndex: 'ID',
    key: 'ID',
    title: '#',
    width: 80,
  },
  {
    dataIndex: 'name',
    key: 'name',
    title: '文件',
  },
  {
    dataIndex: 'size',
    key: 'size',
    title: '大小',
    width: 120,
  },
  {
    dataIndex: 'status',
    key: 'status',
    title: '状态',
    width: 130,
  },
  {
    key: 'advice',
    title: '建议',
  },
  {
    key: 'action',
    title: '操作',
    width: 100,
    fixed: 'right',
  },
]

function beforeUpload() {
  return false
}

function handleUploadChange(info: UploadChangeParam<UploadFile>) {
  selectedFiles.value = uniqueUploadFiles(
    info.fileList.map((item) => ({
      name: item.name,
      originFileObj: item.originFileObj,
      size: item.size ?? item.originFileObj?.size,
      status: item.status,
      uid: item.uid,
    })),
  )

  if (selectedFiles.value.length) {
    notice.value = `已选择 ${selectedFiles.value.length} 个文件，等待入队。`
    error.value = ''
  }
}

function handleRemoveSelected(file: UploadFile) {
  selectedFiles.value = selectedFiles.value.filter((item) => item.uid !== file.uid)
  return true
}

async function loadQueue() {
  saving.value = true
  error.value = ''
  try {
    const result = await getUploadQueueApi()
    rows.value = result.List
    notice.value = rows.value.length ? '已从后台刷新上传队列。' : '后台上传队列当前为空。'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取上传队列失败'
  } finally {
    saving.value = false
  }
}

async function replaceQueue() {
  if (!pendingSelection.value.length) {
    error.value = '请先选择至少一个文件'
    return
  }
  await persistQueue('replace', '已按本次选择覆盖上传队列。')
}

async function appendQueue() {
  if (!pendingSelection.value.length) {
    error.value = '请先选择至少一个文件'
    return
  }
  await persistQueue('append', '已将本次文件追加到上传队列。')
}

async function persistQueue(mode: 'append' | 'replace', successText: string) {
  saving.value = true
  error.value = ''
  try {
    const result = await saveUploadQueueApi({
      files: pendingSelection.value,
      mode,
    })
    rows.value = result.List
    clearSelection()
    notice.value = successText
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存上传队列失败'
  } finally {
    saving.value = false
  }
}

async function completeQueue() {
  saving.value = true
  error.value = ''
  try {
    const result = await completeUploadQueueApi()
    rows.value = result.List
    notice.value = rows.value.length
      ? '后台已将当前队列标记为上传完成。'
      : '当前没有可完成的队列项。'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '完成上传队列失败'
  } finally {
    saving.value = false
  }
}

async function removeFromQueue(id: number) {
  saving.value = true
  error.value = ''
  try {
    const remaining = rows.value
      .filter((item) => item.ID !== id)
      .map((item) => ({ name: item.name, size: item.size }))
    const result = await saveUploadQueueApi({
      files: remaining,
      mode: 'replace',
    })
    rows.value = result.List
    notice.value = `已从后台队列移除文件 #${id}。`
  } catch (err) {
    error.value = err instanceof Error ? err.message : '移除队列项失败'
  } finally {
    saving.value = false
  }
}

async function copyQueueSummary() {
  const text = [
    `队列总数：${rows.value.length}`,
    `待上传：${pendingCount.value}`,
    `已完成：${completedCount.value}`,
    `总大小：${formattedTotalSize.value}`,
    `状态：${queueHealth.value}`,
  ].join('\n')

  try {
    await navigator.clipboard.writeText(text)
    notice.value = '已复制上传队列摘要。'
  } catch {
    window.prompt('当前环境不支持自动复制，请手动复制：', text)
    notice.value = '已切换为手动复制上传队列摘要。'
  }
}

function pendingAdvice(item: QueueDraftFile) {
  return rows.value.some((row) => row.name === item.name && row.size === item.size)
    ? '与当前队列存在同签名文件，追加前请确认是否允许重复。'
    : '可安全写入当前上传队列。'
}

function rowAdvice(item: UploadFileRecord | Record<string, any>) {
  if (item.status === '上传完成') return '该文件已完成上传，可留作验收记录。'
  return '该文件仍在待传队列，可继续上传或直接移除。'
}

function clearSelection() {
  selectedFiles.value = []
}

function uniqueBySignature(files: QueueDraftFile[]) {
  const store = new Map<string, QueueDraftFile>()
  for (const item of files) {
    store.set(`${item.name}__${item.size}`, item)
  }
  return Array.from(store.values())
}

function uniqueUploadFiles(files: UploadFile[]) {
  const store = new Map<string, UploadFile>()
  for (const item of files) {
    const size = item.size ?? item.originFileObj?.size ?? 0
    store.set(`${item.name}__${size}`, item)
  }
  return Array.from(store.values())
}

function parseSize(size: string) {
  const match = size.trim().match(/^([\d.]+)\s*(B|KB|MB|GB)$/i)
  if (!match) return 0
  const value = Number(match[1])
  const unit = match[2]?.toUpperCase() ?? 'B'
  const unitMap: Record<string, number> = {
    B: 1,
    GB: 1024 ** 3,
    KB: 1024,
    MB: 1024 ** 2,
  }
  return Math.round(value * (unitMap[unit] ?? 1))
}

function formatBytes(bytes: number) {
  if (bytes <= 0) return '0 B'
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`
  if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(2)} MB`
  if (bytes >= 1024) return `${Math.ceil(bytes / 1024)} KB`
  return `${bytes} B`
}

onMounted(() => {
  void loadQueue()
})
</script>

<template>
  <Page
    title="上传示例工作台"
  >
    <div class="space-y-4 p-1">
      <Card>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-base font-medium">
              <UploadIcon class="text-[18px] text-primary" />
              <span>主操作区</span>
            </div>
            <p class="mb-0 text-text-secondary">
              使用 Ant Design Vue 的 Upload、Card、List、Table 组合出统一演示页，同时保持原有上传队列逻辑不变。
            </p>
          </div>
          <Space wrap>
            <Button :loading="saving" @click="loadQueue">
              <template #icon>
                <RefreshIcon />
              </template>
              刷新队列
            </Button>
            <Button :disabled="!rows.length" @click="copyQueueSummary">
              <template #icon>
                <CopyIcon />
              </template>
              复制摘要
            </Button>
          </Space>
        </div>

        <div class="mt-4 flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="flex-1 rounded-xl border border-dashed border-[var(--ant-color-border)] bg-[var(--ant-color-fill-alter)]/20 p-4">
            <div class="mb-2 text-sm font-medium">待入队文件</div>
            <div class="mb-3 text-xs text-text-secondary">{{ statusText }}</div>
            <Upload
              :before-upload="beforeUpload"
              :file-list="selectedFiles"
              :multiple="true"
              @change="handleUploadChange"
              @remove="handleRemoveSelected"
            >
              <Button >
                <template #icon>
                  <UploadIcon />
                </template>
                选择文件
              </Button>
            </Upload>
          </div>

          <Space wrap>
            <Button :disabled="saving || !pendingSelection.length" @click="replaceQueue">
              <template #icon>
                <ReplaceIcon />
              </template>
              覆盖队列
            </Button>
            <Button :disabled="saving || !pendingSelection.length" type="primary" @click="appendQueue">
              <template #icon>
                <UploadIcon />
              </template>
              追加到队列
            </Button>
            <Button :disabled="saving || !rows.length" @click="completeQueue">
              <template #icon>
                <ConfirmIcon />
              </template>
              确认全部上传
            </Button>
            <Button :disabled="saving || !selectedFiles.length" @click="clearSelection">
              <template #icon>
                <ClearIcon />
              </template>
              清空选择
            </Button>
          </Space>
        </div>
      </Card>

      <Alert v-if="error" :message="error" show-icon type="error" />
      <Alert v-else-if="notice" :message="notice" show-icon type="success" />

      <Row :gutter="[16, 16]">
        <Col :lg="14" :span="24">
          <Card title="本次待入队文件">
            <List :data-source="pendingSelection" size="small">
              <template #renderItem="{ item }">
                <List.Item>
                  <List.Item.Meta :description="pendingAdvice(item)" :title="item.name" />
                  <template #extra>
                    <Tag>{{ item.size }}</Tag>
                  </template>
                </List.Item>
              </template>
            </List>
          </Card>
        </Col>

        <Col :lg="10" :span="24">
          <Card title="建议区">
            <List :data-source="suggestions" size="small">
              <template #renderItem="{ item }">
                <List.Item>{{ item }}</List.Item>
              </template>
            </List>
          </Card>
        </Col>
      </Row>

      <Card title="上传队列">
        <template #extra>
          <Tag :color="queueHealth === '已全部完成' ? 'success' : queueHealth === '需清理重复项' ? 'warning' : 'processing'">
            {{ queueHealth }}
          </Tag>
        </template>

        <Table
          :columns="tableColumns"
          :data-source="rows"
          :loading="saving"
          :pagination="false"
          row-key="ID"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <Tag :color="record.status === '上传完成' ? 'success' : 'processing'">
                {{ record.status }}
              </Tag>
            </template>
            <template v-else-if="column.key === 'advice'">
              {{ rowAdvice(record) }}
            </template>
            <template v-else-if="column.key === 'action'">
              <Popconfirm
                cancel-text="取消"
                ok-text="移除"
                title="确认从上传队列中移除该文件吗？"
                @confirm="removeFromQueue(record.ID)"
              >
                <Button danger size="small" type="link">移除</Button>
              </Popconfirm>
            </template>
          </template>
        </Table>
      </Card>
    </div>
  </Page>
</template>
