<script lang="ts" setup>
import type { StepProps, UploadFile } from 'ant-design-vue'
import type { FileType } from 'ant-design-vue/es/upload/interface'

import type { ScanSessionRecord } from '#/types/gin-ai-admin'

import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import { Page } from '@vben/common-ui'
import { createIconifyIcon } from '@vben/icons'

import {
  Alert,
  Button,
  Card,
  Col,
  Descriptions,
  List,
  Row,
  Space,
  Steps,
  Tag,
  Upload,
} from 'ant-design-vue'
import { LIST_IGNORE } from 'ant-design-vue/es/upload/Upload'

import { getScanSessionApi, saveScanSessionApi } from '#/api/gin-ai-admin/admin'

interface ScanDraftFile {
  file: File
  id: string
  name: string
  previewUrl: string
  sizeLabel: string
}

const CopyIcon = createIconifyIcon('mdi:content-copy')
const RefreshIcon = createIconifyIcon('mdi:refresh')
const ScanIcon = createIconifyIcon('mdi:qrcode-scan')
const UploadIcon = createIconifyIcon('mdi:image-plus-outline')
const ConfirmIcon = createIconifyIcon('mdi:check-circle-outline')
const ResetIcon = createIconifyIcon('mdi:restart')

const route = useRoute()
const busy = ref(false)
const notice = ref('')
const error = ref('')
const files = ref<ScanDraftFile[]>([])
const session = ref<ScanSessionRecord>({
  sessionId: '',
  status: '待扫码',
})

function getReadableError(error: unknown) {
  return error instanceof Error ? error.message : '获取扫码会话失败'
}

function isUnauthorizedError(error: unknown) {
  const message = getReadableError(error)
  return /\b401\b|Unauthorized/i.test(message)
}

const steps = [
  { desc: '入口已生成，但尚未由移动端确认。', status: '待扫码', title: '等待设备扫码' },
  {
    desc: '页面已进入会话，可继续选择需要上传的图片。',
    status: '已扫码，待选择文件',
    title: '扫码已确认',
  },
  {
    desc: '移动端已选择文件，等待执行上传确认。',
    status: '已选择文件，待上传',
    title: '文件已就绪',
  },
  { desc: '扫码端文件已提交，页面可关闭或重新发起会话。', status: '上传完成', title: '上传已完成' },
] as const

const classId = computed(() => {
  const raw = route.query.id
  return Array.isArray(raw) ? (raw[0] ?? '') : (typeof raw === 'string' ? raw : '')
})
const rawToken = computed(() => {
  const raw = route.query.token
  return Array.isArray(raw) ? (raw[0] ?? '') : (typeof raw === 'string' ? raw : '')
})
const entryTimestamp = computed(() => {
  const raw = route.query.t
  const value = Array.isArray(raw) ? raw[0] : raw
  const parsed = Number(value)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0
})

const hasRouteContext = computed(
  () => Boolean(classId.value || rawToken.value || entryTimestamp.value),
)
const derivedSessionId = computed(() =>
  entryTimestamp.value ? `scan-${classId.value || 'default'}-${entryTimestamp.value}` : '',
)
const classLabel = computed(() => classId.value || '未指定')
const maskedToken = computed(() => {
  if (!rawToken.value) return '未携带'
  if (rawToken.value.length <= 8) return rawToken.value
  return `${rawToken.value.slice(0, 4)}****${rawToken.value.slice(-4)}`
})
const entryUrl = computed(() =>
  typeof window === 'undefined'
    ? route.fullPath
    : `${window.location.origin}${route.fullPath}`,
)
const issuedAtLabel = computed(() => formatDate(entryTimestamp.value))
const expiresAtLabel = computed(() =>
  formatDate(entryTimestamp.value ? entryTimestamp.value + 10 * 60 * 1000 : 0),
)
const totalSizeLabel = computed(() => {
  const total = files.value.reduce((sum, item) => sum + item.file.size, 0)
  if (!total) return '0 KB'
  return total > 1024 * 1024
    ? `${(total / 1024 / 1024).toFixed(1)} MB`
    : `${Math.ceil(total / 1024)} KB`
})
const currentStepIndex = computed(() => {
  const index = steps.findIndex((step) => step.status === session.value.status)
  return Math.max(index, 0)
})
const routeAdvice = computed(() => {
  if (!hasRouteContext.value) return '当前不是从扫码入口进入，建议使用真实扫码链接验收。'
  if (!rawToken.value) return '入口未携带 token，建议确认移动端生成链路。'
  return '入口参数完整，可继续验证扫码与上传流程。'
})
const sessionAdvice = computed(() => {
  if (session.value.status === '上传完成') return '当前会话已完成，可复核产物或重置会话。'
  if (session.value.status === '已选择文件，待上传') return '文件已准备就绪，建议继续确认上传。'
  return '当前会话仍在流程中，可继续推进下一步。'
})

