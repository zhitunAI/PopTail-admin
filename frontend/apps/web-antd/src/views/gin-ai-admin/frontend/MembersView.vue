<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Select, Space, Table, TableColumn, Tabs, TabPane, message } from 'ant-design-vue';

import { deleteMemberApi, getMemberListApi, saveMemberApi } from '#/api/gin-ai-admin/admin';

import type { MemberItem } from './shared';

const rows = ref<MemberItem[]>([]);
const drawerOpen = ref(false);
const editing = ref<MemberItem | null>(null);
const pageTab = ref('members');
const form = ref<MemberItem>({ consolePath: '/member/console', email: '', id: 0, lastLogin: '', nickname: '', provider: 'email', status: 'active' });
const title = computed(() => (editing.value ? '编辑会员' : '新增会员'));

function openCreate() {
  editing.value = null;
  form.value = { consolePath: '/member/console', email: '', id: 0, lastLogin: '', nickname: '', provider: 'email', status: 'active' };
  drawerOpen.value = true;
}
function openEdit(row: MemberItem) { editing.value = row; form.value = { ...row }; drawerOpen.value = true; }
async function loadRows() {
  const page = await getMemberListApi();
  rows.value = page.List.map((item) => ({
    consolePath: item.consolePath,
    email: item.email,
    id: item.ID,
    lastLogin: item.lastLogin,
    nickname: item.nickname,
    provider: item.provider === 'google' ? 'google' : 'email',
    status: item.status === 'disabled' ? 'disabled' : 'active',
  }));
}
async function save() {
  await saveMemberApi({
    ID: editing.value?.id,
    consolePath: form.value.consolePath,
    email: form.value.email,
    lastLogin: form.value.lastLogin,
    nickname: form.value.nickname,
    provider: form.value.provider,
    status: form.value.status,
  });
  await loadRows();
  drawerOpen.value = false;
  message.success('会员已保存');
}
async function remove(id: number) {
  await deleteMemberApi(id);
  await loadRows();
  message.success('会员已删除');
}
onMounted(() => {
  void loadRows();
});
</script>

<template>
  <Page auto-content-height title="会员管理">
    <Tabs v-model:activeKey="pageTab">
      <TabPane key="members" tab="会员列表">
        <div class="mb-4 flex justify-end">
          <Button type="primary" @click="openCreate"><IconifyIcon class="size-5" icon="mdi:plus" />新增会员</Button>
        </div>
        <Table :data-source="rows" :pagination="false" row-key="id" :scroll="{ x: 1100 }">
          <TableColumn data-index="email" key="email" title="邮箱" />
          <TableColumn data-index="nickname" key="nickname" title="昵称" />
          <TableColumn data-index="provider" key="provider" title="注册方式" />
          <TableColumn data-index="status" key="status" title="状态" />
          <TableColumn data-index="lastLogin" key="lastLogin" title="最近登录" />
          <TableColumn data-index="consolePath" key="consolePath" title="会员控制台" />
          <TableColumn fixed="right" key="actions" title="操作" width="140">
            <template #default="{ record }"><Space><Button size="small" type="link" @click="openEdit(record)">编辑</Button><Button danger size="small" type="link" @click="remove(record.id)">删除</Button></Space></template>
          </TableColumn>
        </Table>
      </TabPane>
      <TabPane key="auth" tab="注册与找回">
        <div class="rounded-lg border border-[var(--ant-color-border)] p-4 text-sm text-[var(--ant-color-text-secondary)]">
          当前支持 `邮箱注册 / Google 注册 / 找回密码` 策略配置，后续可接入正式会员服务接口。
        </div>
      </TabPane>
    </Tabs>

    <Drawer v-model:open="drawerOpen" :title="title" width="520">
      <Form layout="vertical">
        <Form.Item label="邮箱"><Input v-model:value="form.email" /></Form.Item>
        <Form.Item label="昵称"><Input v-model:value="form.nickname" /></Form.Item>
        <Form.Item label="注册方式"><Select v-model:value="form.provider" :options="[{ label: '邮箱', value: 'email' }, { label: 'Google', value: 'google' }]" /></Form.Item>
        <Form.Item label="状态"><Select v-model:value="form.status" :options="[{ label: '启用', value: 'active' }, { label: '禁用', value: 'disabled' }]" /></Form.Item>
        <Form.Item label="会员控制台路径"><Input v-model:value="form.consolePath" /></Form.Item>
        <div class="flex justify-end"><Button type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
