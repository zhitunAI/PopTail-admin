<script lang="ts" setup>
import type { RoleRow } from '../types';

import type { ApiInfo, MenuInfo, PolicyPath } from '#/types/gin-ai-admin';

import type { Recordable } from '@vben/types';

import { computed, nextTick, ref } from 'vue';

import { Tree, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Empty, Spin, TabPane, Tabs } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  addMenuAuthorityApi,
  getApiListApi,
  getMenuAuthorityApi,
  getMenuListApi,
  saveAuthorityApi,
  setPolicyPathByAuthorityIdApi,
} from '#/api/gin-ai-admin/admin';
import { getPolicyPathByAuthorityId } from '#/api/gin-ai-admin/auth';

import { useFormSchema } from '../data';

const emit = defineEmits<{
  success: [];
}>();

const rowData = ref<RoleRow>();
const permissions = ref<MenuInfo[]>([]);
const apiPermissions = ref<ApiPermissionNode[]>([]);
const selectedMenuPermissionIds = ref<number[]>([]);
const selectedApiPermissionKeys = ref<string[]>([]);
const loadingPermissions = ref(false);

type ApiPermissionNode = {
  children?: ApiPermissionNode[];
  key: string;
  label: string;
  method?: string;
  path?: string;
};

const [Form, formApi] = useVbenForm({
  schema: useFormSchema(),
  showDefaultActions: false,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    const { valid } = await formApi.validate();
    if (!valid) return;
    const values = await formApi.getValues<{
      authorityId: number;
      authorityName: string;
      defaultRouter?: string;
      parentId?: number;
      status?: number;
    }>();
    drawerApi.lock();
    try {
      const source = rowData.value?.authority;
      const saved = await saveAuthorityApi(
        {
          ID: source?.ID,
          authorityId: Number(values.authorityId),
          authorityName: String(values.authorityName ?? '').trim(),
          defaultRouter: String(values.defaultRouter ?? '').trim() || 'dashboard',
          enable: values.status === 1 ? 1 : 2,
          parentId: Number(values.parentId ?? 0),
          status: values.status === 1 ? 1 : 2,
        },
        { fallbackMode: source ? 'update' : 'create' },
      );
      await saveMenuPermissions(saved.authorityId, selectedMenuPermissionIds.value);
      await saveApiPermissions(saved.authorityId, selectedApiPermissionKeys.value);
      emit('success');
      drawerApi.close();
    } finally {
      drawerApi.unlock();
    }
  },
  async onOpenChange(isOpen) {
    if (!isOpen) return;
    const data = drawerApi.getData<RoleRow>();
    formApi.resetForm();
    rowData.value = data?.id ? data : undefined;
    const source = rowData.value?.authority ?? data?.authority;
    const [selectedPermissions, selectedApiPermissions] = await Promise.all([
      loadPermissions(source?.authorityId),
      loadApiPermissions(source?.authorityId),
    ]);
    selectedMenuPermissionIds.value = selectedPermissions;
    selectedApiPermissionKeys.value = selectedApiPermissions;
    await nextTick();
    formApi.setValues({
      authorityId: source?.authorityId ?? 1000,
      authorityName: source?.authorityName ?? '',
      defaultRouter: source?.defaultRouter || 'dashboard',
      parentId: source?.parentId ?? 0,
      status: (source?.enable ?? source?.status ?? 1) === 2 ? 0 : 1,
    });
  },
});

const drawerTitle = computed(() => (rowData.value?.id ? '编辑角色' : '新增角色'));

async function loadPermissions(authorityId?: number) {
  loadingPermissions.value = true;
  try {
    const [menuList, selectedMenus] = await Promise.all([
      permissions.value.length > 0
        ? Promise.resolve({ List: permissions.value })
        : getMenuListApi(),
      authorityId ? getMenuAuthorityApi(authorityId) : Promise.resolve([]),
    ]);
    if (permissions.value.length === 0) {
      permissions.value = buildUniqueMenuTree(menuList.List ?? []);
    }
    const menuIds = new Set(flattenMenus(permissions.value).map((item) => item.ID));
    return Array.from(
      new Set((selectedMenus ?? []).map((item) => item.ID).filter((id) => menuIds.has(id))),
    );
  } finally {
    loadingPermissions.value = false;
  }
  return [];
}

async function loadApiPermissions(authorityId?: number) {
  loadingPermissions.value = true;
  try {
    const [apiPage, selectedPolicies] = await Promise.all([
      apiPermissions.value.length > 0
        ? Promise.resolve({ List: flattenApiNodes(apiPermissions.value).filter((item) => item.path) as unknown as ApiInfo[] })
        : getApiListApi({ page: 1, pageSize: 500 }),
      authorityId ? getPolicyPathByAuthorityId(authorityId) : Promise.resolve([]),
    ]);
    if (apiPermissions.value.length === 0) {
      apiPermissions.value = buildApiTree(apiPage.List ?? []);
    }
    const available = new Set(flattenApiNodes(apiPermissions.value).filter((item) => item.path).map((item) => item.key));
    return (selectedPolicies ?? [])
      .map((item) => apiPolicyKey(item))
      .filter((key) => available.has(key));
  } finally {
    loadingPermissions.value = false;
  }
}