const uploadFileList = computed<UploadFile[]>(() =>
  files.value.map((item) => ({
    name: item.name,
    size: item.file.size,
    status: 'done',
    thumbUrl: item.previewUrl,
    uid: item.id,
    url: item.previewUrl,
  })),
)
const stepItems = computed<StepProps[]>(() =>
  steps.map((step) => ({
    description: step.desc,
    status: currentStepIndex.value > steps.findIndex((item) => item.status === step.status)
      ? 'finish'
      : (currentStepIndex.value === steps.findIndex((item) => item.status === step.status)
        ? 'process'
        : 'wait'),
    title: step.title,
  })),
)
const suggestions = computed(() => [
  routeAdvice.value,
  sessionAdvice.value,
  files.value.length > 0
    ? `当前已选择 ${files.value.length} 个文件，总大小 ${totalSizeLabel.value}。`
    : '当前尚未选择文件，可通过右侧 Upload 组件补齐演示数据。',
])

async function loadSession() {
  busy.value = true
  error.value = ''
  try {
    const current = await getScanSessionApi()
    if (derivedSessionId.value) {
      session.value = current.sessionId === derivedSessionId.value ? current : (await saveScanSessionApi({
          sessionId: derivedSessionId.value,
          status: '待扫码',
        }));
    } else {
      session.value = current
    }
    notice.value = '已同步扫码会话状态。'
  } catch (error_) {
    if (isUnauthorizedError(error_)) {
      session.value = {
        sessionId: derivedSessionId.value || 'public-scan-preview',
        status: hasRouteContext.value ? '待扫码' : '待扫码',
      }
      notice.value = hasRouteContext.value
        ? '当前为公开扫码预览模式；登录后可同步真实会话状态。'
        : '当前为公开预览模式；请通过真实扫码链接继续验收。'
      error.value = ''
    } else {
      error.value = getReadableError(error_)
    }
  } finally {
    busy.value = false
  }
}

async function syncStatus(status: string) {
  const sessionId = session.value.sessionId || derivedSessionId.value
  if (!sessionId) {
    await loadSession()
  }

  busy.value = true
  error.value = ''
  try {
    session.value = await saveScanSessionApi({
      sessionId: session.value.sessionId || derivedSessionId.value,
      status,
    })
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : '更新扫码会话失败'
  } finally {
    busy.value = false
  }
}

async function markScanned() {
  await syncStatus('已扫码，待选择文件')
  notice.value = '扫码已确认，可继续选择上传图片。'
}

async function markReadyToUpload() {
  if (files.value.length === 0) {
    error.value = '请先选择至少一张图片'
    return
  }
  await syncStatus('已选择文件，待上传')
  notice.value = `已登记 ${files.value.length} 个待上传文件。`
}

async function finishUpload() {
  if (files.value.length === 0) {
    error.value = '没有可上传的图片'
    return
  }
  await syncStatus('上传完成')
  notice.value = `扫码会话 ${session.value.sessionId} 已完成上传。`
}

