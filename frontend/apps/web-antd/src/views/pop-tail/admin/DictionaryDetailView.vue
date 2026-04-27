<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { DictionaryDetailInfo } from '#/types/pop-tail';

import { computed, nextTick, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteDictionaryDetailApi,
  getDictionaryDetailTreeApi,
  saveDictionaryDetailApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface DetailRow extends DictionaryDetailInfo {
  id: number;
  parentLabel: string;
  pathText: string;
}

const route = useRoute();
const router = useRouter();
const dictionaryId = computed(() => Number(route.params.id || 0));
const editingRow = ref<DetailRow>();
const { canUse } = useMenuButtonAccess();

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'label', label: '展示值' },
  { component: 'Input', fieldName: 'value', label: '字典值' },
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
  { component: 'Input', fieldName: 'label', label: '展示值', rules: 'required' },
  { component: 'Input', fieldName: 'value', label: '字典值', rules: 'required' },
  { component: 'Input', fieldName: 'extend', label: '扩展值' },
  { component: 'InputNumber', componentProps: { class: 'w-full', min: 0 }, fieldName: 'sort', label: '排序' },
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
        extend?: string;
        label: string;
        sort?: number;
        status: boolean;
        value: string;
      }>();
      await saveDictionaryDetailApi({
        ID: editingRow.value?.ID,
        extend: values.extend ?? '',
        label: values.label.trim(),
        level: editingRow.value?.level ?? 0,
        parentID: editingRow.value?.parentID ?? null,
        sort: Number(values.sort ?? 0),
        status: Boolean(values.status),
        sysDictionaryID: dictionaryId.value,
        value: values.value.trim(),
      });
      message.success(editingRow.value ? '字典项已更新' : '字典项已创建');
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
      extend: editingRow.value?.extend ?? '',
      label: editingRow.value?.label ?? '',
      sort: editingRow.value?.sort ?? 0,
      status: editingRow.value?.status ?? true,
      value: editingRow.value?.value ?? '',
    });
  },
});

const drawerTitle = computed(() => (editingRow.value ? '编辑字典项' : '新增字典项'));
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<DetailRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'label', slots: { default: 'label' }, title: '展示值', width: 180 },
      { field: 'value', slots: { default: 'value' }, title: '字典值', width: 180 },
      { field: 'extend', slots: { default: 'extend' }, minWidth: 180, title: '扩展值' },
      { field: 'pathText', slots: { default: 'pathText' }, minWidth: 260, title: '路径' },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 100 },
      { field: 'sort', slots: { default: 'sort' }, title: '排序', width: 90 },
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
    pagerConfig: { enabled: false },
    proxyConfig: {
      ajax: {
        query: async (_params, formValues) => {
          pageLoading.value = true;
          try {
            const tree = await getDictionaryDetailTreeApi(dictionaryId.value);
            const rows = flattenDetails(tree ?? []);
            return filterRows(rows, formValues);
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
  } as VxeTableGridOptions<DetailRow>,
});

function flattenDetails(list: DictionaryDetailInfo[], parentLabel = '根节点', path: string[] = []): DetailRow[] {
  return list.flatMap((item) => {
    const currentPath = [...path, item.label];
    const row = {
      ...item,
      id: item.ID,
      parentLabel,
      pathText: currentPath.join(' / '),
    };
    return [row, ...flattenDetails(item.children ?? [], item.label, currentPath)];
  });
}

function filterRows(rows: DetailRow[], formValues: Record<string, unknown>) {
  const label = String(formValues.label ?? '').trim().toLowerCase();
  const value = String(formValues.value ?? '').trim().toLowerCase();
  const hasStatus = typeof formValues.status === 'boolean';
  return rows.filter((item) =>
    (!label || item.label.toLowerCase().includes(label)) &&
    (!value || item.value.toLowerCase().includes(value)) &&
    (!hasStatus || item.status === formValues.status),
  );
}

function onCreate() {
  editingRow.value = undefined;
  drawerApi.open();
}

function onEdit(row: DetailRow) {
  editingRow.value = row;
  drawerApi.open();
}

function onDelete(row: DetailRow) {
  Modal.confirm({
    content: `确认删除字典项「${row.label}」吗？`,
    onOk: async () => {
      await deleteDictionaryDetailApi({ ID: row.ID });
      message.success('字典项已删除');
      onRefresh();
    },
    title: '删除字典项',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="字典项列表">
      <template #toolbar-tools>
        <Space>
          <Button @click="router.push('/system/dictionaries')">返回</Button>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            新增字典项
          </Button>
        </Space>
      </template>
      <template #label="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64%" />
        <span v-else>{{ row.label }}</span>
      </template>
      <template #value="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
        <span v-else>{{ row.value }}</span>
      </template>
      <template #extend="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="62%" />
        <span v-else>{{ row.extend }}</span>
      </template>
      <template #pathText="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
        <span v-else>{{ row.pathText }}</span>
      </template>
      <template #status="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
        <Tag v-else :color="row.status ? 'success' : 'default'">
          {{ row.status ? '启用' : '停用' }}
        </Tag>
      </template>
      <template #sort="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="44px" />
        <span v-else>{{ row.sort }}</span>
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
        <Space v-else size="small">
          <Button
            v-if="canUse(STANDARD_BUTTON_LABELS.edit)"
            size="small"
            type="link"
            @click="onEdit(row as DetailRow)"
          >
            编辑
          </Button>
          <Button
            v-if="canUse(STANDARD_BUTTON_LABELS.delete)"
            danger
            size="small"
            type="link"
            @click="onDelete(row as DetailRow)"
          >
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
