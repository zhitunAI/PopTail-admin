<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { EmailPresetRecord, EmailRecord } from '#/types/pop-tail';

import { computed, nextTick, reactive, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Alert, Button, Form, Input, Modal, Space, Tabs, TabPane, Tag, message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteEmailPresetApi,
  emailTestApi,
  getEmailListApi,
  getEmailPresetListApi,
  saveEmailPresetApi,
  sendEmailApi,
} from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface EmailPresetRow extends EmailPresetRecord {
  id: number;
  updatedAtText: string;
}

interface EmailRecordRow extends EmailRecord {
  createdAtText: string;
  id: number;
}

const activeTab = ref('compose');
const sending = ref(false);
const messageText = ref('');
const errorText = ref('');
const selectedPresetId = ref<null | number>(null);
const presetRows = ref<EmailPresetRow[]>([]);

const form = reactive({
  body: '',
  subject: '',
  to: '',
});

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.to.trim()) issues.push('目标邮箱不能为空。');
  if (form.to && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.to.trim())) issues.push('目标邮箱格式不正确。');
  if (!form.subject.trim()) issues.push('邮件标题不能为空。');
  if (!form.body.trim()) issues.push('邮件内容不能为空。');
  return issues;
});

const filterPresetSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '预设名称' },
];

const filterRecordSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'to', label: '收件人' },
  { component: 'Input', fieldName: 'subject', label: '标题' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '发送', value: 'send' },
        { label: '测试', value: 'test' },
      ],
    },
    fieldName: 'mode',
    label: '模式',
  },
];

const presetDrawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '预设名称', rules: 'required' },
  { component: 'Input', fieldName: 'description', label: '描述' },
  { component: 'Input', fieldName: 'to', label: '默认收件人', rules: 'required' },
  { component: 'Input', fieldName: 'subject', label: '默认标题', rules: 'required' },
  { component: 'Textarea', fieldName: 'body', label: '正文模板', rules: 'required' },
];

const editingPreset = ref<EmailPresetRow>();
useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  if (activeTab.value === 'presets') {
    await presetGridApi.query();
    return;
  }
  if (activeTab.value === 'records') {
    await recordGridApi.query();
  }
});

const [PresetDrawerForm, presetDrawerFormApi] = useVbenForm({
  schema: presetDrawerSchema,
  showDefaultActions: false,
});

const [PresetDrawer, presetDrawerApi] = useVbenDrawer({
  async onConfirm() {
    const { valid } = await presetDrawerFormApi.validate();
    if (!valid) return;
    presetDrawerApi.lock();
    try {
      const values = await presetDrawerFormApi.getValues<{
        body: string;
        description?: string;
        name: string;
        subject: string;
        to: string;
      }>();
      await saveEmailPresetApi({
        ID: editingPreset.value?.ID,
        body: values.body.trim(),
        description: values.description?.trim() || '',
        name: values.name.trim(),
        subject: values.subject.trim(),
        to: values.to.trim(),
      });
      message.success(editingPreset.value ? '邮件预设已更新' : '邮件预设已创建');
      presetDrawerApi.close();
      await presetGridApi.query();
    } finally {
      presetDrawerApi.unlock();
    }
  },
  async onOpenChange(open) {
    if (!open) return;
    presetDrawerFormApi.resetForm();
    await nextTick();
    presetDrawerFormApi.setValues({
      body: editingPreset.value?.body ?? '',
      description: editingPreset.value?.description ?? '',
      name: editingPreset.value?.name ?? '',
      subject: editingPreset.value?.subject ?? '',
      to: editingPreset.value?.to ?? '',
    });
  },
});

const presetDrawerTitle = computed(() => (editingPreset.value ? '编辑邮件预设' : '新增邮件预设'));

const [PresetGrid, presetGridApi] = useVbenVxeGrid<EmailPresetRow>({
  formOptions: {
    collapsed: false,
    schema: filterPresetSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: '预设名称', width: 180 },
      { field: 'description', slots: { default: 'description' }, title: '描述', minWidth: 220 },
      { field: 'to', slots: { default: 'to' }, title: '默认收件人', minWidth: 220 },
      { field: 'subject', slots: { default: 'subject' }, title: '默认标题', minWidth: 220 },
      { field: 'updatedAtText', slots: { default: 'updatedAtText' }, title: '更新时间', width: 180 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 180,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const result = await getEmailPresetListApi({
              name: formValues.name,
              page: page.currentPage,
              pageSize: page.pageSize,
            });
            const rows = (result.List ?? []).map((item) => ({
              ...item,
              id: item.ID,
              updatedAtText: formatTime(item.updatedAt),
            }));
            presetRows.value = rows;
            return {
              items: rows,
              total: result.Total,
            };
          } finally {
            pageLoading.value = false;
          }
        },
      },
    },
    rowConfig: {
      keyField: 'id',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: true,
      search: true,
      zoom: true,
    },
  } as VxeTableGridOptions<EmailPresetRow>,
});

