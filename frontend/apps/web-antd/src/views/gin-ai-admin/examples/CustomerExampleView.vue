<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue'

import { computed, onMounted, reactive, ref } from 'vue'

import { Page } from '@vben/common-ui'
import { createIconifyIcon } from '@vben/icons'

import {
  Alert,
  Button,
  Card,
  Col,
  Descriptions,
  Empty,
  Form,
  Input,
  List,
  Popconfirm,
  Row,
  Select,
  Space,
  Table,
  Tag,
} from 'ant-design-vue'

import {
  createCustomerApi,
  deleteCustomerApi,
  getCustomerDetailApi,
  getCustomerListApi,
  updateCustomerApi,
} from '#/api/gin-ai-admin/admin'
import type { CustomerRecord } from '#/types/gin-ai-admin'

const SearchIcon = createIconifyIcon('mdi:magnify')
const AddIcon = createIconifyIcon('mdi:account-plus-outline')
const ResetIcon = createIconifyIcon('mdi:backup-restore')
const CustomerIcon = createIconifyIcon('mdi:account-box-multiple-outline')
const DetailIcon = createIconifyIcon('mdi:card-account-details-outline')
const EditIcon = createIconifyIcon('mdi:square-edit-outline')

const rows = ref<CustomerRecord[]>([])
const selected = ref<CustomerRecord | null>(null)
const keyword = ref('')
const notice = ref('')
const error = ref('')
const loading = ref(false)
const submitting = ref(false)
const mode = ref<'create' | 'edit'>('create')
const draft = reactive({
  ID: 0,
  customerLevel: 'A',
  customerName: '',
  customerPhoneData: '',
  customerStatus: '跟进中',
  remark: '',
})

const levelOptions = [
  { label: 'A', value: 'A' },
  { label: 'B', value: 'B' },
  { label: 'C', value: 'C' },
]

const statusOptions = [
  { label: '跟进中', value: '跟进中' },
  { label: '已签约', value: '已签约' },
  { label: '暂停中', value: '暂停中' },
]

const suggestions = computed(() => {
  const items = [
    rows.value.length
      ? '客户链路已接后端，建议从列表选择一条数据验证详情和编辑联动。'
      : '当前暂无客户记录，建议先创建一条演示数据。',
    selected.value
      ? `已选中 ${selected.value.customerName}，可继续核对更新时间、备注与归属用户。`
      : '右侧详情区会展示当前选中客户的完整信息。',
    mode.value === 'edit'
      ? '当前为编辑模式，保存后会回写详情区并刷新列表。'
      : '当前为新增模式，创建后会自动刷新台账并聚焦新数据。',
  ]

  if (keyword.value.trim()) {
    items.push(`当前检索关键字为“${keyword.value.trim()}”，可继续缩小结果范围。`)
  }
  return items
})

const tableColumns: TableColumnsType<CustomerRecord> = [
  {
    dataIndex: 'CreatedAt',
    key: 'CreatedAt',
    title: '接入时间',
    width: 180,
  },
  {
    dataIndex: 'customerName',
    key: 'customerName',
    title: '客户',
    width: 160,
  },
  {
    dataIndex: 'customerPhoneData',
    key: 'customerPhoneData',
    title: '电话',
    width: 160,
  },
  {
    dataIndex: 'customerLevel',
    key: 'customerLevel',
    title: '等级',
    width: 90,
  },
  {
    dataIndex: 'customerStatus',
    key: 'customerStatus',
    title: '状态',
    width: 120,
  },
  {
    dataIndex: 'sysUserId',
    key: 'sysUserId',
    title: '归属用户',
    width: 110,
  },
  {
    key: 'action',
    title: '操作',
    width: 220,
    fixed: 'right',
  },
]

function statusColor(status: string) {
  if (status === '已签约') return 'success'
  if (status === '跟进中') return 'processing'
  return 'warning'
}

function getRowClass(record: CustomerRecord) {
  return record.ID === selected.value?.ID ? 'is-active-row' : ''
}

function formatTime(value: number) {
  return new Date(value).toLocaleString('zh-CN')
}

async function loadCustomers() {
  loading.value = true
  error.value = ''
  try {
    const result = await getCustomerListApi({ keyword: keyword.value.trim() })
    rows.value = result.List

    if (selected.value) {
      const next = rows.value.find((item) => item.ID === selected.value?.ID)
      if (next) {
        selected.value = next
      } else if (rows.value[0]) {
        await openDetail(rows.value[0].ID)
      } else {
        selected.value = null
      }
    }

    notice.value = rows.value.length ? '客户台账已刷新。' : '当前没有匹配的客户记录。'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取客户列表失败'
  } finally {
    loading.value = false
  }
}

