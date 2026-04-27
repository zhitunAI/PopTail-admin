<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { AuthorityInfo, UserInfo } from '#/types/pop-tail';

import { computed, nextTick, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Switch, TreeSelect } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteUserApi,
  getAuthorityListApi,
  getUserListApi,
  saveUserApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface UserRow extends UserInfo {
  id: number;
}

type TreeOption = {
  children?: TreeOption[];
  title: string;
  value: number;
};

const authorities = ref<AuthorityInfo[]>([]);
const editingRow = ref<UserRow>();
const roleSelectionCache = ref<Record<number, number[]>>({});
const { canUse } = useMenuButtonAccess();

const authorityTreeData = computed(() => buildAuthorityTreeData(authorities.value));

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'userName', label: '用户名' },
  { component: 'Input', fieldName: 'nickName', label: '昵称' },
  { component: 'Input', fieldName: 'phone', label: '手机号' },
  { component: 'Input', fieldName: 'email', label: '邮箱' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '启用', value: 1 },
        { label: '冻结', value: 2 },
      ],
    },
    fieldName: 'enable',
    label: '状态',
  },
];

const drawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'userName', label: '用户名', rules: 'required' },
  { component: 'Input', fieldName: 'nickName', label: '昵称', rules: 'required' },
  {
    component: 'TreeSelect',
    componentProps() {
      return {
        allowClear: true,
        class: 'w-full',
        dropdownStyle: { minWidth: '420px' },
        maxTagCount: 1,
        maxTagPlaceholder: (omittedValues: Array<{ label?: string }>) => `+ ${omittedValues.length}`,
        multiple: true,
        showCheckedStrategy: 'SHOW_ALL',
        treeCheckable: true,
        treeCheckStrictly: true,
        treeData: authorityTreeData.value,
        treeDefaultExpandAll: true,
      };
    },
    fieldName: 'authorityIds',
    label: '角色选择',
    rules: 'required',
  },
  { component: 'Input', fieldName: 'phone', label: '手机号' },
  { component: 'Input', fieldName: 'email', label: '邮箱' },
  {
    component: 'RadioGroup',
    componentProps: {
      buttonStyle: 'solid',
      optionType: 'button',
      options: [
        { label: '启用', value: 1 },
        { label: '冻结', value: 2 },
      ],
    },
    defaultValue: 1,
    fieldName: 'enable',
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
        authorityIds?: unknown[];
        email?: string;
        enable: number;
        nickName: string;
        phone?: string;
        userName: string;
      }>();
      const authorityIds = normalizeAuthorityValues(values.authorityIds ?? []);
      const authorityId = authorityIds[0] ?? editingRow.value?.authorityId ?? authorities.value[0]?.authorityId ?? 888;
      await saveUserApi({
        ID: editingRow.value?.ID,
        authorityId,
        authorityIds,
        email: values.email ?? '',
        enable: Number(values.enable ?? 1),
        headerImg: editingRow.value?.headerImg,
        nickName: values.nickName.trim(),
        phone: values.phone ?? '',
        userName: values.userName.trim(),
      });
      message.success(editingRow.value ? '用户已更新' : '用户已创建');
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
    const selectedAuthorityIds = editingRow.value?.authorities?.map((item) => item.authorityId)
      ?? [editingRow.value?.authorityId ?? authorities.value[0]?.authorityId ?? 888];
    drawerFormApi.setValues({
      authorityIds: selectedAuthorityIds.map((authorityId) => ({
        label: roleName(authorityId),
        value: authorityId,
      })),
      email: editingRow.value?.email ?? '',
      enable: editingRow.value?.enable ?? 1,
      nickName: editingRow.value?.nickName ?? '',
      phone: editingRow.value?.phone ?? '',
      userName: editingRow.value?.userName ?? '',
    });
  },
});

