<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, InputNumber, Space, Switch, Table, TableColumn, message } from 'ant-design-vue';

import { deleteArticleCategoryApi, getArticleCategoryListApi, saveArticleCategoryApi } from '#/api/gin-ai-admin/admin';

import type { ArticleCategoryItem } from './shared';

const rows = ref<ArticleCategoryItem[]>([]);
const drawerOpen = ref(false);
const editing = ref<ArticleCategoryItem | null>(null);
const form = ref<ArticleCategoryItem>({ id: 0, name: '', slug: '', sort: 1, status: true });
const title = computed(() => (editing.value ? '编辑分类' : '新增分类'));

function openCreate() {
  editing.value = null;
  form.value = { id: 0, name: '', slug: '', sort: rows.value.length + 1, status: true };
  drawerOpen.value = true;
}
function openEdit(row: ArticleCategoryItem) { editing.value = row; form.value = { ...row }; drawerOpen.value = true; }
async function loadRows() {
  const page = await getArticleCategoryListApi();
  rows.value = page.List.map((item) => ({
    id: item.ID,
    name: item.name,
    slug: item.slug,
    sort: item.sort,
    status: item.status,
  }));
}
async function save() {
  await saveArticleCategoryApi({
    ID: editing.value?.id,
    name: form.value.name,
    slug: form.value.slug,
    sort: form.value.sort,
    status: form.value.status,
  });
  await loadRows();
  drawerOpen.value = false;
  message.success('文章分类已保存');
}
async function remove(id: number) {
  await deleteArticleCategoryApi(id);
  await loadRows();
  message.success('文章分类已删除');
}
onMounted(() => {
  void loadRows();
});
</script>

<template>
  <Page auto-content-height title="文章分类管理">
    <div class="mb-4 flex justify-end">
      <Button type="primary" @click="openCreate"><IconifyIcon class="size-5" icon="mdi:plus" />新增分类</Button>
    </div>
    <Table :data-source="rows" :pagination="false" row-key="id" :scroll="{ x: 900 }">
      <TableColumn data-index="name" key="name" title="分类名称" />
      <TableColumn data-index="slug" key="slug" title="Slug" />
      <TableColumn data-index="sort" key="sort" title="排序" />
      <TableColumn key="status" title="状态"><template #default="{ record }"><Switch :checked="record.status" disabled /></template></TableColumn>
      <TableColumn fixed="right" key="actions" title="操作" width="140">
        <template #default="{ record }"><Space><Button size="small" type="link" @click="openEdit(record)">编辑</Button><Button danger size="small" type="link" @click="remove(record.id)">删除</Button></Space></template>
      </TableColumn>
    </Table>
    <Drawer v-model:open="drawerOpen" :title="title" width="480">
      <Form layout="vertical">
        <Form.Item label="分类名称"><Input v-model:value="form.name" /></Form.Item>
        <Form.Item label="Slug"><Input v-model:value="form.slug" /></Form.Item>
        <Form.Item label="排序"><InputNumber v-model:value="form.sort" class="w-full" /></Form.Item>
        <Form.Item label="状态"><Switch v-model:checked="form.status" /></Form.Item>
        <div class="flex justify-end"><Button type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
