<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { ref } from 'vue';

import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Modal, Select, Space, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteLlmConfigApi, getLlmConfigListApi, saveLlmConfigApi } from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

type LlmProvider = 'claude' | 'openai' | 'qwen';
type LlmConfig = {
  apiKey: string;
  baseUrl: string;
  id: number;
  model: string;
  provider: LlmProvider;
  status: 'disabled' | 'enabled';
};

const providerOptions = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Qwen', value: 'qwen' },
  { label: 'Claude', value: 'claude' },
];

const statusOptions = [
  { label: '启用', value: 'enabled' },
  { label: '禁用', value: 'disabled' },
];

const filterSchema: VbenFormSchema[] = [
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: providerOptions,
    },
    fieldName: 'provider',
    label: '供应商',
  },
  { component: 'Input', fieldName: 'model', label: '模型' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: statusOptions,
    },
    fieldName: 'status',
    label: '状态',
  },
];

const drawerOpen = ref(false);
const editing = ref<LlmConfig | null>(null);
const records = ref<LlmConfig[]>([]);
const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});
const form = ref<LlmConfig>({
  id: 0,
  apiKey: '',
  baseUrl: '',
  model: '',
  provider: 'openai',
  status: 'enabled',
});

const [Grid, gridApi] = useVbenVxeGrid<LlmConfig>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'provider', slots: { default: 'provider' }, title: '供应商', width: 140 },
      { field: 'model', slots: { default: 'model' }, title: '模型', minWidth: 180 },
      { field: 'baseUrl', slots: { default: 'baseUrl' }, title: 'API 地址', minWidth: 260 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 120 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 140,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const data = await getLlmConfigListApi();
            const rows = data.List.map((item) => ({
              apiKey: item.apiKey,
              baseUrl: item.baseUrl,
              id: item.ID,
              model: item.model,
              provider: item.provider as LlmProvider,
              status: item.status as LlmConfig['status'],
            }));
            records.value = rows;
            const provider = String(formValues.provider ?? '').trim();
            const model = String(formValues.model ?? '').trim().toLowerCase();
            const status = String(formValues.status ?? '').trim();
            const filtered = rows.filter((item) =>
              (!provider || item.provider === provider) &&
              (!model || item.model.toLowerCase().includes(model)) &&
              (!status || item.status === status),
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
  } as VxeTableGridOptions<LlmConfig>,
});

function openCreate() {
  editing.value = null;
  form.value = { id: 0, apiKey: '', baseUrl: '', model: '', provider: 'openai', status: 'enabled' };
  drawerOpen.value = true;
}

function openEdit(record: LlmConfig) {
  editing.value = record;
  form.value = { ...record };
  drawerOpen.value = true;
}

async function save() {
  if (!form.value.model.trim() || !form.value.baseUrl.trim()) {
    message.warning('请先补全模型名和地址');
    return;
  }
  await saveLlmConfigApi({
    ID: editing.value?.id,
    apiKey: form.value.apiKey,
    baseUrl: form.value.baseUrl,
    model: form.value.model,
    provider: form.value.provider,
    status: form.value.status,
  });
  drawerOpen.value = false;
  await onRefresh();
  message.success('模型配置已保存');
}

function remove(record: LlmConfig) {
  Modal.confirm({
    content: `确认删除模型「${record.model}」吗？`,
    onOk: async () => {
      await deleteLlmConfigApi(record.id);
      await onRefresh();
      message.success('模型配置已删除');
    },
    title: '删除模型',
  });
}

function statusColor(status: string) {
  return status === 'enabled' ? 'success' : 'default';
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <div class="h-full min-h-0 p-4">
    <div class="llm-config-shell">


      <div class="llm-config-pane__content">
        <Grid class="h-full min-h-0 flex-1" table-title="模型列表">
          <template #toolbar-tools>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="openCreate">
              <IconifyIcon class="size-5" icon="mdi:plus" />
              添加模型
            </Button>
          </template>
          <template #provider="{ row }">
            <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
            <span v-else>{{ row.provider }}</span>
          </template>
          <template #model="{ row }">
            <PageRefreshCellSkeleton v-if="pageLoading" width="66%" />
            <span v-else>{{ row.model }}</span>
          </template>
          <template #baseUrl="{ row }">
            <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
            <span v-else>{{ row.baseUrl }}</span>
          </template>
          <template #status="{ row }">
            <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
            <Tag v-else :color="statusColor(row.status)">{{ row.status }}</Tag>
          </template>
          <template #operation="{ row }">
            <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
            <Space v-else>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link"
                @click="openEdit(row as LlmConfig)">编辑</Button>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link"
                @click="remove(row as LlmConfig)">删除</Button>
            </Space>
          </template>
        </Grid>
      </div>

    </div>

    <Drawer v-model:open="drawerOpen" :title="editing == null ? '新增模型' : '编辑模型'" width="520">
      <Form layout="vertical">
        <Form.Item label="供应商">
          <Select v-model:value="form.provider" :options="providerOptions" />
        </Form.Item>
        <Form.Item label="模型名称">
          <Input v-model:value="form.model" placeholder="例如 gpt-4.1 / qwen-max / claude-sonnet-4" />
        </Form.Item>
        <Form.Item label="API 地址">
          <Input v-model:value="form.baseUrl" placeholder="例如 https://api.openai.com/v1" />
        </Form.Item>
        <Form.Item label="API Key">
          <Input v-model:value="form.apiKey" placeholder="请输入 API Key" />
        </Form.Item>
        <Form.Item label="状态">
          <Select v-model:value="form.status" :options="statusOptions" />
        </Form.Item>
        <div class="flex justify-end">
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" type="primary" @click="save">保存</Button>
        </div>
      </Form>
    </Drawer>
  </div>
</template>

<style scoped>
.llm-config-shell {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  flex-direction: column;
}

.llm-config-tabs {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  flex-direction: column;
}

.llm-config-pane {
  min-height: 0;
  height: 100%;
}

.llm-config-pane__content {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  flex-direction: column;
}

.llm-config-tabs :deep(.ant-tabs-content-holder) {
  display: flex;
  flex: 1;
  min-height: 0;
}

.llm-config-tabs :deep(.ant-tabs-content) {
  min-height: 0;
  height: 100%;
}

.llm-config-tabs :deep(.ant-tabs-tabpane) {
  min-height: 0;
  height: 100%;
}
</style>
