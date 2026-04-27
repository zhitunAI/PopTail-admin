<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { DictionaryInfo } from '#/types/pop-tail';

import { computed, nextTick, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteDictionaryApi,
  getDictionaryListApi,
  saveDictionaryApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface DictionaryRow extends DictionaryInfo {
  id: number;
}

const router = useRouter();
const editingRow = ref<DictionaryRow>();
const { canUse } = useMenuButtonAccess();

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
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<DictionaryRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: '字典名称', width: 180 },
      { field: 'type', slots: { default: 'type' }, title: '类型标识', width: 200 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 100 },
      { field: 'desc', slots: { default: 'desc' }, minWidth: 280, title: '说明' },
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
            const result = await getDictionaryListApi({
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

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="字典列表">
      <template #toolbar-tools>
        <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增字典
        </Button>
      </template>
      <template #name="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="66%" />
        <span v-else>{{ row.name }}</span>
      </template>
      <template #type="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="70%" />
        <span v-else>{{ row.type }}</span>
      </template>
      <template #status="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
        <Tag v-else :color="row.status ? 'success' : 'default'">
          {{ row.status ? '启用' : '停用' }}
        </Tag>
      </template>
      <template #desc="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
        <span v-else>{{ row.desc }}</span>
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="160px" />
        <Space v-else size="small">
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.view)" size="small" type="link" @click="onDetail(row as DictionaryRow)">
            详情
          </Button>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="onEdit(row as DictionaryRow)">
            编辑
          </Button>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="onDelete(row as DictionaryRow)">
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
