<script lang="ts" setup>
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { MenuInfo } from '#/types/gin-ai-admin';

import type { MenuRow, TemplateMenuType } from './menu-template/types';

import { shallowRef } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteBaseMenuApi, getMenuListApi } from '#/api/gin-ai-admin/admin';

import { getMenuTypeOptions, useColumns } from './menu-template/data';
import Form from './menu-template/modules/form.vue';

const [FormDrawer, formDrawerApi] = useVbenDrawer({
  connectedComponent: Form,
  destroyOnClose: true,
});

const sourceMenus = shallowRef<MenuInfo[]>([]);
const menuTypeOptions = getMenuTypeOptions();

const [Grid, gridApi] = useVbenVxeGrid<MenuRow>({
  gridOptions: {
    columns: useColumns(),
    height: 'auto',
    keepSource: true,
    pagerConfig: {
      enabled: false,
    },
    proxyConfig: {
      ajax: {
        query: async () => {
          const result = await getMenuListApi();
          sourceMenus.value = buildUniqueMenuTree(result.List ?? []);
          return mapMenus(sourceMenus.value);
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
      zoom: true,
    },
    treeConfig: {
      parentField: 'pid',
      rowField: 'id',
      transform: false,
    },
  } as VxeTableGridOptions<MenuRow>,
});

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

function mapMenus(list: MenuInfo[]): MenuRow[] {
  return list.map((menu) => {
    const children = Array.isArray(menu.children) ? menu.children : [];
    const type = resolveMenuType(menu);
    const component = resolveComponentText(menu, type);
    return {
      authCode: menu.name,
      children: mapMenus(children),
      component,
      id: String(menu.ID),
      menu,
      meta: {
        icon: menu.meta?.icon,
        title: menu.meta?.title || menu.name,
      },
      name: menu.name,
      path: menu.path,
      pid: String(menu.parentId || ''),
      status: menu.hidden ? '隐藏' : '显示',
      type,
    };
  });
}

function resolveMenuType(menu: MenuInfo): TemplateMenuType {
  if ((menu.menuBtn?.length ?? 0) > 0 && !menu.component && !menu.path) {
    return 'button';
  }
  if (menu.meta?.activeName === 'link') {
    return 'link';
  }
  if (menu.component?.includes('iframe')) {
    return 'embedded';
  }
  if ((menu.children?.length ?? 0) > 0 && !menu.component) {
    return 'catalog';
  }
  return 'menu';
}

function resolveComponentText(menu: MenuInfo, type: TemplateMenuType) {
  if (type === 'link') return menu.path;
  if (type === 'embedded') return menu.component || menu.path;
  if (type === 'button') return '';
  return menu.component;
}

function onRefresh() {
  gridApi.query();
}

function onEdit(row: MenuRow) {
  formDrawerApi.setData({ ...row, parentMenus: sourceMenus.value }).open();
}

function onCreate() {
  formDrawerApi.setData({ parentMenus: sourceMenus.value }).open();
}

function onAppend(row: MenuRow) {
  formDrawerApi.setData({ parentMenus: sourceMenus.value, pid: row.id }).open();
}

function onDelete(row: MenuRow) {
  Modal.confirm({
    content: `确认删除菜单「${row.meta.title}」吗？`,
    onOk: async () => {
      const hideLoading = message.loading({
        content: `正在删除 ${row.meta.title}`,
        duration: 0,
        key: 'menu_delete_msg',
      });
      try {
        await deleteBaseMenuApi({ ID: row.menu.ID });
        message.success({
          content: `已删除 ${row.meta.title}`,
          key: 'menu_delete_msg',
        });
        onRefresh();
      } catch (error) {
        hideLoading();
        message.error(error instanceof Error ? error.message : '删除菜单失败');
      }
    },
    title: '删除菜单',
  });
}

function menuTypeColor(type: TemplateMenuType) {
  return menuTypeOptions.find((item) => item.value === type)?.color ?? 'default';
}

function menuTypeLabel(type: TemplateMenuType) {
  return menuTypeOptions.find((item) => item.value === type)?.label ?? type;
}
</script>

<template>
  <Page auto-content-height>
    <FormDrawer @success="onRefresh" />
    <Grid>
      <template #toolbar-tools>
        <Button type="primary" @click="onCreate">
          <IconifyIcon class="size-5" icon="mdi:plus" />
          新增菜单
        </Button>
      </template>
      <template #title="{ row }">
        <div class="flex w-full items-center gap-1">
          <div class="size-5 shrink-0">
            <IconifyIcon
              v-if="row.type === 'button'"
              class="size-full"
              icon="carbon:security"
            />
            <IconifyIcon
              v-else-if="row.meta?.icon"
              :icon="row.meta?.icon || 'carbon:circle-dash'"
              class="size-full"
            />
          </div>
          <span class="flex-auto">{{ row.meta?.title }}</span>
          <div class="items-center justify-end"></div>
        </div>
      </template>
      <template #type="{ row }">
        <Tag :color="menuTypeColor(row.type)">{{ menuTypeLabel(row.type) }}</Tag>
      </template>
      <template #status="{ row }">
        <Tag :color="row.status === '显示' ? 'success' : 'default'">{{ row.status }}</Tag>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button size="small" type="link" @click="onAppend(row)">
            新增下级
          </Button>
          <Button size="small" type="link" @click="onEdit(row)">
            编辑
          </Button>
          <Button danger size="small" type="link" @click="onDelete(row)">
            删除
          </Button>
        </Space>
      </template>
    </Grid>
  </Page>
</template>

<style lang="scss" scoped>
.menu-badge {
  top: 50%;
  right: 0;
  transform: translateY(-50%);

  & > :deep(div) {
    padding-top: 0;
    padding-bottom: 0;
  }
}
</style>
