<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { ParamInfo } from '#/types/gin-ai-admin';

import { computed, nextTick, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteParamApi,
  getParamsListApi,
  saveParamApi,
} from '#/api/gin-ai-admin/admin';

interface ParamRow extends ParamInfo {
  id: number;
}

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'key', label: '参数键' },
  { component: 'Input', fieldName: 'value', label: '参数值' },
  { component: 'Input', fieldName: 'desc', label: '说明' },
];

const drawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'key', label: '参数键', rules: 'required' },
  { component: 'Input', fieldName: 'value', label: '参数值', rules: 'required' },
  { component: 'Textarea', fieldName: 'desc', label: '说明', rules: 'required' },
];

const editingRow = ref<ParamRow>();

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
        desc: string;
        key: string;
        value: string;
      }>();
      await saveParamApi({
        ID: editingRow.value?.ID,
        desc: values.desc.trim(),
        key: values.key.trim(),
        value: values.value.trim(),
      });
      message.success(editingRow.value ? '参数已更新' : '参数已创建');
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
      desc: editingRow.value?.desc ?? '',
      key: editingRow.value?.key ?? '',
      value: editingRow.value?.value ?? '',
    });
  },
});

const drawerTitle = computed(() => (editingRow.value ? '编辑参数' : '新增参数'));

const [Grid, gridApi] = useVbenVxeGrid<ParamRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'ID', title: 'ID', width: 90 },
      { field: 'key', title: '参数键', width: 220 },
      { field: 'value', title: '参数值', minWidth: 260 },
      { field: 'desc', title: '说明', minWidth: 260 },
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
          const result = await getParamsListApi({
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
  } as VxeTableGridOptions<ParamRow>,
});

function onCreate() {
  editingRow.value = undefined;
  drawerApi.open();
}

function onEdit(row: ParamRow) {
  editingRow.value = row;
  drawerApi.open();
}

function onDelete(row: ParamRow) {
  Modal.confirm({
    content: `确认删除参数「${row.key}」吗？`,
    onOk: async () => {
      await deleteParamApi({ ID: row.ID });
      message.success('参数已删除');
      onRefresh();
    },
    title: '删除参数',
  });
}

function onRefresh() {
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="参数列表">
      <template #toolbar-tools>
        <Button type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增参数
        </Button>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button size="small" type="link" @click="onEdit(row as ParamRow)">
            编辑
          </Button>
          <Button danger size="small" type="link" @click="onDelete(row as ParamRow)">
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
