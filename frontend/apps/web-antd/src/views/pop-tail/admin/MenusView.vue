<script lang="ts" setup>
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { MenuInfo, MenuUpsertInput } from '#/types/pop-tail';

import type { MenuRow, TemplateMenuType } from './menu-template/types';

import { onMounted, ref, shallowRef } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon, Plus } from '@vben/icons';

import { Button, message, Modal, Skeleton, Space, Switch, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteBaseMenuApi, getMenuTreeApi, saveMenuApi, saveMenusApi } from '#/api/pop-tail/admin';
import { useNavigationStore } from '#/store/pop-tail/navigation';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefresh } from '#/utils/page-refresh';

import { getMenuTypeOptions, useColumns } from './menu-template/data';
import Form from './menu-template/modules/form.vue';

const [FormDrawer, formDrawerApi] = useVbenDrawer({
  connectedComponent: Form,
  destroyOnClose: true,
});

const sourceMenus = shallowRef<MenuInfo[]>([]);
const menuTypeOptions = getMenuTypeOptions();
const { canUse } = useMenuButtonAccess();
const navigationStore = useNavigationStore();
const pageLoading = ref(false);

const [Grid, gridApi] = useVbenVxeGrid<MenuRow>({
  gridOptions: {
    columns: useColumns(),
    height: 'auto',
    keepSource: true,
    pagerConfig: {
      enabled: false,
    },
    proxyConfig: {
      autoLoad: false,
      ajax: {
        query: async () => {
          const items = await loadMenuRows();
          await syncGridRows(items);
          return {
            items,
            total: items.length,
          };
        },
      },
    },
    rowConfig: {
      keyField: 'id',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: false,
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
  return [...list]
    .sort((a, b) => (a.sort ?? 0) - (b.sort ?? 0))
    .map((item) => ({
      ...item,
      children: buildUniqueMenuTree(item.children ?? []),
    }));
}

function mapMenus(list: MenuInfo[], level = 0): MenuRow[] {
  return list.map((menu) => {
    const children = Array.isArray(menu.children) ? menu.children : [];
    const type = resolveMenuType(menu);
    const component = resolveComponentText(menu, type);
    return {
      authCode: menu.name,
      children: mapMenus(children, level + 1),
      component,
      id: String(menu.ID),
      level,
      menu,
      meta: {
        icon: normalizeMenuIcon(menu.meta?.icon),
        title: menu.meta?.title || menu.name,
      },
      name: menu.name,
      path: menu.path,
      pid: String(menu.parentId || ''),
      sort: menu.sort ?? 0,
      status: menu.hidden ? '隐藏' : '显示',
      type,
    };
  });
}

function normalizeMenuIcon(icon?: string) {
  const value = String(icon ?? '').trim();
  if (!value) {
    return '';
  }
  if (value.includes(':')) {
    return value;
  }
  return `lucide:${value}`;
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

async function loadMenuRows(force = false) {
  const result =
    !force && navigationStore.menus.length > 0
      ? navigationStore.menus
      : await getMenuTreeApi();
  navigationStore.menus = result ?? [];
  sourceMenus.value = buildUniqueMenuTree(result ?? []);
  return mapMenus(sourceMenus.value);
}

function showPageSkeleton() {
  pageLoading.value = true;
  void syncGridRows(mapMenus(sourceMenus.value));
}

function hidePageSkeleton() {
  pageLoading.value = false;
}

async function refreshGridData(force = true) {
  // showPageSkeleton();
  try {

    const items = await loadMenuRows(force);
    await syncGridRows(items);
  } catch (error) {
    message.error(error instanceof Error ? error.message : '获取菜单列表失败');
    // hidePageSkeleton();
  } finally {
    // hidePageSkeleton();
  }
}

async function onRefresh() {
  showPageSkeleton();
  try {
    await refreshGridData(true);
  } catch (error) {
    await refreshGridData(true);
    message.error(error instanceof Error ? error.message : '获取菜单列表失败');
  } finally {
    hidePageSkeleton();
  }
}

async function syncGridRows(items = mapMenus(sourceMenus.value)) {
  const grid = gridApi.grid;
  if (typeof grid?.reloadData === 'function') {
    await grid.reloadData(items);
    return;
  }
  if (typeof grid?.loadData === 'function') {
    await grid.loadData(items);
    return;
  }
  gridApi.setGridOptions({
    data: items,
  });
}

onMounted(() => {
  if (navigationStore.menus.length > 0) {
    sourceMenus.value = buildUniqueMenuTree(navigationStore.menus);
    showPageSkeleton();
  }
  void refreshGridData(false).finally(() => {
    hidePageSkeleton();
  });
});

usePageRefresh(onRefresh);

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
      showPageSkeleton();
      try {
        await deleteBaseMenuApi({ ID: row.menu.ID });
        await refreshGridData(true);
        message.success({
          content: `已删除 ${row.meta.title}`,
          key: 'menu_delete_msg',
        });
      } catch (error) {
        await refreshGridData(true);
        hideLoading();
        message.error(error instanceof Error ? error.message : '删除菜单失败');
      } finally {
        hidePageSkeleton();
      }
    },
    title: '删除菜单',
  });
}

