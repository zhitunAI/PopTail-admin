<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, InputNumber, Modal, Space, Switch, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteConsoleMenuApi, getConsoleMenuListApi, saveConsoleMenuApi } from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

import type { ConsoleMenuItem } from './shared';

interface ConsoleMenuRow extends ConsoleMenuItem {
  id: number;
}

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'title', label: '标题' },
  { component: 'Input', fieldName: 'group', label: '分组' },
  { component: 'Input', fieldName: 'path', label: '路径' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '显示', value: true },
        { label: '隐藏', value: false },
      ],
    },
    fieldName: 'visible',
    label: '显示',
  },
];

const sourceRows = ref<ConsoleMenuRow[]>([]);
const drawerOpen = ref(false);
const editing = ref<ConsoleMenuRow | null>(null);
const form = ref<ConsoleMenuRow>({
  group: '账号',
  icon: 'lucide:panel-left',
  id: 0,
  order: 1,
  path: '/member/profile',
  title: '',
  visible: true,
});
const title = computed(() => (editing.value ? '编辑控制台菜单' : '新增控制台菜单'));
const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<ConsoleMenuRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'title', slots: { default: 'title' }, title: '标题', minWidth: 160 },
      { field: 'group', slots: { default: 'group' }, title: '分组', minWidth: 140 },
      { field: 'path', slots: { default: 'path' }, title: '路径', minWidth: 220 },
      { field: 'icon', slots: { default: 'icon' }, title: '图标', minWidth: 160 },
      { field: 'order', slots: { default: 'order' }, title: '排序', width: 100 },
      { field: 'visible', slots: { default: 'visible' }, title: '显示', width: 120 },
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
            const result = await getConsoleMenuListApi();
            const rows = result.List.map((item) => ({
              group: item.group,
              icon: item.icon,
              id: item.ID,
              order: item.order,
              path: item.path,
              title: item.title,
              visible: item.visible,
            }));
            sourceRows.value = rows;
            const title = String(formValues.title ?? '').trim().toLowerCase();
            const group = String(formValues.group ?? '').trim().toLowerCase();
            const path = String(formValues.path ?? '').trim().toLowerCase();
            const hasVisible = typeof formValues.visible === 'boolean';
            const filtered = rows.filter((item) =>
              (!title || item.title.toLowerCase().includes(title)) &&
              (!group || item.group.toLowerCase().includes(group)) &&
              (!path || item.path.toLowerCase().includes(path)) &&
              (!hasVisible || item.visible === formValues.visible),
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
  } as VxeTableGridOptions<ConsoleMenuRow>,
});

function openCreate() {
  editing.value = null;
  form.value = {
    group: '账号',
    icon: 'lucide:panel-left',
    id: 0,
    order: sourceRows.value.length + 1,
    path: '/member/profile',
    title: '',
    visible: true,
  };
  drawerOpen.value = true;
}

function openEdit(row: ConsoleMenuRow) {
  editing.value = row;
  form.value = { ...row };
  drawerOpen.value = true;
}

async function save() {
  await saveConsoleMenuApi({
    ID: editing.value?.id,
    group: form.value.group,
    icon: form.value.icon,
    order: form.value.order,
    path: form.value.path,
    title: form.value.title,
    visible: form.value.visible,
  });
  drawerOpen.value = false;
  await onRefresh();
  message.success('控制台菜单已保存');
}

function remove(row: ConsoleMenuRow) {
  Modal.confirm({
    content: `确认删除控制台菜单「${row.title}」吗？`,
    onOk: async () => {
      await deleteConsoleMenuApi(row.id);
      await onRefresh();
      message.success('控制台菜单已删除');
    },
    title: '删除菜单',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="前端控制台菜单">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="控制台菜单列表">
        <template #toolbar-tools>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="openCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            新增菜单
          </Button>
        </template>
        <template #title="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="64%" />
          <span v-else>{{ row.title }}</span>
        </template>
        <template #group="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
          <span v-else>{{ row.group }}</span>
        </template>
        <template #path="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
          <span v-else>{{ row.path }}</span>
        </template>
        <template #icon="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="66%" />
          <span v-else>{{ row.icon }}</span>
        </template>
        <template #order="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="44px" />
          <span v-else>{{ row.order }}</span>
        </template>
        <template #visible="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
          <Switch v-else :checked="row.visible" disabled />
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
          <Space v-else>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="openEdit(row as ConsoleMenuRow)">编辑</Button>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="remove(row as ConsoleMenuRow)">删除</Button>
          </Space>
        </template>
      </Grid>
    </div>
    <Drawer v-model:open="drawerOpen" :title="title" width="480">
      <Form layout="vertical">
        <Form.Item label="标题"><Input v-model:value="form.title" /></Form.Item>
        <Form.Item label="分组"><Input v-model:value="form.group" /></Form.Item>
        <Form.Item label="路径"><Input v-model:value="form.path" /></Form.Item>
        <Form.Item label="图标"><Input v-model:value="form.icon" /></Form.Item>
        <Form.Item label="排序"><InputNumber v-model:value="form.order" class="w-full" /></Form.Item>
        <Form.Item label="显示"><Switch v-model:checked="form.visible" /></Form.Item>
        <div class="flex justify-end"><Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