async function resetSession() {
  files.value.forEach((item) => URL.revokeObjectURL(item.previewUrl))
  files.value = []
  await syncStatus('待扫码')
  notice.value = '会话已重置，可重新扫码并发起上传。'
}

async function copyEntryUrl() {
  await copyText(entryUrl.value, '扫码入口链接已复制到剪贴板。')
}

async function copySessionSummary() {
  const text = [
    `会话：${session.value.sessionId || '-'}`,
    `阶段：${session.value.status}`,
    `目标：${classLabel.value}`,
    `入口：${entryUrl.value}`,
    `文件数：${files.value.length}`,
  ].join('\n')
  await copyText(text, '扫码会话摘要已复制。')
}

function beforeSelectImage(file: FileType) {
  if (!file.type.startsWith('image/')) {
    error.value = `${file.name} 不是图片文件，已跳过。`
    return LIST_IGNORE
  }
  if (file.size > 8 * 1024 * 1024) {
    error.value = `${file.name} 超过 8MB，已跳过。`
    return LIST_IGNORE
  }

  const id = `${file.name}-${file.size}-${file.lastModified}`
  if (!files.value.some((item) => item.id === id)) {
    files.value = [
      ...files.value,
      {
        file,
        id,
        name: file.name,
        previewUrl: URL.createObjectURL(file),
        sizeLabel:
          file.size > 1024 * 1024
            ? `${(file.size / 1024 / 1024).toFixed(1)} MB`
            : `${Math.ceil(file.size / 1024)} KB`,
      },
    ]
  }

  error.value = ''
  notice.value = `已选择 ${files.value.length} 个待上传文件。`
  void updateSessionAfterPick()
  return LIST_IGNORE
}

async function updateSessionAfterPick() {
  if (files.value.length === 0) return
  if (session.value.status === '待扫码') {
    await markScanned()
  }
  await markReadyToUpload()
}

function handleRemoveFile(file: UploadFile) {
  removeFile(file.uid)
  return true
}

function fileAdvice(file: ScanDraftFile) {
  return file.file.size > 1024 * 1024
    ? '体积较大，建议优先确认网络环境。'
    : '可直接进入上传确认。'
}

function removeFile(id: string) {
  const hit = files.value.find((item) => item.id === id)
  if (hit) {
    URL.revokeObjectURL(hit.previewUrl)
  }
  files.value = files.value.filter((item) => item.id !== id)
  notice.value = files.value.length > 0
    ? `已保留 ${files.value.length} 个待上传文件。`
    : '待上传列表已清空。'
  if (files.value.length === 0 && session.value.status !== '上传完成') {
    void syncStatus('已扫码，待选择文件')
  }
}

async function copyText(text: string, success: string) {
  try {
    await navigator.clipboard.writeText(text)
    notice.value = success
  } catch {
    window.prompt('当前环境不支持自动复制，请手动复制：', text)
    notice.value = '已切换为手动复制。'
  }
}

function formatDate(timestamp: number) {
  if (!timestamp) return '未提供'
  return new Date(timestamp).toLocaleString()
}

onMounted(() => {
  void loadSession()
})

onBeforeUnmount(() => {
  files.value.forEach((item) => URL.revokeObjectURL(item.previewUrl))
})
</script>

