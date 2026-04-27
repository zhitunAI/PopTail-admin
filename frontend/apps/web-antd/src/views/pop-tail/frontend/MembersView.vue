<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Modal, Select, Space, Tabs, TabPane, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteMemberApi, getMemberListApi, saveMemberApi } from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

import type { MemberItem } from './shared';

interface MemberRow extends MemberItem {
  id: number;
}

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'email', label: '邮箱' },
  { component: 'Input', fieldName: 'nickname', label: '昵称' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '邮箱', value: 'email' },
        { label: 'Google', value: 'google' },
      ],
    },
    fieldName: 'provider',
    label: '注册方式',
  },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '启用', value: 'active' },
        { label: '禁用', value: 'disabled' },
      ],
    },
    fieldName: 'status',
    label: '状态',
  },
];

const sourceRows = ref<MemberRow[]>([]);
const drawerOpen = ref(false);
const editing = ref<MemberRow | null>(null);
const pageTab = ref('members');
const form = ref<MemberRow>({
  consolePath: '/member/console',
  email: '',
  id: 0,
  lastLogin: '',
  nickname: '',
  provider: 'email',
  status: 'active',
});
const title = computed(() => (editing.value ? '编辑会员' : '新增会员'));
const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<MemberRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'email', slots: { default: 'email' }, title: '邮箱', minWidth: 220 },
      { field: 'nickname', slots: { default: 'nickname' }, title: '昵称', width: 160 },
      { field: 'provider', slots: { default: 'provider' }, title: '注册方式', width: 120 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 120 },
      { field: 'lastLogin', slots: { default: 'lastLogin' }, title: '最近登录', width: 180 },
      { field: 'consolePath', slots: { default: 'consolePath' }, title: '会员控制台', minWidth: 220 },
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
            const data = await getMemberListApi();
            const rows = data.List.map((item) => ({
              consolePath: item.consolePath,
              email: item.email,
              id: item.ID,
              lastLogin: item.lastLogin,
              nickname: item.nickname,
              provider: (item.provider === 'google' ? 'google' : 'email') as MemberRow['provider'],
              status: (item.status === 'disabled' ? 'disabled' : 'active') as MemberRow['status'],
            }));
            sourceRows.value = rows;
            const email = String(formValues.email ?? '').trim().toLowerCase();
            const nickname = String(formValues.nickname ?? '').trim().toLowerCase();
            const provider = String(formValues.provider ?? '').trim();
            const status = String(formValues.status ?? '').trim();
            const filtered = rows.filter((item) =>
              (!email || item.email.toLowerCase().includes(email)) &&
              (!nickname || item.nickname.toLowerCase().includes(nickname)) &&
              (!provider || item.provider === provider) &&
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
  } as VxeTableGridOptions<MemberRow>,
});

function openCreate() {
  editing.value = null;
  form.value = { consolePath: '/member/console', email: '', id: 0, lastLogin: '', nickname: '', provider: 'email', status: 'active' };
  drawerOpen.value = true;
}

function openEdit(row: MemberRow) {
  editing.value = row;
  form.value = { ...row };
  drawerOpen.value = true;
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
  drawerOpen.value = false;
  await onRefresh();
  message.success('会员已保存');
}

function remove(row: MemberRow) {
  Modal.confirm({
    content: `确认删除会员「${row.email}」吗？`,
    onOk: async () => {
      await deleteMemberApi(row.id);
      await onRefresh();
      message.success('会员已删除');
    },
    title: '删除会员',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="会员管理">
    <Tabs v-model:activeKey="pageTab">
      <TabPane key="members" tab="会员列表">
        <div class="relative flex h-full min-h-0 flex-1 flex-col">
          <Grid class="h-full min-h-0 flex-1" table-title="会员列表">
            <template #toolbar-tools>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="openCreate">
                <IconifyIcon class="size-5" icon="mdi:plus" />
                新增会员
              </Button>
            </template>
            <template #email="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
              <span v-else>{{ row.email }}</span>
            </template>
            <template #nickname="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="62%" />
              <span v-else>{{ row.nickname }}</span>
            </template>
            <template #provider="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
              <span v-else>{{ row.provider }}</span>
            </template>
            <template #status="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
              <span v-else>{{ row.status }}</span>
            </template>
            <template #lastLogin="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
              <span v-else>{{ row.lastLogin }}</span>
            </template>
            <template #consolePath="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
              <span v-else>{{ row.consolePath }}</span>
            </template>
            <template #operation="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
              <Space v-else>
                <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="openEdit(row as MemberRow)">编辑</Button>
                <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link" @click="remove(row as MemberRow)">删除</Button>
              </Space>
            </template>
          </Grid>
        </div>
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
        <div class="flex justify-end"><Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" type="primary" @click="save">保存</Button></div>
      </Form>
    </Drawer>
  </Page>
</template>