const [RecordGrid, recordGridApi] = useVbenVxeGrid<EmailRecordRow>({
  formOptions: {
    collapsed: false,
    schema: filterRecordSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'createdAtText', slots: { default: 'createdAtText' }, title: '时间', width: 180 },
      { field: 'to', slots: { default: 'to' }, title: '收件人', minWidth: 200 },
      { field: 'subject', slots: { default: 'subject' }, title: '标题', minWidth: 220 },
      { field: 'mode', slots: { default: 'mode' }, title: '模式', width: 100 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 120 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 120,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const result = await getEmailListApi();
            const rows = (result.List ?? []).map((item) => ({
              ...item,
              createdAtText: formatTime(item.createdAt),
              id: item.ID,
            }));
            const to = String(formValues.to ?? '').trim().toLowerCase();
            const subject = String(formValues.subject ?? '').trim().toLowerCase();
            const mode = String(formValues.mode ?? '').trim();
            const filtered = rows.filter((item) =>
              (!to || item.to.toLowerCase().includes(to)) &&
              (!subject || item.subject.toLowerCase().includes(subject)) &&
              (!mode || item.mode === mode),
            );
            const start = (page.currentPage - 1) * page.pageSize;
            return {
              items: filtered.slice(start, start + page.pageSize),
              total: filtered.length,
            };
          } finally {
            pageLoading.value = false;
          }
        },
      },
    },
    rowConfig: {
      keyField: 'id',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: true,
      search: true,
      zoom: true,
    },
  } as VxeTableGridOptions<EmailRecordRow>,
});

function formatTime(value: number) {
  return new Date(value).toLocaleString('zh-CN');
}

function openCreatePreset() {
  editingPreset.value = undefined;
  presetDrawerApi.open();
}

function openEditPreset(row: EmailPresetRow) {
  editingPreset.value = row;
  presetDrawerApi.open();
}

async function removePreset(row: EmailPresetRow) {
  Modal.confirm({
    content: `确认删除邮件预设「${row.name}」吗？`,
    onOk: async () => {
      await deleteEmailPresetApi(row.ID);
      await presetGridApi.query();
      message.success('邮件预设已删除');
    },
    title: '删除预设',
  });
}

function usePreset(row: EmailPresetRow) {
  selectedPresetId.value = row.ID;
  form.to = row.to;
  form.subject = row.subject;
  form.body = row.body;
  activeTab.value = 'compose';
  messageText.value = `已载入预设：${row.name}`;
  errorText.value = '';
}

async function dispatch(mode: 'send' | 'test') {
  if (validationIssues.value.length > 0) {
    errorText.value = validationIssues.value[0] ?? '邮件表单不完整';
    return;
  }
  sending.value = true;
  errorText.value = '';
  try {
    const payload = {
      body: form.body.trim(),
      subject: form.subject.trim(),
      to: form.to.trim(),
    };
    const created = mode === 'test' ? await emailTestApi(payload) : await sendEmailApi(payload);
    messageText.value = `${mode === 'test' ? '测试邮件' : '邮件'}已发送：${created.to}`;
    await recordGridApi.query();
    activeTab.value = 'records';
  } catch (error) {
    errorText.value = error instanceof Error ? error.message : '发送邮件失败';
  } finally {
    sending.value = false;
  }
}

async function copyRecord(row: EmailRecordRow) {
  const text = [
    `收件人：${row.to}`,
    `标题：${row.subject}`,
    `模式：${row.mode}`,
    `状态：${row.status}`,
    `时间：${row.createdAtText}`,
  ].join('\n');
  try {
    await navigator.clipboard.writeText(text);
    message.success('已复制邮件摘要');
  } catch {
    window.prompt('当前环境不支持自动复制，请手动复制：', text);
  }
}

function modeColor(mode: string) {
  return mode === 'test' ? 'processing' : 'success';
}

function statusColor(status: string) {
  return status.includes('失败') ? 'error' : 'success';
}
</script>

