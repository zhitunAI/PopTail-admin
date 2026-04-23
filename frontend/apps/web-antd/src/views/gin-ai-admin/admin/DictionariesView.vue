<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { DictionaryInfo } from '#/types/gin-ai-admin';

import { computed, nextTick, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteDictionaryApi,
  getDictionaryListApi,
  saveDictionaryApi,
} from '#/api/gin-ai-admin/admin';

interface DictionaryRow extends DictionaryInfo {
  id: number;
}

const router = useRouter();
const editingRow = ref<DictionaryRow>();

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'keyword', label: '关键词' },
  { component: 'Input', fieldName: 'type', label: '类型标识' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '启用', value: true },
        { label: '停用', value: false },
      ],
    },
    fieldName: 'status',
    label: '状态',
  },
];

const drawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '字典名称', rules: 'required' },
  { component: 'Input', fieldName: 'type', label: '类型标识', rules: 'required' },
  { component: 'Textarea', fieldName: 'desc', label: '说明', rules: 'required' },
  {
    component: 'RadioGroup',
    componentProps: {
      buttonStyle: 'solid',
      optionType: 'button',
      options: [
        { label: '启用', value: true },
        { label: '停用', value: false },
      ],
    },
    defaultValue: true,
    fieldName: 'status',
    label: '状态',
  },
];

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
        name: string;
        status: boolean;
        type: string;
      }>();
      await saveDictionaryApi({
        ID: editingRow.value?.ID,
        desc: values.desc.trim(),
        name: values.name.trim(),
        status: Boolean(values.status),
        type: values.type.trim(),
      });
      message.success(editingRow.value ? '字典已更新' : '字典已创建');
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
      name: editingRow.value?.name ?? '',
      status: editingRow.value?.status ?? true,
      type: editingRow.value?.type ?? '',
    });
  },
});

const drawerTitle = computed(() => (editingRow.value ? '编辑字典' : '新增字典'));

const [Grid, gridApi] = useVbenVxeGrid<DictionaryRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', title: '字典名称', width: 180 },
      { field: 'type', title: '类型标识', width: 200 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 100 },
      { field: 'desc', minWidth: 280, title: '说明' },
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
          const result = await getDictionaryListApi({
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
  } as VxeTableGridOptions<DictionaryRow>,
});

function onCreate() {
  editingRow.value = undefined;
  drawerApi.open();
}

function onEdit(row: DictionaryRow) {
  editingRow.value = row;
  drawerApi.open();
}

function onDetail(row: DictionaryRow) {
  router.push(`/system/dictionaries/${row.ID}`);
}

function onDelete(row: DictionaryRow) {
  Modal.confirm({
    content: `确认删除字典「${row.name}」吗？`,
    onOk: async () => {
      await deleteDictionaryApi({ ID: row.ID });
      message.success('字典已删除');
      onRefresh();
    },
    title: '删除字典',
  });
}

function onRefresh() {
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="字典列表">
      <template #toolbar-tools>
        <Button type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增字典
        </Button>
      </template>
      <template #status="{ row }">
        <Tag :color="row.status ? 'success' : 'default'">
          {{ row.status ? '启用' : '停用' }}
        </Tag>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button size="small" type="link" @click="onDetail(row as DictionaryRow)">
            详情
          </Button>
          <Button size="small" type="link" @click="onEdit(row as DictionaryRow)">
            编辑
          </Button>
          <Button danger size="small" type="link" @click="onDelete(row as DictionaryRow)">
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