function cloneMenuTree(list: MenuInfo[]) {
  return list.map((item) => ({
    ...item,
    children: cloneMenuTree(item.children ?? []),
  }));
}

function buildMenuPayload(menu: MenuInfo): MenuUpsertInput {
  return {
    ID: menu.ID,
    component: menu.component,
    hidden: menu.hidden,
    menuBtn: menu.menuBtn,
    meta: menu.meta,
    name: menu.name,
    parameters: menu.parameters,
    parentId: menu.parentId,
    path: menu.path,
    sort: menu.sort ?? 0,
  };
}

function findSiblingCollection(list: MenuInfo[], parentId: number): MenuInfo[] | null {
  if (parentId === 0) {
    return list;
  }

  for (const item of list) {
    if (item.ID === parentId) {
      return item.children ?? [];
    }
    const nested = findSiblingCollection(item.children ?? [], parentId);
    if (nested) {
      return nested;
    }
  }

  return null;
}

async function persistSiblingOrder(menus: MenuInfo[], parentId: number, changedIds: number[]) {
  const siblings = findSiblingCollection(menus, parentId);
  if (!siblings) {
    throw new Error('未找到需要排序的菜单分组');
  }

  const targets = siblings.filter((menu) => changedIds.includes(menu.ID));
  await saveMenusApi(targets.map((menu) => buildMenuPayload(menu)));
}
// async function movecaidan(){
//   showPageSkeleton();
// }
async function moveMenu(row: MenuRow, direction: 'up' | 'down') {
  showPageSkeleton();
  try {
    const draftMenus = cloneMenuTree(sourceMenus.value);
    const siblings = findSiblingCollection(draftMenus, row.menu.parentId);
    const currentIndex = siblings?.findIndex((item) => item.ID === row.menu.ID) ?? -1;
    if (!siblings || currentIndex < 0) {
      throw new Error('未找到需要移动的菜单');
    }

    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
    if (targetIndex < 0 || targetIndex >= siblings.length) {
      hidePageSkeleton();
      return;
    }

    const current = siblings[currentIndex];
    const target = siblings[targetIndex];
    if (!current || !target) {
      throw new Error('未找到目标菜单');
    }

    const currentSort = current.sort ?? 0;
    current.sort = target.sort ?? 0;
    target.sort = currentSort;

    sourceMenus.value = buildUniqueMenuTree(draftMenus);
    await persistSiblingOrder(draftMenus, row.menu.parentId, [current.ID, target.ID]);
    await refreshGridData(true);
    message.success(`${row.meta.title}${direction === 'up' ? ' 已上移' : ' 已下移'}`);
  } catch (error) {
    await refreshGridData(true);
    message.error(error instanceof Error ? error.message : '更新菜单排序失败');
  } finally {
    hidePageSkeleton();
  }
}

function menuTypeColor(type: TemplateMenuType) {
  return menuTypeOptions.find((item) => item.value === type)?.color ?? 'default';
}

function menuTypeLabel(type: TemplateMenuType) {
  return menuTypeOptions.find((item) => item.value === type)?.label ?? type;
}

async function toggleHidden(row: MenuRow, visible: boolean) {
  showPageSkeleton();
  try {
    await saveMenuApi({
      ...buildMenuPayload(row.menu),
      hidden: !visible,
    });
    await refreshGridData(true);
    message.success(`${row.meta.title}${visible ? ' 已显示' : ' 已隐藏'}`);
  } catch (error) {
    await refreshGridData(true);
    message.error(error instanceof Error ? error.message : '更新显示状态失败');
  } finally {
    hidePageSkeleton();
  }
}
</script>

