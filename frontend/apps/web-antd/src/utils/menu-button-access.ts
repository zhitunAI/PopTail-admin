import { computed } from 'vue';
import { useRoute } from 'vue-router';

import type { MenuButtonInfo, MenuInfo } from '#/types/pop-tail';

import { useAccess } from '@vben/access';

import { normalizeAuthRoutePath } from '#/store/pop-tail/auth';
import { useNavigationStore } from '#/store/pop-tail/navigation';

function flattenMenus(list: MenuInfo[]): MenuInfo[] {
  return list.flatMap((item) => [item, ...flattenMenus(item.children ?? [])]);
}

function findMenuByPath(list: MenuInfo[], path: string) {
  const normalized = normalizeAuthRoutePath(path);
  return flattenMenus(list).find(
    (item) => normalizeAuthRoutePath(item.path || '') === normalized,
  );
}

function normalizeLabels(labels: string[]) {
  return labels.map((item) => item.trim()).filter(Boolean);
}

export const STANDARD_BUTTON_LABELS = {
  create: ['新增', '创建', 'add', 'create'],
  delete: ['删除', 'delete', 'remove'],
  edit: ['编辑', '修改', 'edit', 'update'],
  view: ['查看', '详情', 'view', 'detail'],
};

export function useMenuButtonAccess() {
  const route = useRoute();
  const navigationStore = useNavigationStore();
  const { hasAccessByCodes } = useAccess();

  const currentMenu = computed(() => {
    const activePath =
      typeof route.meta?.activePath === 'string' ? route.meta.activePath : route.path;
    return findMenuByPath(navigationStore.menus, activePath);
  });

  const configuredButtons = computed<MenuButtonInfo[]>(
    () => currentMenu.value?.menuBtn ?? [],
  );

  function canUse(labels: string[]) {
    const normalizedLabels = normalizeLabels(labels);
    if (normalizedLabels.length === 0) {
      return true;
    }

    const activePath =
      typeof route.meta?.activePath === 'string' ? route.meta.activePath : route.path;
    const currentPath = normalizeAuthRoutePath(activePath || '');
    const configured = configuredButtons.value.filter((item) =>
      normalizedLabels.includes(String(item.name || '').trim()),
    );

    if (configured.length === 0) {
      return true;
    }

    const codes = configured.flatMap((item) => {
      const name = String(item.name || '').trim();
      return [`btn:${currentPath}:${name}`, `btn:${name}`, name];
    });

    return hasAccessByCodes(codes);
  }

  return {
    canUse,
    configuredButtons,
    currentMenu,
  };
}
