<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { ApiInfo } from '#/types/pop-tail';

import { computed, nextTick, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteApiApi,
  getApiListApi,
  saveApiApi,
} from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface ApiRow extends ApiInfo {
  id: number;
}

const methodOptions = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'].map((value) => ({
  label: value,
  value,
}));

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'path', label: '路径' },
  { component: 'Input', fieldName: 'apiGroup', label: '分组' },
  { component: 'Input', fieldName: 'description', label: '描述' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: methodOptions,
    },
    fieldName: 'method',
    label: '方法',
  },
];

const drawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'path', label: '接口路径', rules: 'required' },
  { component: 'Input', fieldName: 'apiGroup', label: '接口分组', rules: 'required' },
  { component: 'Input', fieldName: 'description', label: '接口描述', rules: 'required' },
  {
    component: 'Select',
    componentProps: {
      options: methodOptions,
    },
    fieldName: 'method',
    label: '请求方法',
    rules: 'required',
  },
];

const editingRow = ref<ApiRow>();
const { canUse } = useMenuButtonAccess();

const [DrawerForm, drawerFormApi] = useVbenForm({
  schema: drawerSchema,
  showDefaultActions: false,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    const { valid } = await drawerFormApi.validate();
    if (!valid) return;
    drawerApi.lock();
    try {
      const values = await drawerFormApi.getValues<{
        apiGroup: string;
        description: string;
        method: string;
        path: string;
      }>();
      await saveApiApi({
        ID: editingRow.value?.ID,
        apiGroup: values.apiGroup.trim(),
        description: values.description.trim(),
        method: values.method,
        path: values.path.trim(),
      });
      message.success(editingRow.value ? '接口已更新' : '接口已创建');
      drawerApi.close();
      onRefresh();
    } finally {
      drawerApi.unlock();
    }
  },
  async onOpenChange(open) {
    if (!open) return;
    drawerFormApi.resetForm();
    await nextTick();
    drawerFormApi.setValues({
      apiGroup: editingRow.value?.apiGroup ?? '',
      description: editingRow.value?.description ?? '',
      method: editingRow.value?.method ?? 'POST',
      path: editingRow.value?.path ?? '',
    });
  },
});

const drawerTitle = computed(() => (editingRow.value ? '编辑接口' : '新增接口'));
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<ApiRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'path', slots: { default: 'path' }, minWidth: 260, title: '路径' },
      { field: 'apiGroup', slots: { default: 'apiGroup' }, title: '分组', width: 160 },
      { field: 'description', slots: { default: 'description' }, minWidth: 240, title: '描述' },
      { field: 'method', slots: { default: 'method' }, title: '方法', width: 110 },
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
            const result = await getApiListApi({
              page: page.currentPage,
              pageSize: page.pageSize,
              ...formValues,
            });
            return {
              items: (result.List ?? []).map((item) => ({ ...item, id: item.ID })),
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
  } as VxeTableGridOptions<ApiRow>,
});

function methodColor(method: string) {
  if (method === 'GET') return 'success';
  if (method === 'DELETE') return 'error';
  if (method === 'PUT' || method === 'PATCH') return 'warning';
  return 'processing';
}

function onCreate() {
  editingRow.value = undefined;
  drawerApi.open();
}

function onEdit(row: ApiRow) {
  editingRow.value = row;
  drawerApi.open();
}

function onDelete(row: ApiRow) {
  Modal.confirm({
    content: `确认删除接口「${row.path}」吗？`,
    onOk: async () => {
      await deleteApiApi({ ID: row.ID });
      message.success('接口已删除');
      onRefresh();
    },
    title: '删除接口',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="接口列表">
      <template #toolbar-tools>
        <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增接口
        </Button>
      </template>
      <template #path="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="72%" />
        <span v-else>{{ row.path }}</span>
      </template>
      <template #apiGroup="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="54%" />
        <span v-else>{{ row.apiGroup }}</span>
      </template>
      <template #description="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="78%" />
        <span v-else>{{ row.description }}</span>
      </template>
      <template #method="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
        <Tag v-else :color="methodColor(row.method)">{{ row.method }}</Tag>
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
        <Space v-else size="small">
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="onEdit(row as ApiRow)">
            编辑
          </Button>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="onDelete(row as ApiRow)">
            删除
          </Button>
        </Space>
      </template>
      </Grid>
    </div>
    <Drawer class="w-full max-w-160" :title="drawerTitle">
      <DrawerForm class="mx-4" layout="vertical" />
    </Drawer>
  </Page>
</template>
