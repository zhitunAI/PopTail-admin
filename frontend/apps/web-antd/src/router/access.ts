import type {
  ComponentRecordType,
  GenerateMenuAndRoutesOptions,
  RouteRecordRaw,
  RouteRecordStringComponent,
} from '@vben/types';

import { generateAccessible } from '@vben/access';
import { preferences } from '@vben/preferences';

import { message } from 'ant-design-vue';

import { apiClient } from '#/api/gin-ai-admin/client';
import { BasicLayout, IFrameView } from '#/layouts';
import { $t } from '#/locales';
import {
  normalizeAuthRoutePath,
  normalizeDefaultRouteName,
} from '#/store/gin-ai-admin/auth';

const forbiddenComponent = () => import('../views/_core/fallback/forbidden.vue');
let hasWarnedAboutLegacyMenuFallback = false;

type LegacyContractRoute = RouteRecordStringComponent & {
  btns?: Record<string, unknown>;
  defaultMenu?: boolean;
  hidden?: boolean;
  keepAlive?: boolean;
  meta?: Record<string, unknown> & {
    btns?: Record<string, unknown>;
    defaultMenu?: boolean;
    hidden?: boolean;
    keepAlive?: boolean;
  };
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object' && !Array.isArray(value);
}

function joinRoutePath(parentPath: string, currentPath: string) {
  if (!currentPath) {
    return normalizeAuthRoutePath(parentPath);
  }
  if (/^(https?:)?\/\//.test(currentPath)) {
    return currentPath;
  }
  if (currentPath.startsWith('/')) {
    return normalizeAuthRoutePath(currentPath);
  }
  return normalizeAuthRoutePath(`${parentPath}/${currentPath}`);
}

function buildLegacyContractIndex(routes: LegacyContractRoute[]) {
  const index = new Map<string, LegacyContractRoute>();

  function register(key: string, route: LegacyContractRoute) {
    if (!key || index.has(key)) {
      return;
    }
    index.set(key, route);
  }

  function visit(routeList: LegacyContractRoute[], parentPath = '') {
    for (const route of routeList) {
      const name = typeof route.name === 'string' ? route.name : '';
      const normalizedName = normalizeDefaultRouteName(name);
      const normalizedPath = joinRoutePath(parentPath, String(route.path ?? ''));

      register(`name:${name}`, route);
      register(`name:${normalizedName}`, route);
      register(`path:${normalizedPath}`, route);

      if (route.children?.length) {
        visit(route.children as LegacyContractRoute[], normalizedPath);
      }
    }
  }

  visit(routes);
  return index;
}

function findLegacyContract(
  route: RouteRecordRaw,
  contractIndex: Map<string, LegacyContractRoute>,
) {
  const routeName = typeof route.name === 'string' ? route.name : '';
  const normalizedName = normalizeDefaultRouteName(routeName);
  const normalizedPath = normalizeAuthRoutePath(route.path);

  return (
    contractIndex.get(`path:${normalizedPath}`) ??
    contractIndex.get(`name:${routeName}`) ??
    contractIndex.get(`name:${normalizedName}`)
  );
}

function mergeRouteContract(
  route: RouteRecordRaw,
  contract?: LegacyContractRoute,
): RouteRecordRaw {
  if (!contract) {
    return route;
  }

  const meta = ({ ...route.meta } as NonNullable<RouteRecordRaw['meta']>);
  const contractMeta = isRecord(contract.meta) ? contract.meta : undefined;
  let btns: Record<string, unknown> | undefined;
  if (isRecord(contract.btns)) {
    btns = contract.btns;
  } else if (isRecord(contractMeta?.btns)) {
    btns = contractMeta.btns;
  }
  const keepAlive =
    contract.keepAlive ??
    (typeof contractMeta?.keepAlive === 'boolean' ? contractMeta.keepAlive : undefined);
  const defaultMenu =
    contract.defaultMenu ??
    (typeof contractMeta?.defaultMenu === 'boolean'
      ? contractMeta.defaultMenu
      : undefined);
  const hidden =
    contract.hidden ??
    (typeof contractMeta?.hidden === 'boolean' ? contractMeta.hidden : undefined);

  if (btns) {
    (meta as Record<string, unknown>).btns = btns;
  }
  if (keepAlive !== undefined) {
    meta.keepAlive = keepAlive;
  }
  if (defaultMenu !== undefined) {
    (meta as Record<string, unknown>).defaultMenu = defaultMenu;
  }
  if (hidden) {
    meta.hideInMenu = true;
  }

  return {
    ...route,
    meta,
  } as RouteRecordRaw;
}

