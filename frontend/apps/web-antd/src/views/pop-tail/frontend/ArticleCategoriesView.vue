<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, InputNumber, Modal, Space, Switch, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteArticleCategoryApi, getArticleCategoryListApi, saveArticleCategoryApi } from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

import type { ArticleCategoryItem } from './shared';

interface ArticleCategoryRow extends ArticleCategoryItem {
  id: number;
}

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '分类名称' },
  { component: 'Input', fieldName: 'slug', label: 'Slug' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '启用', value: true },
        { label: '禁用', value: false },
      ],
    },
    fieldName: 'status',
    label: '状态',
  },
];

const sourceRows = ref<ArticleCategoryRow[]>([]);
const drawerOpen = ref(false);
const editing = ref<ArticleCategoryRow | null>(null);
const form = ref<ArticleCategoryRow>({ id: 0, name: '', slug: '', sort: 1, status: true });
const title = computed(() => (editing.value ? '编辑分类' : '新增分类'));
const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<ArticleCategoryRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: '分类名称', minWidth: 180 },
      { field: 'slug', slots: { default: 'slug' }, title: 'Slug', minWidth: 200 },
      { field: 'sort', slots: { default: 'sort' }, title: '排序', width: 100 },
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
            const result = await getArticleCategoryListApi();
            const rows = result.List.map((item) => ({
              id: item.ID,
              name: item.name,
              slug: item.slug,
              sort: item.sort,
              status: item.status,
            }));
            sourceRows.value = rows;
            const name = String(formValues.name ?? '').trim().toLowerCase();
            const slug = String(formValues.slug ?? '').trim().toLowerCase();
            const hasStatus = typeof formValues.status === 'boolean';
            const filtered = rows.filter((item) =>
              (!name || item.name.toLowerCase().includes(name)) &&
              (!slug || item.slug.toLowerCase().includes(slug)) &&
              (!hasStatus || item.status === formValues.status),
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
  } as VxeTableGridOptions<ArticleCategoryRow>,
});

function openCreate() {
  editing.value = null;
  form.value = {
    id: 0,
    name: '',
    slug: '',
    sort: sourceRows.value.length + 1,
    status: true,
  };
  drawerOpen.value = true;
}

function openEdit(row: ArticleCategoryRow) {
  editing.value = row;
  form.value = { ...row };
  drawerOpen.value = true;
}

async function save() {
  await saveArticleCategoryApi({
    ID: editing.value?.id,
    name: form.value.name,
    slug: form.value.slug,
    sort: form.value.sort,
    status: form.value.status,
  });
  drawerOpen.value = false;
  await onRefresh();
  message.success('文章分类已保存');
}

function remove(row: ArticleCategoryRow) {
  Modal.confirm({
    content: `确认删除分类「${row.name}」吗？`,
    onOk: async () => {
      await deleteArticleCategoryApi(row.id);
      await onRefresh();
      message.success('文章分类已删除');
    },
    title: '删除分类',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="文章分类管理">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="分类列表">
        <template #toolbar-tools>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="openCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            新增分类
          </Button>
        </template>
        <template #name="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="68%" />
          <span v-else>{{ row.name }}</span>
        </template>
        <template #slug="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="72%" />
          <span v-else>{{ row.slug }}</span>
        </template>
        <template #sort="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="44px" />
          <span v-else>{{ row.sort }}</span>
        </template>
        <template #status="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
          <Switch v-else :checked="row.status" disabled />
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
          <Space v-else>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="openEdit(row as ArticleCategoryRow)">编辑</Button>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="remove(row as ArticleCategoryRow)">删除</Button>
          </Space>
        </template>
      </Grid>
    </div>
    <Drawer v-model:open="drawerOpen" :title="title" width="480">
      <Form layout="vertical">
        <Form.Item label="分类名称"><Input v-model:value="form.name" /></Form.Item>
        <Form.Item label="Slug"><Input v-model:value="form.slug" /></Form.Item>
        <Form.Item label="排序"><InputNumber v-model:value="form.sort" class="w-full" /></Form.Item>
        <Form.Item label="状态"><Switch v-model:checked="form.status" /></Form.Item>
        <div class="flex justify-end"><Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