async function openDetail(id: number) {
  loading.value = true
  error.value = ''
  try {
    selected.value = await getCustomerDetailApi(id)
    notice.value = `已打开客户 ${selected.value.customerName} 的详情。`
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取客户详情失败'
  } finally {
    loading.value = false
  }
}

function startCreate() {
  mode.value = 'create'
  resetDraft()
  notice.value = '已切换到新增客户模式。'
}

function startEdit(item: CustomerRecord | Record<string, any>) {
  mode.value = 'edit'
  draft.ID = item.ID
  draft.customerName = item.customerName
  draft.customerPhoneData = item.customerPhoneData
  draft.customerLevel = item.customerLevel
  draft.customerStatus = item.customerStatus
  draft.remark = item.remark
  notice.value = `已载入客户 ${item.customerName} 的编辑草稿。`
}

async function submit() {
  if (!draft.customerName.trim() || !draft.customerPhoneData.trim()) {
    error.value = '请填写客户名称和电话'
    return
  }

  submitting.value = true
  error.value = ''

  const payload = {
    customerLevel: draft.customerLevel,
    customerName: draft.customerName.trim(),
    customerPhoneData: draft.customerPhoneData.trim(),
    customerStatus: draft.customerStatus,
    remark: draft.remark.trim(),
  }

  try {
    if (mode.value === 'edit' && draft.ID) {
      const saved = await updateCustomerApi({ ID: draft.ID, ...payload })
      selected.value = saved
      notice.value = `客户 ${saved.customerName} 已更新。`
    } else {
      const created = await createCustomerApi(payload)
      selected.value = created
      notice.value = `客户 ${created.customerName} 已创建。`
    }
    await loadCustomers()
    resetDraft()
    mode.value = 'create'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存客户失败'
  } finally {
    submitting.value = false
  }
}

async function remove(id: number) {
  submitting.value = true
  error.value = ''
  try {
    await deleteCustomerApi(id)
    if (selected.value?.ID === id) {
      selected.value = null
    }
    notice.value = `客户 #${id} 已删除。`
    await loadCustomers()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '删除客户失败'
  } finally {
    submitting.value = false
  }
}

function resetDraft() {
  draft.ID = 0
  draft.customerName = ''
  draft.customerPhoneData = ''
  draft.customerLevel = 'A'
  draft.customerStatus = '跟进中'
  draft.remark = ''
}

onMounted(async () => {
  await loadCustomers()
  if (rows.value[0]) {
    await openDetail(rows.value[0].ID)
  }
})
</script>

