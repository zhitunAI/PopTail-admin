<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Select, Space, Table, TableColumn, Tabs, TabPane, message } from 'ant-design-vue';

import { deleteArticleApi, getArticleCategoryListApi, getArticleListApi, saveArticleApi, uploadFileAssetApi } from '#/api/gin-ai-admin/admin';

import type { ArticleCategoryItem, ArticleItem } from './shared';

const categories = ref<ArticleCategoryItem[]>([]);
const rows = ref<ArticleItem[]>([]);
const drawerOpen = ref(false);
const editing = ref<ArticleItem | null>(null);
const activeTab = ref('edit');
const imageInput = ref<HTMLInputElement | null>(null);
const markdownArea = ref<{ focus?: () => void; resizableTextArea?: { textArea?: HTMLTextAreaElement } } | null>(null);
const form = ref<ArticleItem>({
  categoryId: categories.value[0]?.id ?? 0,
  content: '# 新文章\n',
  id: 0,
  slug: '',
  status: 'draft',
  title: '',
  updatedAt: '',
});
const categoryOptions = computed(() => categories.value.map((item) => ({ label: item.name, value: item.id })));
const title = computed(() => (editing.value ? '编辑文章' : '新增文章'));

async function loadRows() {
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
  rows.value = articlePage.List.map((item) => ({
    categoryId: item.categoryId,
    content: item.content,
    id: item.ID,
    slug: item.slug,
    status: item.status === 'published' ? 'published' : 'draft',
    title: item.title,
    updatedAt: item.updatedAt,
  }));
}

function openCreate() {
  editing.value = null;
  form.value = { categoryId: categories.value[0]?.id ?? 0, content: '# 新文章\n', id: 0, slug: '', status: 'draft', title: '', updatedAt: '' };
  activeTab.value = 'edit';
  drawerOpen.value = true;
}
function openEdit(row: ArticleItem) { editing.value = row; form.value = { ...row }; activeTab.value = 'edit'; drawerOpen.value = true; }
async function save() {
  await saveArticleApi({
    ID: editing.value?.id,
    categoryId: form.value.categoryId,
    content: form.value.content,
    slug: form.value.slug,
    status: form.value.status,
    title: form.value.title,
  });
  await loadRows();
  drawerOpen.value = false;
  message.success('文章已保存');
}
async function remove(id: number) {
  await deleteArticleApi(id);
  await loadRows();
  message.success('文章已删除');
}
function categoryName(categoryId: number) { return categories.value.find((item) => item.id === categoryId)?.name ?? '-'; }

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
  const marks = '#'.repeat(level);
  insertMarkdown(`${marks} `, '', '标题');
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

onMounted(() => {
  void loadRows();
});
</script>

<template>
  <Page auto-content-height title="文章管理">
    <div class="mb-4 flex justify-end">
      <Button type="primary" @click="openCreate"><IconifyIcon class="size-5" icon="mdi:plus" />新增文章</Button>
    </div>
    <Table :data-source="rows" :pagination="false" row-key="id" :scroll="{ x: 1100 }">
      <TableColumn data-index="title" key="title" title="标题" />
      <TableColumn key="category" title="分类"><template #default="{ record }">{{ categoryName(record.categoryId) }}</template></TableColumn>
      <TableColumn data-index="slug" key="slug" title="Slug" />
      <TableColumn data-index="status" key="status" title="状态" />
      <TableColumn data-index="updatedAt" key="updatedAt" title="更新时间" />
      <TableColumn fixed="right" key="actions" title="操作" width="140">
        <template #default="{ record }"><Space><Button size="small" type="link" @click="openEdit(record)">编辑</Button><Button danger size="small" type="link" @click="remove(record.id)">删除</Button></Space></template>
      </TableColumn>
    </Table>
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
              <Button size="small" type="primary" @click="triggerImageUpload">上传图片</Button>
              <input ref="imageInput" accept="image/*" class="hidden" type="file" @change="uploadMarkdownImage" />
            </div>
            <Input.TextArea ref="markdownArea" v-model:value="form.content" :rows="18" />
          </TabPane>
          <TabPane key="preview" tab="预览">
            <pre class="whitespace-pre-wrap rounded-lg border border-[var(--ant-color-border)] p-4">{{ form.content }}</pre>
          </TabPane>
        </Tabs>
        <div class="mt-4 flex justify-end"><Button type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