async function saveMenuPermissions(authorityId: number, selectedPermissionIds: number[]) {
  const menuMap = new Map(flattenMenus(permissions.value).map((item) => [item.ID, item] as const));
  const menus = selectedPermissionIds
    .map((id) => menuMap.get(id))
    .filter((item): item is MenuInfo => Boolean(item));
  await addMenuAuthorityApi({ authorityId, menus });
}

async function saveApiPermissions(authorityId: number, selectedKeys: string[]) {
  const nodeMap = new Map(flattenApiNodes(apiPermissions.value).map((item) => [item.key, item] as const));
  const policies = selectedKeys
    .map((key) => nodeMap.get(key))
    .filter((item): item is ApiPermissionNode => Boolean(item?.path && item.method))
    .map((item) => ({
      method: item.method!,
      path: item.path!,
    }));
  await setPolicyPathByAuthorityIdApi({ authorityId, policies });
}

function apiPolicyKey(item: Pick<PolicyPath, 'method' | 'path'>) {
  return `${item.method.toUpperCase()} ${item.path}`;
}

function buildApiTree(items: ApiInfo[]): ApiPermissionNode[] {
  const groups = new Map<string, ApiPermissionNode[]>();
  for (const item of items) {
    const group = item.apiGroup || '默认分组';
    const key = apiPolicyKey({ method: item.method, path: item.path });
    const children = groups.get(group) ?? [];
    children.push({
      key,
      label: `${item.description || item.path}  ${item.method.toUpperCase()} ${item.path}`,
      method: item.method.toUpperCase(),
      path: item.path,
    });
    groups.set(group, children);
  }
  return Array.from(groups.entries()).map(([group, children]) => ({
    children,
    key: `group:${group}`,
    label: group,
  }));
}

function flattenApiNodes(list: ApiPermissionNode[]): ApiPermissionNode[] {
  return list.flatMap((item) => [item, ...flattenApiNodes(item.children ?? [])]);
}

function flattenMenus(list: MenuInfo[]): MenuInfo[] {
  return list.flatMap((item) => [item, ...flattenMenus(Array.isArray(item.children) ? item.children : [])]);
}

function buildUniqueMenuTree(list: MenuInfo[]) {
  const byId = new Map<number, MenuInfo>();

  for (const item of list) {
    byId.set(item.ID, {
      ...item,
      children: [],
    });
  }

  const roots: MenuInfo[] = [];
  for (const item of byId.values()) {
    const parent = item.parentId ? byId.get(item.parentId) : undefined;
    if (parent) {
      parent.children = [...(parent.children ?? []), item];
    } else {
      roots.push(item);
    }
  }

  const sortMenus = (items: MenuInfo[]): MenuInfo[] =>
    items
      .sort((a, b) => (a.sort ?? 0) - (b.sort ?? 0))
      .map((item) => ({
        ...item,
        children: sortMenus(item.children ?? []),
      }));

  return sortMenus(roots);
}

function getNodeClass(node: Recordable<any>) {
  const classes: string[] = [];
  if ((node.value?.menuBtn?.length ?? 0) > 0) {
    classes.push('inline-flex');
  }
  return classes.join(' ');
}
</script>

<template>
  <Drawer :title="drawerTitle">
    <Form />
    <div class="mx-4 mt-4">
      <Tabs>
        <TabPane key="menu" tab="角色菜单">
          <Spin :spinning="loadingPermissions" wrapper-class-name="w-full">
          <Tree
            :tree-data="permissions"
            bordered
            :default-expanded-level="2"
            :get-node-class="getNodeClass"
            icon-field="meta.icon"
            label-field="meta.title"
            multiple
            v-model:model-value="selectedMenuPermissionIds"
            value-field="ID"
          >
            <template #node="{ value }">
              <IconifyIcon v-if="value.meta?.icon" :icon="value.meta.icon" />
              {{ value.meta?.title || value.name }}
            </template>
          </Tree>
          </Spin>
        </TabPane>
        <TabPane key="api" tab="角色 API">
          <Spin :spinning="loadingPermissions" wrapper-class-name="w-full">
          <Tree
            :tree-data="apiPermissions"
            bordered
            :default-expanded-level="1"
            label-field="label"
            multiple
            v-model:model-value="selectedApiPermissionKeys"
            value-field="key"
          />
          </Spin>
        </TabPane>
        <TabPane key="resource" tab="资源权限">
          <Empty description="资源权限预留，可后续接入文件、素材、会员资源权限。" />
        </TabPane>
      </Tabs>
    </div>
  </Drawer>
</template>