<template>
  <Page auto-content-height title="邮件管理">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Alert v-if="messageText" class="mb-4" show-icon type="info" :message="messageText" />
      <Alert v-if="errorText" class="mb-4" show-icon type="error" :message="errorText" />

      <Tabs v-model:activeKey="activeTab" class="flex h-full min-h-0 flex-1 flex-col">
        <TabPane key="compose" tab="立即发送" class="h-full min-h-0">
          <div class="email-compose-card">
            <Form layout="vertical">
              <Form.Item label="目标邮箱">
                <Input v-model:value="form.to" placeholder="ops-team@gaa.local" />
              </Form.Item>
              <Form.Item label="邮件标题">
                <Input v-model:value="form.subject" placeholder="请输入邮件标题" />
              </Form.Item>
              <Form.Item label="邮件正文">
                <Input.TextArea v-model:value="form.body" :rows="12" placeholder="请输入邮件正文" />
              </Form.Item>
              <div v-if="validationIssues.length > 0" class="email-compose-errors">
                <div v-for="item in validationIssues" :key="item">{{ item }}</div>
              </div>
              <div class="flex justify-end gap-3">
                <Button :disabled="sending || validationIssues.length > 0" @click="dispatch('test')">
                  {{ sending ? '发送中...' : '发送测试邮件' }}
                </Button>
                <Button type="primary" :disabled="sending || validationIssues.length > 0" @click="dispatch('send')">
                  {{ sending ? '发送中...' : '立即发送' }}
                </Button>
              </div>
            </Form>
          </div>
        </TabPane>

        <TabPane key="presets" tab="模板预设" class="h-full min-h-0">
          <div class="flex h-full min-h-0 flex-1 flex-col">
            <PresetGrid class="h-full min-h-0 flex-1" table-title="邮件预设列表">
              <template #toolbar-tools>
                <Button type="primary" @click="openCreatePreset">
                  <IconifyIcon class="size-5" icon="mdi:plus" />
                  新增预设
                </Button>
              </template>
              <template #name="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="68%" />
                <span v-else>{{ row.name }}</span>
              </template>
              <template #description="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="84%" />
                <span v-else>{{ row.description || '-' }}</span>
              </template>
              <template #to="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
                <span v-else>{{ row.to }}</span>
              </template>
              <template #subject="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
                <span v-else>{{ row.subject }}</span>
              </template>
              <template #updatedAtText="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
                <span v-else>{{ row.updatedAtText }}</span>
              </template>
              <template #operation="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="160px" />
                <Space v-else>
                  <Button size="small" type="link" @click="usePreset(row as EmailPresetRow)">套用</Button>
                  <Button size="small" type="link" @click="openEditPreset(row as EmailPresetRow)">编辑</Button>
                  <Button danger size="small" type="link" @click="removePreset(row as EmailPresetRow)">删除</Button>
                </Space>
              </template>
            </PresetGrid>
          </div>
        </TabPane>

        <TabPane key="records" tab="邮件记录" class="h-full min-h-0">
          <div class="flex h-full min-h-0 flex-1 flex-col">
            <RecordGrid class="h-full min-h-0 flex-1" table-title="邮件记录">
              <template #createdAtText="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
                <span v-else>{{ row.createdAtText }}</span>
              </template>
              <template #to="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
                <span v-else>{{ row.to }}</span>
              </template>
              <template #subject="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="84%" />
                <span v-else>{{ row.subject }}</span>
              </template>
              <template #mode="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
                <Tag v-else :color="modeColor(row.mode)">{{ row.mode }}</Tag>
              </template>
              <template #status="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
                <Tag v-else :color="statusColor(row.status)">{{ row.status }}</Tag>
              </template>
              <template #operation="{ row }">
                <PageRefreshCellSkeleton v-if="pageLoading" width="80px" />
                <Button v-else size="small" type="link" @click="copyRecord(row as EmailRecordRow)">复制摘要</Button>
              </template>
            </RecordGrid>
          </div>
        </TabPane>
      </Tabs>
    </div>

    <PresetDrawer class="w-full max-w-180" :title="presetDrawerTitle">
      <PresetDrawerForm class="mx-4" layout="vertical" />
    </PresetDrawer>
  </Page>
</template>

<style scoped>
:deep(.ant-form-item-label > label) {
  color: hsl(var(--foreground));
  font-weight: 600;
}

:deep(.ant-input),
:deep(.ant-input-affix-wrapper),
:deep(.ant-input-number),
:deep(.ant-input-number-input),
:deep(.ant-input-outlined),
:deep(.ant-input-textarea),
:deep(.ant-input-textarea textarea) {
  background: hsl(var(--input-background));
  border-color: hsl(var(--border));
  color: hsl(var(--foreground));
}

:deep(.ant-input::placeholder),
:deep(.ant-input-textarea textarea::placeholder) {
  color: hsl(var(--muted-foreground));
}

:deep(.ant-input:hover),
:deep(.ant-input-affix-wrapper:hover),
:deep(.ant-input-number:hover),
:deep(.ant-input-textarea:hover),
:deep(.ant-input-textarea textarea:hover) {
  border-color: hsl(var(--ring) / 0.55);
}

:deep(.ant-input:focus),
:deep(.ant-input-focused),
:deep(.ant-input-affix-wrapper-focused),
:deep(.ant-input-number-focused),
:deep(.ant-input-textarea textarea:focus) {
  border-color: hsl(var(--ring));
  box-shadow: 0 0 0 2px hsl(var(--ring) / 0.18);
}

.email-compose-card {
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  border-radius: calc(var(--radius) + 6px);
  padding: 24px;
}

.email-compose-errors {
  margin-bottom: 16px;
  border: 1px solid hsl(var(--destructive) / 0.28);
  background: hsl(var(--destructive) / 0.08);
  color: hsl(var(--destructive));
  border-radius: calc(var(--radius) - 2px);
  padding: 12px;
  font-size: 14px;
}

:deep(.ant-tabs-content-holder) {
  display: flex;
  flex: 1;
  min-height: 0;
}

:deep(.ant-tabs-content) {
  min-height: 0;
  height: 100%;
}

:deep(.ant-tabs-tabpane) {
  min-height: 0;
  height: 100%;
}
</style>
