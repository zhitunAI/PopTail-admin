<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, InputNumber, Space, Switch, Table, TableColumn, message } from 'ant-design-vue';

import { deleteFrontendNavApi, getFrontendNavListApi, saveFrontendNavApi } from '#/api/gin-ai-admin/admin';

import type { FrontendNavItem } from './shared';

const rows = ref<FrontendNavItem[]>([]);
const drawerOpen = ref(false);
const editing = ref<FrontendNavItem | null>(null);
const form = ref<FrontendNavItem>({ id: 0, icon: 'lucide:menu', order: 1, path: '/', title: '', visible: true });

const title = computed(() => (editing.value ? '编辑导航' : '新增导航'));

function openCreate() {
  editing.value = null;
  form.value = { id: 0, icon: 'lucide:menu', order: rows.value.length + 1, path: '/', title: '', visible: true };
  drawerOpen.value = true;
}

function openEdit(row: FrontendNavItem) {
  editing.value = row;
  form.value = { ...row };
  drawerOpen.value = true;
}

async function loadRows() {
  const page = await getFrontendNavListApi();
  rows.value = page.List.map((item) => ({
    icon: item.icon,
    id: item.ID,
    order: item.order,
    path: item.path,
    title: item.title,
    visible: item.visible,
  }));
}

async function save() {
  await saveFrontendNavApi({
    ID: editing.value?.id,
    icon: form.value.icon,
    order: form.value.order,
    path: form.value.path,
    title: form.value.title,
    visible: form.value.visible,
  });
  await loadRows();
  drawerOpen.value = false;
  message.success('前端导航已保存');
}

async function remove(id: number) {
  await deleteFrontendNavApi(id);
  await loadRows();
  message.success('前端导航已删除');
}

onMounted(() => {
  void loadRows();
});
</script>

<template>
  <Page auto-content-height title="前端导航管理">
    <div class="mb-4 flex justify-end">
      <Button type="primary" @click="openCreate">
        <IconifyIcon class="size-5" icon="mdi:plus" />
        新增导航
      </Button>
    </div>
    <Table :data-source="rows" :pagination="false" row-key="id" :scroll="{ x: 900 }">
      <TableColumn data-index="title" key="title" title="标题" />
      <TableColumn data-index="path" key="path" title="路径" />
      <TableColumn data-index="icon" key="icon" title="图标" />
      <TableColumn data-index="order" key="order" title="排序" />
      <TableColumn key="visible" title="显示">
        <template #default="{ record }">
          <Switch :checked="record.visible" disabled />
        </template>
      </TableColumn>
      <TableColumn fixed="right" key="actions" title="操作" width="140">
        <template #default="{ record }">
          <Space>
            <Button size="small" type="link" @click="openEdit(record)">编辑</Button>
            <Button danger size="small" type="link" @click="remove(record.id)">删除</Button>
          </Space>
        </template>
      </TableColumn>
    </Table>
    <Drawer v-model:open="drawerOpen" :title="title" width="480">
      <Form layout="vertical">
        <Form.Item label="标题"><Input v-model:value="form.title" /></Form.Item>
        <Form.Item label="路径"><Input v-model:value="form.path" /></Form.Item>
        <Form.Item label="图标"><Input v-model:value="form.icon" /></Form.Item>
        <Form.Item label="排序"><InputNumber v-model:value="form.order" class="w-full" /></Form.Item>
        <Form.Item label="显示"><Switch v-model:checked="form.visible" /></Form.Item>
        <div class="flex justify-end"><Button type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
