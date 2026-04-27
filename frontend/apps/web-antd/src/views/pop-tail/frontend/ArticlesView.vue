<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Modal, Select, Space, Tabs, TabPane, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteArticleApi, getArticleCategoryListApi, getArticleListApi, saveArticleApi, uploadFileAssetApi } from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

import type { ArticleCategoryItem, ArticleItem } from './shared';

interface ArticleRow extends ArticleItem {
  categoryName: string;
  id: number;
}

const categories = ref<ArticleCategoryItem[]>([]);
const rows = ref<ArticleRow[]>([]);
const drawerOpen = ref(false);
const editing = ref<ArticleRow | null>(null);
const activeTab = ref('edit');
const imageInput = ref<HTMLInputElement | null>(null);
const markdownArea = ref<{ focus?: () => void; resizableTextArea?: { textArea?: HTMLTextAreaElement } } | null>(null);
const { canUse } = useMenuButtonAccess();
const form = ref<ArticleItem>({
  categoryId: 0,
  content: '# 新文章\n',
  id: 0,
  slug: '',
  status: 'draft',
  title: '',
  updatedAt: '',
});
const categoryOptions = computed(() => categories.value.map((item) => ({ label: item.name, value: item.id })));
const title = computed(() => (editing.value ? '编辑文章' : '新增文章'));
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'title', label: '标题' },
  { component: 'Input', fieldName: 'slug', label: 'Slug' },
  {
    component: 'Select',
    componentProps: () => ({
      allowClear: true,
      options: categoryOptions.value,
    }),
    fieldName: 'categoryId',
    label: '分类',
  },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '草稿', value: 'draft' },
        { label: '已发布', value: 'published' },
      ],
    },
    fieldName: 'status',
    label: '状态',
  },
];

const [Grid, gridApi] = useVbenVxeGrid<ArticleRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'title', slots: { default: 'title' }, title: '标题', minWidth: 240 },
      { field: 'categoryName', slots: { default: 'categoryName' }, title: '分类', width: 160 },
      { field: 'slug', slots: { default: 'slug' }, title: 'Slug', minWidth: 180 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 120 },
      { field: 'updatedAt', slots: { default: 'updatedAt' }, title: '更新时间', width: 180 },
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
            const [categoryPage, articlePage] = await Promise.all([
              getArticleCategoryListApi(),
              getArticleListApi(),
            ]);
            categories.value = categoryPage.List.map((item) => ({
              id: item.ID,
              name: item.name,
              slug: item.slug,
              sort: item.sort,
              status: item.status,
            }));
            const categoryMap = new Map(categories.value.map((item) => [item.id, item.name]));
            const mappedRows = articlePage.List.map((item) => ({
              categoryId: item.categoryId,
              categoryName: categoryMap.get(item.categoryId) ?? '-',
              content: item.content,
              id: item.ID,
              slug: item.slug,
              status: (item.status === 'published' ? 'published' : 'draft') as ArticleRow['status'],
              title: item.title,
              updatedAt: item.updatedAt,
            }));
            rows.value = mappedRows;
            const title = String(formValues.title ?? '').trim().toLowerCase();
            const slug = String(formValues.slug ?? '').trim().toLowerCase();
            const categoryId = Number(formValues.categoryId ?? 0);
            const status = String(formValues.status ?? '').trim();
            const filtered = mappedRows.filter((item) =>
              (!title || item.title.toLowerCase().includes(title)) &&
              (!slug || item.slug.toLowerCase().includes(slug)) &&
              (!categoryId || item.categoryId === categoryId) &&
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
  } as VxeTableGridOptions<ArticleRow>,
});

function openCreate() {
  editing.value = null;
  form.value = { categoryId: categories.value[0]?.id ?? 0, content: '# 新文章\n', id: 0, slug: '', status: 'draft', title: '', updatedAt: '' };
  activeTab.value = 'edit';
  drawerOpen.value = true;
}

function openEdit(row: ArticleRow) {
  editing.value = row;
  form.value = { ...row };
  activeTab.value = 'edit';
  drawerOpen.value = true;
}

async function save() {
  await saveArticleApi({
    ID: editing.value?.id,
    categoryId: form.value.categoryId,
    content: form.value.content,
    slug: form.value.slug,
    status: form.value.status,
    title: form.value.title,
  });
  drawerOpen.value = false;
  await onRefresh();
  message.success('文章已保存');
}

function remove(row: ArticleRow) {
  Modal.confirm({
    content: `确认删除文章「${row.title}」吗？`,
    onOk: async () => {
      await deleteArticleApi(row.id);
      await onRefresh();
      message.success('文章已删除');
    },
    title: '删除文章',
  });
}

