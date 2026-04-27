<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { ParamInfo } from '#/types/pop-tail';

import { computed, nextTick, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteParamApi,
  getParamsListApi,
  saveParamApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

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
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<ParamRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'ID', slots: { default: 'id' }, title: 'ID', width: 90 },
      { field: 'key', slots: { default: 'key' }, title: '参数键', width: 220 },
      { field: 'value', slots: { default: 'value' }, title: '参数值', minWidth: 260 },
      { field: 'desc', slots: { default: 'desc' }, title: '说明', minWidth: 260 },
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
            const result = await getParamsListApi({
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

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="参数列表">
      <template #toolbar-tools>
        <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增参数
        </Button>
      </template>
      <template #id="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="40px" />
        <span v-else>{{ row.ID }}</span>
      </template>
      <template #key="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="68%" />
        <span v-else>{{ row.key }}</span>
      </template>
      <template #value="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="78%" />
        <span v-else>{{ row.value }}</span>
      </template>
      <template #desc="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
        <span v-else>{{ row.desc }}</span>
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
        <Space v-else size="small">
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="onEdit(row as ParamRow)">
            编辑
          </Button>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="onDelete(row as ParamRow)">
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