<template>
  <Page
    title="扫码上传会话工作台"
  >
    <div class="space-y-4 p-1">
      <Card>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-base font-medium">
              <ScanIcon class="text-[18px] text-primary" />
              <span>扫码入口与会话控制</span>
            </div>
            <p class="mb-0 text-text-secondary">
              以扫码落地页视角呈现会话状态、入口上下文、文件预览与移动端操作提示，而不是简单切换文案。
            </p>
          </div>
          <Space wrap>
            <Button @click="copyEntryUrl">
              <template #icon>
                <CopyIcon />
              </template>
              复制入口链接
            </Button>
            <Button :disabled="busy" @click="copySessionSummary">
              <template #icon>
                <CopyIcon />
              </template>
              复制会话摘要
            </Button>
            <Button :loading="busy" type="primary" @click="loadSession">
              <template #icon>
                <RefreshIcon />
              </template>
              刷新会话
            </Button>
          </Space>
        </div>
      </Card>

      <Alert v-if="error" :message="error" show-icon type="error" />
      <Alert v-else-if="notice" :message="notice" show-icon type="success" />
      <Alert :message="routeAdvice" banner show-icon type="info" />
      <Row :gutter="[16, 16]">
        <Col :lg="14" :span="24">
          <div class="space-y-4">
            <Card title="会话上下文">
              <Descriptions :column="1" size="small">
                <Descriptions.Item label="入口链接">
                  <span class="break-all">{{ entryUrl }}</span>
                </Descriptions.Item>
                <Descriptions.Item label="访问令牌">{{ maskedToken }}</Descriptions.Item>
                <Descriptions.Item label="会话时间戳">{{ issuedAtLabel }}</Descriptions.Item>
                <Descriptions.Item label="建议过期时间">{{ expiresAtLabel }}</Descriptions.Item>
                <Descriptions.Item label="入口状态">
                  {{ hasRouteContext ? '来自扫码链接' : '直接打开页面' }}
                </Descriptions.Item>
                <Descriptions.Item label="入口风险">{{ routeAdvice }}</Descriptions.Item>
              </Descriptions>
            </Card>

            <Card title="流程阶段">
              <Steps :current="currentStepIndex" :items="stepItems" direction="vertical" size="small" />
              <div class="mt-4">
                <Space wrap>
                  <Button :disabled="busy" @click="markScanned">
                    <template #icon>
                      <ScanIcon />
                    </template>
                    确认已扫码
                  </Button>
                  <Button :disabled="busy || files.length === 0" @click="markReadyToUpload">
                    <template #icon>
                      <UploadIcon />
                    </template>
                    标记已选文件
                  </Button>
                  <Button
                    :disabled="busy || files.length === 0 || session.status === '上传完成'"
                    type="primary"
                    @click="finishUpload"
                  >
                    <template #icon>
                      <ConfirmIcon />
                    </template>
                    完成上传
                  </Button>
                  <Button :disabled="busy" @click="resetSession">
                    <template #icon>
                      <ResetIcon />
                    </template>
                    重置会话
                  </Button>
                </Space>
              </div>
            </Card>
          </div>
        </Col>

        <Col :lg="10" :span="24">
          <div class="space-y-4">
            <Card title="扫码端上传区">
              <div class="mb-3 text-sm text-text-secondary">
                选择图片后会将会话推进到“待上传”，完成后写回扫码会话状态。
              </div>
              <Upload
                :before-upload="beforeSelectImage"
                :file-list="uploadFileList"
                :multiple="true"
                list-type="picture-card"
                @remove="handleRemoveFile"
              >
                <div class="flex flex-col items-center justify-center gap-2">
                  <UploadIcon class="text-[22px] text-primary" />
                  <span class="text-sm">选择图片</span>
                </div>
              </Upload>
              <Space class="mt-3" wrap>
                <Tag>仅支持图片</Tag>
                <Tag>单文件 ≤ 8MB</Tag>
                <Tag>累计 {{ totalSizeLabel }}</Tag>
              </Space>
            </Card>

            <Card title="预览与建议区">
              <List :data-source="files" size="small">
                <template #renderItem="{ item }">
                  <List.Item>
                    <List.Item.Meta :description="fileAdvice(item)" :title="item.name" />
                    <template #extra>
                      <Tag>{{ item.sizeLabel }}</Tag>
                    </template>
                  </List.Item>
                </template>
              </List>
              <List :data-source="suggestions" class="mt-4" size="small">
                <template #renderItem="{ item }">
                  <List.Item>{{ item }}</List.Item>
                </template>
              </List>
            </Card>
          </div>
        </Col>
      </Row>
    </div>
  </Page>
</template>