const drawerTitle = computed(() => (editingRow.value ? '编辑用户' : '新增用户'));
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<UserRow>({
  formOptions: {
    collapsed: false,
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'userName', slots: { default: 'userName' }, title: '用户名', width: 160 },
      { field: 'nickName', slots: { default: 'nickName' }, title: '昵称', width: 160 },
      { field: 'authority.authorityName', slots: { default: 'role' }, title: '用户角色', width: 260 },
      { field: 'phone', slots: { default: 'phone' }, title: '手机号', width: 160 },
      { field: 'email', slots: { default: 'email' }, minWidth: 220, title: '邮箱' },
      { field: 'enable', slots: { default: 'status' }, title: '状态', width: 120 },
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
            const [users, authorityList] = await Promise.all([
              getUserListApi({
                page: page.currentPage,
                pageSize: page.pageSize,
                ...formValues,
              }),
              authorities.value.length > 0 ? Promise.resolve(authorities.value) : getAuthorityListApi(),
            ]);
            authorities.value = authorityList;
            for (const item of users.List ?? []) {
              roleSelectionCache.value[item.ID] = (item.authorities ?? []).map((authority) => authority.authorityId);
            }
            return {
              items: (users.List ?? []).map((item) => ({ ...item, id: item.ID })),
              total: users.Total,
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
  } as VxeTableGridOptions<UserRow>,
});

function onCreate() {
  editingRow.value = undefined;
  drawerApi.open();
}

function onEdit(row: UserRow) {
  editingRow.value = row;
  drawerApi.open();
}

function onDelete(row: UserRow) {
  Modal.confirm({
    content: `确认删除用户「${row.userName}」吗？`,
    onOk: async () => {
      await deleteUserApi({ ID: row.ID });
      message.success('用户已删除');
      onRefresh();
    },
    title: '删除用户',
  });
}

async function onStatusChange(checked: boolean, row: UserRow) {
  await saveUserApi({
    ID: row.ID,
    authorityId: row.authorityId,
    authorityIds: rowAuthorityIds(row),
    email: row.email,
    enable: checked ? 1 : 2,
    headerImg: row.headerImg,
    nickName: row.nickName,
    phone: row.phone,
    userName: row.userName,
  });
  message.success('用户状态已更新');
  onRefresh();
}

function buildAuthorityTreeData(items: AuthorityInfo[]): TreeOption[] {
  return items.map((item) => ({
    children: item.children?.length ? buildAuthorityTreeData(item.children) : undefined,
    title: item.authorityName,
    value: item.authorityId,
  }));
}

async function onUserAuthoritiesChange(value: unknown, row: UserRow, extra?: unknown) {
  const authorityIds = normalizeAuthorityChange(value, row, extra);
  const authorityId = authorityIds[0] ?? row.authorityId;
  roleSelectionCache.value[row.ID] = authorityIds;
  await saveUserApi({
    ID: row.ID,
    authorityId,
    authorityIds,
    email: row.email,
    enable: row.enable,
    headerImg: row.headerImg,
    nickName: row.nickName,
    phone: row.phone,
    userName: row.userName,
  });
  message.success('用户角色已更新');
  onRefresh();
}

function rowAuthorityIds(row: UserRow) {
  const cached = roleSelectionCache.value[row.ID];
  if (cached) {
    return cached.length > 0 ? cached : [row.authorityId];
  }
  const ids = (row.authorities ?? []).map((item) => item.authorityId);
  return ids.length > 0 ? ids : [row.authorityId];
}

function roleName(authorityId: number) {
  const findRole = (items: AuthorityInfo[]): AuthorityInfo | undefined => {
    for (const item of items) {
      if (item.authorityId === authorityId) {
        return item;
      }
      const child = findRole(item.children ?? []);
      if (child) {
        return child;
      }
    }
  };
  return findRole(authorities.value)?.authorityName ?? String(authorityId);
}

function rowAuthorityTreeValues(row: UserRow) {
  return rowAuthorityIds(row).map((authorityId) => ({
    label: roleName(authorityId),
    value: authorityId,
  }));
}

function normalizeAuthorityValues(value: unknown) {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((item) => {
      if (typeof item === 'object' && item !== null && 'value' in item) {
        return Number((item as { value?: unknown }).value);
      }
      return Number(item);
    })
    .filter((item) => Number.isFinite(item) && item > 0);
}