<template>
  <Page auto-content-height>
    <FormDrawer @success="onRefresh" />
    <div class="menu-grid-shell">
      <div class="menu-grid-shell__content">
        <Grid class="menu-grid-shell__grid">
          <template #toolbar-tools>
            <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" type="primary" @click="onCreate">
              <Plus class="size-5" />
              新增菜单
            </Button>
          </template>
          <template #title="{ row }">
            <div v-if="pageLoading" class="menu-cell-skeleton menu-cell-skeleton--title">
              <Skeleton active :paragraph="false" :title="{ width: `${42 + (Number(row.sort ?? 0) % 4) * 10}%` }" />
            </div>
            <div v-else class="flex w-full items-center gap-1">
              <div class="flex size-4 shrink-0 items-center justify-center">
                <IconifyIcon v-if="row.type === 'button'" class="size-3.5" icon="carbon:security" />
                <IconifyIcon v-else-if="row.meta?.icon" :icon="row.meta?.icon || 'carbon:circle-dash'"
                  class="size-3.5" />
              </div>
              <span class="flex-auto">{{ row.meta?.title }}</span>
              <div class="items-center justify-end"></div>
            </div>
          </template>
          <template #type="{ row }">
            <Skeleton v-if="pageLoading" active :paragraph="false" :title="{ width: '70px' }" />
            <Tag v-else :color="menuTypeColor(row.type)">{{ menuTypeLabel(row.type) }}</Tag>
          </template>
          <template #authCode="{ row }">
            <Skeleton v-if="pageLoading" active :paragraph="false"
              :title="{ width: `${48 + (Number(row.sort ?? 0) % 3) * 14}%` }" />
            <span v-else>{{ row.authCode }}</span>
          </template>
          <template #path="{ row }">
            <Skeleton v-if="pageLoading" active :paragraph="false"
              :title="{ width: `${56 + (Number(row.sort ?? 0) % 3) * 10}%` }" />
            <span v-else>{{ row.path }}</span>
          </template>
          <template #status="{ row }">
            <div v-if="pageLoading" class="menu-cell-skeleton">
              <Skeleton active :paragraph="false" :title="{ width: '88px' }" />
            </div>
            <div v-else class="flex items-center justify-center gap-2">
              <Switch :checked="row.status === '显示'" :disabled="!canUse(STANDARD_BUTTON_LABELS.edit)" size="small"
                @change="(checked) => toggleHidden(row, Boolean(checked))" />
              <Tag :color="row.status === '显示' ? 'success' : 'default'">{{ row.status }}</Tag>
            </div>
          </template>
          <template #sort="{ row }">
            <div v-if="pageLoading" class="menu-cell-skeleton">
              <Skeleton active :paragraph="false" :title="{ width: '140px' }" />
            </div>
            <div v-else class="flex items-center justify-center gap-2">
              <span class="min-w-10 text-center font-medium">{{ row.sort }}</span>
              <Button size="small" @click="moveMenu(row, 'up')">上移</Button>
              <Button size="small" @click="moveMenu(row, 'down')">下移</Button>
            </div>
          </template>
          <template #component="{ row }">
            <Skeleton v-if="pageLoading" active :paragraph="false"
              :title="{ width: `${52 + (Number(row.sort ?? 0) % 4) * 9}%` }" />
            <span v-else>{{ row.component }}</span>
          </template>
          <template #operation="{ row }">
            <div v-if="pageLoading" class="menu-cell-skeleton">
              <Skeleton active :paragraph="false" :title="{ width: '180px' }" />
            </div>
            <Space v-else size="small">
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.create)" size="small" type="link" @click="onAppend(row)">
                新增下级
              </Button>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" size="small" type="link" @click="onEdit(row)">
                编辑
              </Button>
              <Button v-if="canUse(STANDARD_BUTTON_LABELS.delete)" danger size="small" type="link"
                @click="onDelete(row)">
                删除
              </Button>
            </Space>
          </template>
        </Grid>
      </div>
    </div>
  </Page>
</template>

<style lang="scss" scoped>
.menu-badge {
  top: 50%;
  right: 0;
  transform: translateY(-50%);

  &> :deep(div) {
    padding-top: 0;
    padding-bottom: 0;
  }
}

.menu-grid-shell {
  position: relative;
  display: flex;
  min-height: 0;
  height: 100%;
  flex: 1;
  flex-direction: column;
}

.menu-grid-shell__content {
  min-height: 0;
  height: 100%;
  flex: 1;
}

.menu-grid-shell__grid {
  min-height: 0;
  height: 100%;
}

.menu-cell-skeleton {
  display: flex;
  align-items: center;
  min-height: 32px;
}

.menu-cell-skeleton--title {
  padding-left: 2px;
}

.menu-cell-skeleton :deep(.ant-skeleton) {
  width: 100%;
}

.menu-cell-skeleton :deep(.ant-skeleton-title) {
  margin-block-start: 0;
  margin-block-end: 0;
}
</style>