<template>
  <Page
    title="客户示例演示页"
  >
    <div class="space-y-4 p-1">
      <Card>
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="space-y-2">
            <div class="flex items-center gap-2 text-base font-medium">
              <CustomerIcon class="text-[18px] text-primary" />
              <span>页头 + 摘要卡 + 主操作区</span>
            </div>
            <p class="mb-0 text-text-secondary">
              当前页保留“客户台账 + 单条详情 + 编辑入口”的联动逻辑，并统一为官方 Vben 风格的演示页骨架。
            </p>
          </div>
          <Space wrap>
            <Button :loading="loading" @click="loadCustomers">
              <template #icon>
                <SearchIcon />
              </template>
              查询
            </Button>
            <Button type="primary" @click="startCreate">
              <template #icon>
                <AddIcon />
              </template>
              新增客户
            </Button>
          </Space>
        </div>

        <Form class="mt-4" layout="vertical">
          <Row :gutter="16">
            <Col :lg="12" :span="24">
              <Form.Item label="客户名称 / 状态 / 等级">
                <Input
                  v-model:value="keyword"
                  allow-clear
                  placeholder="输入客户名称、状态或等级"
                  @press-enter="loadCustomers"
                />
              </Form.Item>
            </Col>
            <Col :lg="12" :span="24">
              <Form.Item label="说明">
                <Input disabled value="查询区与详情、编辑区联动，便于展示完整 CRUD 示例链路。" />
              </Form.Item>
            </Col>
          </Row>
          <Space wrap>
            <Button :loading="loading" type="primary" @click="loadCustomers">
              <template #icon>
                <SearchIcon />
              </template>
              重新查询
            </Button>
            <Button @click="keyword = ''; loadCustomers()">
              <template #icon>
                <ResetIcon />
              </template>
              清空条件
            </Button>
          </Space>
        </Form>
      </Card>

      <Alert v-if="error" :message="error" show-icon type="error" />
      <Alert v-else-if="notice" :message="notice" show-icon type="success" />

      <Row :gutter="[16, 16]">
        <Col :lg="15" :span="24">
          <Card title="客户台账">
            <Table
              :columns="tableColumns"
              :data-source="rows"
              :loading="loading"
              :pagination="{ pageSize: 8 }"
              :row-class-name="getRowClass"
              row-key="ID"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'CreatedAt'">
                  {{ formatTime(record.CreatedAt) }}
                </template>
                <template v-else-if="column.key === 'customerLevel'">
                  <Tag color="blue">{{ record.customerLevel }}</Tag>
                </template>
                <template v-else-if="column.key === 'customerStatus'">
                  <Tag :color="statusColor(record.customerStatus)">{{ record.customerStatus }}</Tag>
                </template>
                <template v-else-if="column.key === 'action'">
                  <Space :size="4" wrap>
                    <Button size="small" type="link" @click="openDetail(record.ID)">查看</Button>
                    <Button size="small" type="link" @click="startEdit(record)">编辑</Button>
                    <Popconfirm
                      cancel-text="取消"
                      ok-text="删除"
                      title="确认删除这条客户记录吗？"
                      @confirm="remove(record.ID)"
                    >
                      <Button danger size="small" type="link">删除</Button>
                    </Popconfirm>
                  </Space>
                </template>
              </template>
            </Table>
          </Card>
        </Col>

        <Col :lg="9" :span="24">
          <div class="space-y-4">
            <Card>
              <template #title>
                <div class="flex items-center gap-2">
                  <DetailIcon class="text-[18px] text-primary" />
                  <span>客户详情</span>
                </div>
              </template>

              <Descriptions v-if="selected" :column="1" size="small">
                <Descriptions.Item label="客户名">{{ selected.customerName }}</Descriptions.Item>
                <Descriptions.Item label="电话">{{ selected.customerPhoneData }}</Descriptions.Item>
                <Descriptions.Item label="等级">
                  <Tag color="blue">{{ selected.customerLevel }}</Tag>
                </Descriptions.Item>
                <Descriptions.Item label="状态">
                  <Tag :color="statusColor(selected.customerStatus)">
                    {{ selected.customerStatus }}
                  </Tag>
                </Descriptions.Item>
                <Descriptions.Item label="归属用户">{{ selected.sysUserId }}</Descriptions.Item>
                <Descriptions.Item label="备注">{{ selected.remark || '-' }}</Descriptions.Item>
                <Descriptions.Item label="更新时间">
                  {{ formatTime(selected.UpdatedAt) }}
                </Descriptions.Item>
              </Descriptions>
              <Empty v-else description="请选择一条客户记录查看详情" />
            </Card>

            <Card>
              <template #title>
                <div class="flex items-center gap-2">
                  <EditIcon class="text-[18px] text-primary" />
                  <span>{{ mode === 'edit' ? '编辑客户' : '新增客户' }}</span>
                </div>
              </template>

              <Form layout="vertical">
                <Row :gutter="12">
                  <Col :span="12">
                    <Form.Item label="客户名" required>
                      <Input v-model:value="draft.customerName" placeholder="请输入客户名" />
                    </Form.Item>
                  </Col>
                  <Col :span="12">
                    <Form.Item label="联系电话" required>
                      <Input v-model:value="draft.customerPhoneData" placeholder="请输入联系电话" />
                    </Form.Item>
                  </Col>
                </Row>

                <Row :gutter="12">
                  <Col :span="12">
                    <Form.Item label="等级">
                      <Select v-model:value="draft.customerLevel" :options="levelOptions" />
                    </Form.Item>
                  </Col>
                  <Col :span="12">
                    <Form.Item label="状态">
                      <Select v-model:value="draft.customerStatus" :options="statusOptions" />
                    </Form.Item>
                  </Col>
                </Row>

                <Form.Item label="备注">
                  <Input.TextArea v-model:value="draft.remark" :rows="4" />
                </Form.Item>

                <Space wrap>
                  <Button :loading="submitting" type="primary" @click="submit">
                    {{ mode === 'edit' ? '保存变更' : '创建客户' }}
                  </Button>
                  <Button @click="resetDraft">重置草稿</Button>
                </Space>
              </Form>
            </Card>

            <Card title="建议区">
              <List :data-source="suggestions" size="small">
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

<style scoped>
:deep(.is-active-row > td) {
  background: rgb(24 144 255 / 8%) !important;
}
</style>