function shouldKeepRoute(route: RouteRecordRaw, matched: boolean) {
  if (matched) {
    return true;
  }

  if (route.meta?.ignoreAccess) {
    return true;
  }

  if (route.meta?.hideInMenu || route.meta?.hideInTab || route.meta?.hideInBreadcrumb) {
    return true;
  }

  return typeof route.name === 'string' && route.name.startsWith('Legacy');
}

function applyLegacyContract(
  routes: RouteRecordRaw[],
  contractRoutes: LegacyContractRoute[] | null,
): RouteRecordRaw[] {
  if (!contractRoutes?.length) {
    return routes;
  }

  const contractIndex = buildLegacyContractIndex(contractRoutes);

  function visit(route: RouteRecordRaw): null | RouteRecordRaw {
    const matchedContract = findLegacyContract(route, contractIndex);
    const nextChildren = route.children
      ?.map((child) => visit(child))
      .filter((child): child is RouteRecordRaw => child !== null);

    const mergedRoute = mergeRouteContract(
      (nextChildren ? { ...route, children: nextChildren } : route) as RouteRecordRaw,
      matchedContract,
    );

    if (nextChildren) {
      mergedRoute.children = nextChildren;
    }

    if (nextChildren?.length) {
      return mergedRoute;
    }

    return shouldKeepRoute(mergedRoute, Boolean(matchedContract))
      ? mergedRoute
      : null;
  }

  return routes
    .map((route) => visit(route))
    .filter((route): route is RouteRecordRaw => route !== null);
}

async function fetchLegacyContractRoutes() {
  const unwrapRoutes = (payload: unknown): LegacyContractRoute[] | null => {
    if (Array.isArray(payload)) {
      return payload as LegacyContractRoute[];
    }
    if (!payload || typeof payload !== 'object') {
      return null;
    }
    const wrappedData = (payload as { data?: unknown }).data;
    if (Array.isArray(wrappedData)) {
      return wrappedData as LegacyContractRoute[];
    }
    if (wrappedData && typeof wrappedData === 'object') {
      const menus = (wrappedData as { menus?: unknown }).menus;
      if (Array.isArray(menus)) {
        return menus as LegacyContractRoute[];
      }
    }
    return null;
  };

  const tryFetchRoutes = async (
    request: () => Promise<{ data: unknown }>,
  ): Promise<LegacyContractRoute[] | null> => {
    try {
      const response = await request();
      const routes = unwrapRoutes(response.data);
      return routes?.length ? routes : null;
    } catch {
      return null;
    }
  };

  try {
    const preferredRoutes = await tryFetchRoutes(() =>
      apiClient.post('/menu/getMenu', {}),
    );
    if (preferredRoutes) {
      return preferredRoutes;
    }

    const legacyRoutes = await tryFetchRoutes(() => apiClient.get('/menu/all'));
    if (legacyRoutes) {
      return legacyRoutes;
    }

    return null;
  } catch (error) {
    if (!hasWarnedAboutLegacyMenuFallback) {
      console.warn(
        '[parity] fallback to static access routes because menu contract endpoints failed',
        error,
      );
      hasWarnedAboutLegacyMenuFallback = true;
    }
    return null;
  }
}

async function generateAccess(options: GenerateMenuAndRoutesOptions) {
  const pageMap: ComponentRecordType = import.meta.glob('../views/**/*.vue');

  const layoutMap: ComponentRecordType = {
    BasicLayout,
    IFrameView,
  };

  const legacyContractRoutes = await fetchLegacyContractRoutes();
  const parityRoutes = applyLegacyContract(options.routes, legacyContractRoutes);

  return await generateAccessible(preferences.app.accessMode, {
    ...options,
    fetchMenuListAsync: async () => {
      message.loading({
        content: `${$t('common.loadingMenu')}...`,
        duration: 1.5,
      });
      return legacyContractRoutes ?? [];
    },
    forbiddenComponent,
    layoutMap,
    pageMap,
    routes: parityRoutes,
  });
}

export { generateAccess };
