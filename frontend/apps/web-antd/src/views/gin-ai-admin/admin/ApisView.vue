<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { ApiInfo } from '#/types/gin-ai-admin';

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
} from '#/api/gin-ai-admin/admin';

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

const [Grid, gridApi] = useVbenVxeGrid<ApiRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'path', minWidth: 260, title: '路径' },
      { field: 'apiGroup', title: '分组', width: 160 },
      { field: 'description', minWidth: 240, title: '描述' },
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
          const result = await getApiListApi({
            page: page.currentPage,
            pageSize: page.pageSize,
            ...formValues,
          });
          return {
            items: (result.List ?? []).map((item) => ({ ...item, id: item.ID })),
            total: result.Total,
          };
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

function onRefresh() {
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="接口列表">
      <template #toolbar-tools>
        <Button type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增接口
        </Button>
      </template>
      <template #method="{ row }">
        <Tag :color="methodColor(row.method)">{{ row.method }}</Tag>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button size="small" type="link" @click="onEdit(row as ApiRow)">
            编辑
          </Button>
          <Button danger size="small" type="link" @click="onDelete(row as ApiRow)">
            删除
          </Button>
        </Space>
      </template>
    </Grid>
    <Drawer class="w-full max-w-160" :title="drawerTitle">
      <DrawerForm class="mx-4" layout="vertical" />
    </Drawer>
  </Page>
</template>