function normalizeAuthorityChange(value: unknown, row: UserRow, extra?: unknown) {
  const extraData = extra as {
    checked?: boolean;
    selected?: boolean;
    triggerValue?: unknown;
  } | undefined;
  const triggerValue = normalizeAuthorityValue(extraData?.triggerValue);
  if (triggerValue) {
    const current = new Set(rowAuthorityIds(row));
    const checked = extraData?.checked ?? extraData?.selected;
    if (checked === false) {
      for (const id of [triggerValue, ...findAuthorityDescendants(triggerValue)]) {
        current.delete(id);
      }
    } else if (checked === true) {
      current.add(triggerValue);
      for (const id of findAuthorityAncestors(triggerValue)) {
        current.add(id);
      }
    } else {
      const normalized = normalizeAuthorityValues(value);
      return normalized.length > 0 ? normalized : [row.authorityId];
    }
    return normalizeAuthoritySelection(Array.from(current), row.authorityId);
  }
  const normalized = normalizeAuthorityValues(value);
  return normalizeAuthoritySelection(normalized, row.authorityId);
}

function normalizeAuthorityValue(value: unknown) {
  if (typeof value === 'object' && value !== null && 'value' in value) {
    return Number((value as { value?: unknown }).value);
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function normalizeAuthoritySelection(ids: number[], fallbackAuthorityId: number) {
  const normalized = new Set<number>();
  for (const id of ids) {
    normalized.add(id);
    for (const ancestorId of findAuthorityAncestors(id)) {
      normalized.add(ancestorId);
    }
  }
  const values = Array.from(normalized).sort((a, b) => a - b);
  return values.length > 0 ? values : [fallbackAuthorityId];
}

function findAuthorityAncestors(authorityId: number, items: AuthorityInfo[] = authorities.value, path: number[] = []) {
  for (const item of items) {
    const nextPath = [...path, item.authorityId];
    if (item.authorityId === authorityId) {
      return nextPath.slice(0, -1);
    }
    const childPath = findAuthorityAncestors(authorityId, item.children ?? [], nextPath);
    if (childPath.length > 0) {
      return childPath;
    }
  }
  return [];
}

function findAuthorityDescendants(authorityId: number, items: AuthorityInfo[] = authorities.value): number[] {
  for (const item of items) {
    if (item.authorityId === authorityId) {
      return flattenAuthorityIds(item.children ?? []);
    }
    const childIds = findAuthorityDescendants(authorityId, item.children ?? []);
    if (childIds.length > 0) {
      return childIds;
    }
  }
  return [];
}

function flattenAuthorityIds(items: AuthorityInfo[]): number[] {
  return items.flatMap((item) => [item.authorityId, ...flattenAuthorityIds(item.children ?? [])]);
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="用户列表">
      <template #toolbar-tools>
        <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增用户
        </Button>
      </template>
      <template #userName="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64%" />
        <span v-else>{{ row.userName }}</span>
      </template>
      <template #nickName="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
        <span v-else>{{ row.nickName }}</span>
      </template>
      <template #status="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="92px" />
        <Switch
          v-else
          :checked="row.enable === 1"
          checked-children="启用"
          un-checked-children="冻结"
          @change="(checked) => onStatusChange(Boolean(checked), row as UserRow)"
        />
      </template>
      <template #phone="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="60%" />
        <span v-else>{{ row.phone }}</span>
      </template>
      <template #email="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="78%" />
        <span v-else>{{ row.email }}</span>
      </template>
      <template #role="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="88%" />
        <TreeSelect
          v-else
          class="w-full user-role-tree-select"
          :dropdown-style="{ minWidth: '720px' }"
          max-tag-count="responsive"
          :max-tag-placeholder="(omittedValues) => `+ ${omittedValues.length}`"
          multiple
          placeholder="请选择用户角色"
          show-checked-strategy="SHOW_ALL"
          tree-checkable
          tree-check-strictly
          tree-default-expand-all
          :tree-data="authorityTreeData"
          :value="rowAuthorityTreeValues(row as UserRow)"
          @change="(value, _label, extra) => onUserAuthoritiesChange(value, row as UserRow, extra)"
        />
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
        <Space v-else size="small">
          <Button
            v-if="canUse(STANDARD_BUTTON_LABELS.edit)"
            size="small"
            type="link"
            @click="onEdit(row as UserRow)"
          >
            编辑
          </Button>
          <Button
            v-if="canUse(STANDARD_BUTTON_LABELS.delete)"
            danger
            size="small"
            type="link"
            @click="onDelete(row as UserRow)"
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

<style>
.user-role-tree-select .ant-select-selection-overflow {
  flex-wrap: nowrap;
}

.user-role-tree-select .ant-select-selection-overflow-item-rest {
  flex: 0 0 auto;
}
</style>