function getMarkdownTextarea() {
  return markdownArea.value?.resizableTextArea?.textArea ?? null;
}

function insertMarkdown(before: string, after = '', fallback = '') {
  const textarea = getMarkdownTextarea();
  const content = form.value.content;
  const start = textarea?.selectionStart ?? content.length;
  const end = textarea?.selectionEnd ?? content.length;
  const selected = content.slice(start, end) || fallback;
  form.value.content = `${content.slice(0, start)}${before}${selected}${after}${content.slice(end)}`;
  requestAnimationFrame(() => {
    const next = getMarkdownTextarea();
    const cursor = start + before.length + selected.length + after.length;
    next?.focus();
    next?.setSelectionRange(cursor, cursor);
  });
}

function insertHeading(level: 1 | 2 | 3) {
  insertMarkdown(`${'#'.repeat(level)} `, '', '标题');
}

function insertLink() {
  const href = window.prompt('请输入链接地址', 'https://');
  if (!href) return;
  insertMarkdown('[', `](${href})`, '链接文字');
}

function insertImageLink() {
  const href = window.prompt('请输入图片地址', 'https://');
  if (!href) return;
  insertMarkdown('![', `](${href})`, '图片描述');
}

function insertCodeBlock() {
  insertMarkdown('```\n', '\n```', '代码块');
}

function triggerImageUpload() {
  imageInput.value?.click();
}

async function uploadMarkdownImage(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) return;
  const asset = await uploadFileAssetApi(file);
  insertMarkdown('![', `](${asset.url})`, asset.name || '图片');
  message.success('图片已上传并插入');
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="文章管理">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="文章列表">
        <template #toolbar-tools>
          <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="openCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            新增文章
          </Button>
        </template>
        <template #title="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
          <span v-else>{{ row.title }}</span>
        </template>
        <template #categoryName="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
          <span v-else>{{ row.categoryName }}</span>
        </template>
        <template #slug="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="70%" />
          <span v-else>{{ row.slug }}</span>
        </template>
        <template #status="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
          <span v-else>{{ row.status }}</span>
        </template>
        <template #updatedAt="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
          <span v-else>{{ row.updatedAt }}</span>
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
          <Space v-else>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="openEdit(row as ArticleRow)">编辑</Button>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="remove(row as ArticleRow)">删除</Button>
          </Space>
        </template>
      </Grid>
    </div>
    <Drawer v-model:open="drawerOpen" :title="title" width="860">
      <Form layout="vertical">
        <div class="grid grid-cols-2 gap-4">
          <Form.Item label="标题"><Input v-model:value="form.title" /></Form.Item>
          <Form.Item label="Slug"><Input v-model:value="form.slug" /></Form.Item>
          <Form.Item label="分类"><Select v-model:value="form.categoryId" :options="categoryOptions" /></Form.Item>
          <Form.Item label="状态"><Select v-model:value="form.status" :options="[{ label: '草稿', value: 'draft' }, { label: '已发布', value: 'published' }]" /></Form.Item>
        </div>
        <Tabs v-model:activeKey="activeTab">
          <TabPane key="edit" tab="Markdown 编辑">
            <div class="mb-3 flex flex-wrap gap-2 rounded-lg border border-[var(--ant-color-border)] bg-[var(--ant-color-bg-container)] p-2">
              <Button size="small" @click="insertHeading(1)">H1</Button>
              <Button size="small" @click="insertHeading(2)">H2</Button>
              <Button size="small" @click="insertHeading(3)">H3</Button>
              <Button size="small" @click="insertMarkdown('**', '**', '加粗文字')">加粗</Button>
              <Button size="small" @click="insertMarkdown('*', '*', '斜体文字')">斜体</Button>
              <Button size="small" @click="insertMarkdown('> ', '', '引用内容')">引用</Button>
              <Button size="small" @click="insertMarkdown('`', '`', 'code')">行内代码</Button>
              <Button size="small" @click="insertCodeBlock">代码块</Button>
              <Button size="small" @click="insertLink">链接</Button>
              <Button size="small" @click="insertImageLink">图片链接</Button>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="primary" @click="triggerImageUpload">上传图片</Button>
              <input ref="imageInput" accept="image/*" class="hidden" type="file" @change="uploadMarkdownImage" />
            </div>
            <Input.TextArea ref="markdownArea" v-model:value="form.content" :rows="18" />
          </TabPane>
          <TabPane key="preview" tab="预览">
            <pre class="whitespace-pre-wrap rounded-lg border border-[var(--ant-color-border)] p-4">{{ form.content }}</pre>
          </TabPane>
        </Tabs>
        <div class="mt-4 flex justify-end"><Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
