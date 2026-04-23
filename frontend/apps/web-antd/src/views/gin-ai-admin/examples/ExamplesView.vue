<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue'

import { onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'

import { Page } from '@vben/common-ui'
import { createIconifyIcon } from '@vben/icons'

import {
  Alert,
  Button,
  Card,
  Col,
  List,
  Row,
  Space,
  Table,
  Tag,
} from 'ant-design-vue'

import {
  getCustomerListApi,
  getResumeUploadListApi,
  getScanSessionApi,
  getUploadQueueApi,
} from '#/api/gin-ai-admin/admin'

interface ExampleEntry {
  desc: string
  health: string
  name: string
  path: string
}

interface CheckItem {
  detail: string
  path?: string
  title: string
}

const RefreshIcon = createIconifyIcon('mdi:refresh')
const CompassIcon = createIconifyIcon('mdi:compass-outline')

const loading = ref(false)
const notice = ref('')
const error = ref('')

const summary = reactive({
  customers: 0,
  resumeInFlight: 0,
  scanAdvice: '待拉取会话状态',
  scanStatus: '-',
  uploadPending: 0,
})

const rows = ref<ExampleEntry[]>([])
const checks = ref<CheckItem[]>([])

const tableColumns: TableColumnsType<ExampleEntry> = [
  {
    dataIndex: 'name',
    key: 'name',
    title: '示例',
  },
  {
    dataIndex: 'desc',
    key: 'desc',
    title: '状态摘要',
  },
  {
    dataIndex: 'health',
    key: 'health',
    title: '健康度',
    width: 140,
  },
  {
    key: 'action',
    title: '入口',
    width: 120,
  },
]

function healthColor(health: string) {
  if (health.includes('已接后端') || health.includes('队列平稳') || health.includes('已收敛')) {
    return 'success'
  }
  if (health.includes('有任务') || health.includes('可联调') || health.includes('需跟进')) {
    return 'processing'
  }
  return 'warning'
}

async function reload() {
  loading.value = true
  error.value = ''
  try {
    const [customers, uploadQueue, resumeList, scanSession] = await Promise.all([
      getCustomerListApi(),
      getUploadQueueApi(),
      getResumeUploadListApi(),
      getScanSessionApi(),
    ])

    summary.customers = customers.Total
    summary.uploadPending = uploadQueue.List.filter(
      (item) => item.status !== 'done' && item.status !== '上传完成',
    ).length
    summary.resumeInFlight = resumeList.List.filter((item) => item.progress < 100).length
    summary.scanStatus = scanSession.status
    summary.scanAdvice =
      scanSession.status === 'active'
        ? '扫码链路可继续验证'
        : scanSession.status === 'completed'
          ? '最近会话已完成'
          : '建议重新打开扫码上传页'

    rows.value = [
      {
        desc: `客户台账 ${customers.Total} 条，支持详情、编辑与筛选`,
        health: customers.Total > 0 ? '已接后端' : '待补数据',
        name: '客户示例',
        path: '/examples/customer',
      },
      {
        desc: `待处理文件 ${summary.uploadPending} 个`,
        health: summary.uploadPending > 0 ? '有任务' : '队列平稳',
        name: '上传示例',
        path: '/examples/upload',
      },
      {
        desc: `进行中 ${summary.resumeInFlight} 项`,
        health: summary.resumeInFlight > 0 ? '需跟进' : '已收敛',
        name: '断点续传',
        path: '/examples/breakpoint',
      },
      {
        desc: `最近状态：${scanSession.status}`,
        health: scanSession.status === 'active' ? '可联调' : '待复查',
        name: '扫码上传',
        path: '/scan-upload',
      },
    ]

    checks.value = [
      {
        detail: customers.Total
          ? '客户示例已接后端，可继续验证增删改查。'
          : '客户示例当前无数据，建议先补一条测试数据。',
        path: '/examples/customer',
        title: '客户链路',
      },
      {
        detail: summary.uploadPending
          ? `当前仍有 ${summary.uploadPending} 个文件待处理，建议进入上传页确认完成状态。`
          : '上传队列已清空，可直接发起新一轮验收。',
        path: '/examples/upload',
        title: '上传队列',
      },
      {
        detail: summary.resumeInFlight
          ? `还有 ${summary.resumeInFlight} 个断点任务未收敛，建议继续推进或恢复。`
          : '续传任务当前已全部完成。',
        path: '/examples/breakpoint',
        title: '续传任务',
      },
      {
        detail: `当前扫码会话为 ${scanSession.status}，${summary.scanAdvice}。`,
        path: '/scan-upload',
        title: '扫码状态',
      },
    ]

    notice.value = '已刷新示例链路概览。'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取示例中心状态失败'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void reload()
})
</script>

<template>
  <Page
    title="Gin AI Admin 示例中心"
  >
    <div class="space-y-4 p-1">
      <Card>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-base font-medium">
              <CompassIcon class="text-[18px] text-primary" />
              <span>统一演示骨架</span>
            </div>
            <p class="mb-0 text-text-secondary">
              当前页负责汇总真实后端状态，并把各示例页收敛为统一的“页头 + 摘要卡 + 主操作区 + 数据区 + 建议区”演示入口。
            </p>
          </div>
          <Space wrap>
            <Button :loading="loading" type="primary" @click="reload">
              <template #icon>
                <RefreshIcon />
              </template>
              刷新概览
            </Button>
          </Space>
        </div>
      </Card>

      <Alert
        v-if="error"
        :message="error"
        show-icon
        type="error"
      />
      <Alert
        v-else-if="notice"
        :message="notice"
        show-icon
        type="success"
      />
      <Row :gutter="[16, 16]">
        <Col :lg="15" :span="24">
          <Card title="示例入口">
            <Table :columns="tableColumns" :data-source="rows" :pagination="false" row-key="path">
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'health'">
                  <Tag :color="healthColor(record.health)">{{ record.health }}</Tag>
                </template>
                <template v-else-if="column.key === 'action'">
                  <RouterLink :to="record.path">
                    <Button size="small" type="link">打开页面</Button>
                  </RouterLink>
                </template>
              </template>
            </Table>
          </Card>
        </Col>

        <Col :lg="9" :span="24">
          <Card title="建议区">
            <List :data-source="checks" item-layout="vertical">
              <template #renderItem="{ item }">
                <List.Item>
                  <template #actions>
                    <RouterLink v-if="item.path" :to="item.path">
                      <Button size="small" type="link">立即查看</Button>
                    </RouterLink>
                  </template>
                  <List.Item.Meta :description="item.detail" :title="item.title" />
                </List.Item>
              </template>
            </List>
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>
