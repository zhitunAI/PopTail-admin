<script lang="ts" setup>
import type { Recordable } from '@vben/types';

import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { AuthorityInfo } from '#/types/gin-ai-admin';

import type { RoleRow } from './authority-template/types';

import { shallowRef } from 'vue';

import dayjs from 'dayjs';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Switch } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteAuthorityApi,
  getAuthorityListApi,
  saveAuthorityApi,
} from '#/api/gin-ai-admin/admin';

import { useColumns, useGridFormSchema } from './authority-template/data';
import Form from './authority-template/modules/form.vue';

const [FormDrawer, formDrawerApi] = useVbenDrawer({
  connectedComponent: Form,
  destroyOnClose: true,
});

const lastRows = shallowRef<RoleRow[]>([]);

const [Grid, gridApi] = useVbenVxeGrid<RoleRow>({
  formOptions: {
    fieldMappingTime: [['createTime', ['startTime', 'endTime']]],
    schema: useGridFormSchema(),
    submitOnChange: true,
  },
  gridOptions: {
    columns: useColumns(),
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          const rows = await getRoleList({
            page: page.currentPage,
            pageSize: page.pageSize,
            ...formValues,
          });
          lastRows.value = rows.items;
          return rows;
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
  } as VxeTableGridOptions<RoleRow>,
});

async function getRoleList(params: Recordable<any>) {
  const authorities = await getAuthorityListApi();
  const rows = flattenAuthorities(authorities);
  const filtered = filterRows(rows, params);
  const page = Number(params.page ?? 1);
  const pageSize = Number(params.pageSize ?? 20);
  const start = (page - 1) * pageSize;

  return {
    items: filtered.slice(start, start + pageSize),
    total: filtered.length,
  };
}

function flattenAuthorities(list: AuthorityInfo[], level = 0, parentName = '根角色'): RoleRow[] {
  return list.flatMap((authority) => {
    const children = Array.isArray(authority.children) ? authority.children : [];
    const createdAt = resolveCreatedAt(authority);
    const status = (authority.enable ?? authority.status ?? 1) === 2 ? 0 : 1;
    const current: RoleRow = {
      authority,
      createTime: createdAt ? dayjs(createdAt).format('YYYY/MM/DD HH:mm:ss') : '-',
      defaultRouter: authority.defaultRouter || 'dashboard',
      id: String(authority.authorityId),
      name: authority.authorityName,
      parentId: authority.parentId,
      permissions: [],
      remark:
        level === 0
          ? `根角色 / 默认入口：${authority.defaultRouter || 'dashboard'}`
          : `父角色：${parentName} / 默认入口：${authority.defaultRouter || 'dashboard'}`,
      status,
    };
    return [current, ...flattenAuthorities(children, level + 1, authority.authorityName)];
  });
}

function resolveCreatedAt(authority: AuthorityInfo) {
  const raw = (authority as AuthorityInfo & { CreatedAt?: string | number; createdAt?: string | number }).CreatedAt
    ?? (authority as AuthorityInfo & { CreatedAt?: string | number; createdAt?: string | number }).createdAt;
  if (typeof raw === 'number') {
    return raw > 1e12 ? raw : raw * 1000;
  }
  if (typeof raw === 'string' && raw.trim()) {
    const parsed = Date.parse(raw);
    return Number.isFinite(parsed) ? parsed : 0;
  }
  return 0;
}

function filterRows(rows: RoleRow[], params: Recordable<any>) {
  const name = String(params.name ?? '').trim().toLowerCase();
  const id = String(params.id ?? '').trim();
  const remark = String(params.remark ?? '').trim().toLowerCase();
  const hasStatus = params.status === 0 || params.status === 1;
  const startTime = params.startTime ? dayjs(params.startTime).startOf('day').valueOf() : 0;
  const endTime = params.endTime ? dayjs(params.endTime).endOf('day').valueOf() : Number.MAX_SAFE_INTEGER;
  const hasTimeRange = Boolean(params.startTime && params.endTime);

  return rows.filter((row) => {
    const createdAt = resolveCreatedAt(row.authority);
    return (
      (!name || row.name.toLowerCase().includes(name)) &&
      (!id || row.id.includes(id)) &&
      (!remark || row.remark.toLowerCase().includes(remark)) &&
      (!hasStatus || row.status === params.status) &&
      (!hasTimeRange || (createdAt >= startTime && createdAt <= endTime))
    );
  });
}

function onActionClick({ code, row }: { code: string; row: RoleRow }) {
  switch (code) {
    case 'delete': {
      onDelete(row);
      break;
    }
    case 'edit': {
      onEdit(row);
      break;
    }
  }
}

function confirm(content: string, title: string) {
  return new Promise((resolve, reject) => {
    Modal.confirm({
      content,
      onCancel() {
        reject(new Error('已取消'));
      },
      onOk() {
        resolve(true);
      },
      title,
    });
  });
}

async function onStatusChange(checked: boolean, row: RoleRow) {
  const nextStatus = checked ? 1 : 0;
  try {
    await confirm(
      `你要将 ${row.name} 的状态切换为【${nextStatus === 1 ? '启用' : '禁用'}】吗？`,
      '切换状态',
    );
    await saveAuthorityApi(
      {
        ID: row.authority.ID,
        authorityId: row.authority.authorityId,
        authorityName: row.authority.authorityName,
        defaultRouter: row.authority.defaultRouter || 'dashboard',
        enable: nextStatus === 1 ? 1 : 2,
        parentId: row.authority.parentId,
        status: nextStatus === 1 ? 1 : 2,
      },
      { fallbackMode: 'update' },
    );
    message.success('状态已更新');
  } finally {
    onRefresh();
  }
}

function onEdit(row: RoleRow) {
  formDrawerApi.setData(row).open();
}

function onDelete(row: RoleRow) {
  Modal.confirm({
    content: `确认删除角色「${row.name}」吗？`,
    onOk: async () => {
      const hideLoading = message.loading({
        content: `正在删除 ${row.name}`,
        duration: 0,
        key: 'authority_delete_msg',
      });
      try {
        await deleteAuthorityApi(row.authority.authorityId);
        message.success({
          content: `已删除 ${row.name}`,
          key: 'authority_delete_msg',
        });
        onRefresh();
      } catch (error) {
        hideLoading();
        message.error(error instanceof Error ? error.message : '删除角色失败');
      }
    },
    title: '删除角色',
  });
}

function onRefresh() {
  gridApi.query();
}

function onCreate() {
  const nextAuthorityId =
    lastRows.value.length > 0
      ? Math.max(...lastRows.value.map((item) => Number(item.id))) + 1
      : 1000;
  formDrawerApi
    .setData({
      authority: {
        authorityId: nextAuthorityId,
        authorityName: '',
        children: [],
        defaultRouter: 'dashboard',
        ID: undefined,
        parentId: 0,
      },
    })
    .open();
}
</script>

<template>
  <Page auto-content-height>
    <FormDrawer @success="onRefresh" />
    <Grid table-title="角色列表">
      <template #toolbar-tools>
        <Button type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增角色
        </Button>
      </template>
      <template #status="{ row }">
        <Switch
          :checked="row.status === 1"
          checked-children="启用"
          un-checked-children="禁用"
          @change="onStatusChange(Boolean($event), row)"
        />
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button size="small" type="link" @click="onActionClick({ code: 'edit', row })">
            编辑
          </Button>
          <Button danger size="small" type="link" @click="onActionClick({ code: 'delete', row })">
            删除
          </Button>
        </Space>
      </template>
    </Grid>
  </Page>
</template>
