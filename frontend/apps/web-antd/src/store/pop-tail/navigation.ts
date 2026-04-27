import type { RouteRecordRaw } from 'vue-router';

import type { MenuRecordRaw } from '@vben/types';

import { defineStore } from 'pinia';

import { getMenuTreeApi } from '#/api/pop-tail/admin';
import type { MenuInfo } from '#/types/pop-tail';

import { normalizeAuthRoutePath } from './auth';

interface NavigationState {
  menus: MenuInfo[];
}

function flattenRoutes(routes: RouteRecordRaw[]): RouteRecordRaw[] {
  return routes.flatMap((route) => [route, ...(route.children ? flattenRoutes(route.children) : [])]);
}

function sortMenus(items: MenuInfo[]) {
  return [...items].sort((left, right) => (left.sort ?? 999) - (right.sort ?? 999));
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

function buildRouteMetaMap(routes: RouteRecordRaw[]) {
  const map = new Map<string, RouteRecordRaw>();
  for (const route of flattenRoutes(routes)) {
    const normalizedPath = normalizeAuthRoutePath(String(route.path ?? ''));
    if (!normalizedPath) {
      continue;
    }
    map.set(normalizedPath, route);
  }
  return map;
}

function transformMenuTree(
  items: MenuInfo[],
  routeMetaMap: Map<string, RouteRecordRaw>,
  parents: string[] = [],
): MenuRecordRaw[] {
  return sortMenus(items)
    .map((item): MenuRecordRaw | null => {
      const path = normalizeAuthRoutePath(item.path || '');
      const route = routeMetaMap.get(path);
      const children = transformMenuTree(item.children || [], routeMetaMap, [...parents, path]);
      const show = !item.hidden;
      const name = item.meta?.title || item.name;
      const icon = normalizeMenuIcon(item.meta?.icon) || normalizeMenuIcon(String(route?.meta?.icon ?? ''));

      if (!show) {
        return null;
      }

      if (!route && children.length === 0) {
        return null;
      }

      return {
        activeIcon: route?.meta?.activeIcon,
        badge: route?.meta?.badge,
        badgeType: route?.meta?.badgeType,
        badgeVariants: route?.meta?.badgeVariants,
        children,
        icon,
        name,
        order: item.sort,
        parent: parents.at(-1),
        parents,
        path,
        query: route?.meta?.query,
        show,
      };
    })
    .filter((item): item is MenuRecordRaw => item !== null);
}

export async function loadBackendAccessMenus(accessRoutes: RouteRecordRaw[]) {
  const navigationStore = useNavigationStore();
  const menus = navigationStore.menus.length > 0
    ? navigationStore.menus
    : await getMenuTreeApi();
  navigationStore.menus = menus;
  return transformMenuTree(menus, buildRouteMetaMap(accessRoutes));
}

export const useNavigationStore = defineStore('navigation', {
  state: (): NavigationState => ({
    menus: [],
  }),
  actions: {
    async loadMenus() {
      this.menus = this.menus.length > 0 ? this.menus : await getMenuTreeApi();
      return this.menus;
    },
    async loadAccessMenus(accessRoutes: RouteRecordRaw[]) {
      this.menus = this.menus.length > 0 ? this.menus : await getMenuTreeApi();
      return transformMenuTree(this.menus, buildRouteMetaMap(accessRoutes));
    },
    clear() {
      this.menus = [];
    },
  },
});
