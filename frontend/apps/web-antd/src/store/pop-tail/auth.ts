import type { RouteRecordRaw } from 'vue-router';

import type { UserInfo as VbenUserInfo } from '@vben/types';

import type {
  AuthorityButtonMatrixBatchSelectionItem,
  MenuInfo,
  UserInfo as LegacyUserInfo,
  PolicyPath,
} from '#/types/pop-tail';

import { computed, ref } from 'vue';

import { preferences } from '@vben/preferences';
import { useAccessStore, useUserStore } from '@vben/stores';

import { defineStore } from 'pinia';

import { getAuthorityBtnsApi, getMenuTreeApi } from '#/api/pop-tail/admin';
import { useNavigationStore } from '#/store/pop-tail/navigation';
import {
  getBootstrapApi,
  getPolicyPathByAuthorityId,
  refreshApi,
  getUserInfoApi,
  loginApi,
  logoutApi,
  setUserAuthorityApi,
} from '#/api/pop-tail/auth';
import {
  clearClientAuthState,
  getStoredTokenExpireAt,
  getStoredToken,
  isTokenExpired,
  readAuthSyncEvent,
} from '#/api/pop-tail/client';

const DEFAULT_HOME_PATH = '/dashboard';
const REFRESH_AHEAD_MS = 5 * 60 * 1000;
const REFRESH_CHECK_INTERVAL_MS = 60 * 1000;

const LEGACY_ROUTE_NAME_TO_PATH: Record<string, string> = {
  about: '/about',
  aiWorkflow: '/system/tools/ai-workflow',
  anInfo: '/system/tools/announcement',
  api: '/system/apis',
  authority: '/system/authorities',
  apiToken: '/system/tools/api-tokens',
  breakpoint: '/examples/breakpoint',
  customer: '/examples/customer',
  dashboard: '/dashboard',
  dictionary: '/system/dictionaries',
  example: '/examples',
  init: '/init',
  loginLog: '/system/login-logs',
  menu: '/system/menus',
  operation: '/system/operation-logs',
  person: '/profile',
  plugin: '/system/tools',
  'plugin-email': '/system/tools/plugin-email',
  scanUpload: '/scan-upload',
  skills: '/system/tools/skills',
  state: '/system/state',
  superAdmin: '/system/overview',
  system: '/system/tools/config',
  systemTools: '/system/tools',
  sysParams: '/system/params',
  upload: '/examples/upload',
  user: '/system/users',
};

const LEGACY_ROUTE_NAME_TO_CANONICAL_NAME: Record<string, string> = {
  about: 'about',
  aiWorkflow: 'aiWorkflow',
  anInfo: 'announcementInfo',
  api: 'apis',
  authority: 'authorities',
  apiToken: 'apiTokens',
  breakpoint: 'breakpointExample',
  customer: 'customerExample',
  dashboard: 'dashboard',
  dictionary: 'dictionaries',
  example: 'examplesRoot',
  init: 'init',
  loginLog: 'loginLogs',
  menu: 'menus',
  operation: 'operationLogs',
  person: 'profile',
  plugin: 'systemTools',
  'plugin-email': 'pluginEmail',
  scanUpload: 'scanUpload',
  skills: 'skills',
  state: 'systemState',
  superAdmin: 'systemOverview',
  system: 'systemConfig',
  systemTools: 'systemTools',
  sysParams: 'params',
  upload: 'uploadExample',
  user: 'users',
};

const LEGACY_PATH_ALIAS_TO_CANONICAL_PATH: Record<string, string> = {
  '/admin': '/system/overview',
  '/admin/api': '/system/apis',
  '/admin/apiToken': '/system/tools/api-tokens',
  '/admin/authority': '/system/authorities',
  '/admin/dictionary': '/system/dictionaries',
  '/admin/loginLog': '/system/login-logs',
  '/admin/menu': '/system/menus',
  '/admin/operation': '/system/operation-logs',
  '/admin/sysParams': '/system/params',
  '/admin/system': '/system/tools/config',
  '/admin/user': '/system/users',
  '/anInfo': '/system/tools/announcement',
  '/example': '/examples',
  '/example/breakpoint': '/examples/breakpoint',
  '/example/customer': '/examples/customer',
  '/example/upload': '/examples/upload',
  '/person': '/profile',
  '/plugin': '/system/tools',
  '/plugin-email': '/system/tools/plugin-email',
  '/scanUpload': '/scan-upload',
  '/state': '/system/state',
  '/systemTools': '/system/tools',
  '/systemTools/aiWorkflow': '/system/tools/ai-workflow',
  '/systemTools/skills': '/system/tools/skills',
};

const LOGIN_PATH = '/auth/login';

function ensureLeadingSlash(path: string) {
  if (!path) {
    return DEFAULT_HOME_PATH;
  }
  const normalized = path.startsWith('/') ? path : `/${path}`;
  const segments = normalized.split('/').filter(Boolean);
  return `/${segments.join('/')}`;
}

export function normalizeDefaultRouteName(routeName?: string) {
  if (!routeName) {
    return 'dashboard';
  }
  return LEGACY_ROUTE_NAME_TO_CANONICAL_NAME[routeName] ?? routeName;
}

export function normalizeAuthRoutePath(input?: string) {
  if (!input) {
    return DEFAULT_HOME_PATH;
  }

  const routeName = input.replaceAll(/^\/+|\/+$/g, '');
  if (LEGACY_ROUTE_NAME_TO_PATH[routeName]) {
    return LEGACY_ROUTE_NAME_TO_PATH[routeName] ?? DEFAULT_HOME_PATH;
  }

  const withSlash = ensureLeadingSlash(input);
  if (LEGACY_PATH_ALIAS_TO_CANONICAL_PATH[withSlash]) {
    return LEGACY_PATH_ALIAS_TO_CANONICAL_PATH[withSlash] ?? DEFAULT_HOME_PATH;
  }

  return withSlash;
}

export function isAuthenticationPath(path?: string) {
  const normalized = normalizeAuthRoutePath(path);
  return normalized === '/auth/login' || normalized.startsWith('/auth/');
}

function normalizeHomePath(path?: string) {
  return normalizeAuthRoutePath(path);
}

function buildLoginRedirectPath(targetPath?: string) {
  const normalizedTarget = normalizeAuthRoutePath(targetPath);
  if (
    !normalizedTarget ||
    normalizedTarget === LOGIN_PATH ||
    isAuthenticationPath(normalizedTarget)
  ) {
    return LOGIN_PATH;
  }
  return `${LOGIN_PATH}?redirect=${encodeURIComponent(normalizedTarget)}`;
}

export function buildPolicyAccessCodes(policyPaths: PolicyPath[]) {
  const codes = new Set<string>();

  for (const policy of policyPaths) {
    const path = ensureLeadingSlash(String(policy.path || '').trim());
    const method = String(policy.method || '').trim().toUpperCase();
    if (!path || path === '/') {
      continue;
    }

    codes.add(path);
    if (method) {
      codes.add(`${method}:${path}`);
      codes.add(`${path}:${method}`);
      codes.add(`api:${method}:${path}`);
    }
  }

  return [...codes];
}

export function collectButtonAccessCodesFromRoutes(routes: RouteRecordRaw[]) {
  const codes = new Set<string>();

  function visit(routeList: RouteRecordRaw[]) {
    for (const route of routeList) {
      const btns = (route.meta as Record<string, unknown> | undefined)?.btns;
      const routeName = typeof route.name === 'string' ? route.name : '';
      const routePath = normalizeAuthRoutePath(route.path);

      if (btns && typeof btns === 'object') {
        for (const btnName of Object.keys(btns)) {
          if (!btnName) {
            continue;
          }
          codes.add(btnName);
          codes.add(`btn:${btnName}`);
          if (routeName) {
            codes.add(`btn:${routeName}:${btnName}`);
          }
          if (routePath) {
            codes.add(`btn:${routePath}:${btnName}`);
          }
        }
      }

      if (route.children?.length) {
        visit(route.children);
      }
    }
  }

  visit(routes);
  return [...codes];
}

function flattenMenuTree(items: Array<{ children?: any[] }>) {
  return items.flatMap((item) => [item, ...flattenMenuTree(item.children ?? [])]);
}

async function collectButtonAccessCodesFromServer(authorityId: number, menus: MenuInfo[]) {
  const buttonMenus = flattenMenuTree(menus).filter(
    (item) => Array.isArray(item.menuBtn) && item.menuBtn.length > 0 && item.path,
  );

  if (buttonMenus.length === 0) {
    return [];
  }

  const selections = await getAuthorityBtnsApi({
    authorityId,
    menuIDs: buttonMenus.map((menu) => menu.ID),
  });
  const selectedMap = new Map(
    selections.map((item) => [item.menuID, new Set(item.selected ?? [])] as const),
  );
  const codes = new Set<string>();
  for (const menu of buttonMenus) {
    const path = ensureLeadingSlash(String(menu.path || '').trim());
    if (!path || path === '/') {
      continue;
    }
    const selected = selectedMap.get(menu.ID) ?? new Set<number>();
    for (const button of menu.menuBtn ?? []) {
      const buttonId = Number(button.ID ?? 0);
      if (!buttonId || !selected.has(buttonId)) {
        continue;
      }
      const name = String(button.name || '').trim();
      if (!name) {
        continue;
      }
      codes.add(`btn:${path}:${name}`);
      codes.add(name);
      codes.add(`btn:${name}`);
    }
  }

  return [...codes];
}

function collectButtonAccessCodesFromSelections(
  menus: MenuInfo[],
  selections: AuthorityButtonMatrixBatchSelectionItem[],
) {
  const buttonMenus = flattenMenuTree(menus).filter(
    (item) => Array.isArray(item.menuBtn) && item.menuBtn.length > 0 && item.path,
  );
  const selectedMap = new Map(
    selections.map((item) => [item.menuID, new Set(item.selected ?? [])] as const),
  );
  const codes = new Set<string>();

  for (const menu of buttonMenus) {
    const path = ensureLeadingSlash(String(menu.path || '').trim());
    if (!path || path === '/') {
      continue;
    }
    const selected = selectedMap.get(menu.ID) ?? new Set<number>();
    for (const button of menu.menuBtn ?? []) {
      const buttonId = Number(button.ID ?? 0);
      if (!buttonId || !selected.has(buttonId)) {
        continue;
      }
      const name = String(button.name || '').trim();
      if (!name) {
        continue;
      }
      codes.add(`btn:${path}:${name}`);
      codes.add(name);
      codes.add(`btn:${name}`);
    }
  }

  return [...codes];
}

function toVbenUserInfo(
  user: LegacyUserInfo,
  token: string,
): LegacyUserInfo & VbenUserInfo {
  return {
    ...user,
    avatar: user.headerImg || preferences.app.defaultAvatar,
    desc: user.email || user.phone || user.authority?.authorityName || 'PopTail-admin User',
    homePath: normalizeHomePath(user.authority?.defaultRouter),
    realName: user.nickName || user.userName,
    roles: [
      `authority:${user.authorityId}`,
      user.authority?.authorityName || `authority-${user.authorityId}`,
    ],
    token,
    userId: String(user.ID),
    username: user.userName,
  };
}

export const useAuthStore = defineStore('pop-tail-auth', () => {
  const accessStore = useAccessStore();
  const navigationStore = useNavigationStore();
  const userStore = useUserStore();
  let refreshTimer: null | ReturnType<typeof setInterval> = null;
  let authSyncHandler: null | ((event: StorageEvent) => void) = null;
  let visibilityHandler: null | (() => void) = null;
  let focusHandler: null | (() => void) = null;

  const bootstrapped = ref(false);
  const loginLoading = ref(false);
  const policyPaths = ref<PolicyPath[]>([]);

  const userInfo = computed(() =>
    (userStore.userInfo as LegacyUserInfo & VbenUserInfo | null) ?? null,
  );
  const token = computed(() => accessStore.accessToken || getStoredToken() || '');
  const isLoggedIn = computed(() => Boolean(token.value || userInfo.value));
  const defaultRouterName = computed(() =>
    normalizeDefaultRouteName(userInfo.value?.authority?.defaultRouter),
  );

  function resetAccessContext() {
    accessStore.setAccessCodes([]);
    accessStore.setAccessMenus([]);
    accessStore.setAccessRoutes([]);
    accessStore.setIsAccessChecked(false);
  }

  async function ensureFreshSession(force = false) {
    const currentToken = getStoredToken() || accessStore.accessToken || '';
    if (!currentToken) {
      return null;
    }

    const expiresAt = getStoredTokenExpireAt();
    const remainingMs = expiresAt > 0 ? expiresAt - Date.now() : Number.MAX_SAFE_INTEGER;
    if (!force && remainingMs > REFRESH_AHEAD_MS) {
      return currentToken;
    }

    const refreshed = await refreshApi();
    accessStore.setAccessToken(refreshed.token);
    return refreshed.token;
  }

  function stopSessionKeepAlive() {
    if (refreshTimer) {
      clearInterval(refreshTimer);
      refreshTimer = null;
    }
    if (typeof window === 'undefined') {
      return;
    }
    if (visibilityHandler) {
      document.removeEventListener('visibilitychange', visibilityHandler);
      visibilityHandler = null;
    }
    if (focusHandler) {
      window.removeEventListener('focus', focusHandler);
      focusHandler = null;
    }
  }

  function stopAuthSync() {
    if (typeof window === 'undefined' || !authSyncHandler) {
      return;
    }
    window.removeEventListener('storage', authSyncHandler);
    authSyncHandler = null;
  }

  function startAuthSync() {
    if (typeof window === 'undefined' || authSyncHandler) {
      return;
    }

    authSyncHandler = (event: StorageEvent) => {
      if (event.key !== 'pop-tail-auth-event') {
        return;
      }
      const syncEvent = readAuthSyncEvent(event.newValue);
      if (!syncEvent) {
        return;
      }

      if (syncEvent.type === 'signed-out') {
        stopSessionKeepAlive();
        clearClientAuthState({
          loginExpired: Boolean(syncEvent.loginExpired),
          publish: false,
        });
        policyPaths.value = [];
        bootstrapped.value = true;
        const currentPath = `${window.location.pathname}${window.location.search}${window.location.hash}`;
        const nextLoginPath = buildLoginRedirectPath(currentPath);
        if (window.location.pathname !== LOGIN_PATH) {
          window.location.replace(nextLoginPath);
        }
        return;
      }

      const currentToken = getStoredToken();
      if (!currentToken || isTokenExpired(currentToken)) {
        return;
      }

      accessStore.setLoginExpired(false);
      if (syncEvent.type === 'signed-in' || syncEvent.type === 'token-refreshed') {
        void hydrateAccessEnvelope().catch(() => undefined);
      }
    };

    window.addEventListener('storage', authSyncHandler);
  }

  function startSessionKeepAlive() {
    stopSessionKeepAlive();
    if (typeof window === 'undefined') {
      return;
    }
    const currentToken = getStoredToken() || accessStore.accessToken || '';
    if (!currentToken) {
      return;
    }

    refreshTimer = setInterval(() => {
      void ensureFreshSession().catch(() => undefined);
    }, REFRESH_CHECK_INTERVAL_MS);

    visibilityHandler = () => {
      if (document.visibilityState === 'visible') {
        void ensureFreshSession().catch(() => undefined);
      }
    };
    focusHandler = () => {
      void ensureFreshSession().catch(() => undefined);
    };

    document.addEventListener('visibilitychange', visibilityHandler);
    window.addEventListener('focus', focusHandler);
  }

  async function hydrateAccessEnvelope(user?: LegacyUserInfo) {
    const currentToken = getStoredToken() || accessStore.accessToken || '';
    if (currentToken && isTokenExpired(currentToken)) {
      clear();
      throw new Error('登录状态已过期，请重新登录');
    }
    accessStore.setAccessToken(currentToken || null);
    accessStore.setLoginExpired(false);
    const bootstrapData = await getBootstrapApi().catch(() => null);
    const effectiveUser = user ?? bootstrapData?.userInfo ?? (await getUserInfoApi());
    userStore.setUserInfo(toVbenUserInfo(effectiveUser, currentToken));
    policyPaths.value = bootstrapData?.policyPaths
      ?? await getPolicyPathByAuthorityId(effectiveUser.authorityId);
    const menus = bootstrapData?.menus ?? await getMenuTreeApi().catch(() => []);
    navigationStore.menus = menus;
    const buttonCodes = bootstrapData
      ? collectButtonAccessCodesFromSelections(menus, bootstrapData.authorityButtons ?? [])
      : await collectButtonAccessCodesFromServer(
          effectiveUser.authorityId,
          menus,
        ).catch(() => []);
    accessStore.setAccessCodes([
      ...new Set([...buildPolicyAccessCodes(policyPaths.value), ...buttonCodes]),
    ]);
    bootstrapped.value = true;
    startAuthSync();
    startSessionKeepAlive();
    return userStore.userInfo as LegacyUserInfo & VbenUserInfo;
  }

  async function login(username: string, password: string) {
    loginLoading.value = true;
    try {
      const data = await loginApi(username, password);
      accessStore.setAccessToken(null);
      resetAccessContext();
      await hydrateAccessEnvelope(data.user);
      return userStore.userInfo as LegacyUserInfo & VbenUserInfo;
    } finally {
      loginLoading.value = false;
    }
  }

  async function bootstrap() {
    const persistedToken = getStoredToken() || accessStore.accessToken || '';
    if (persistedToken) {
      if (isTokenExpired(persistedToken)) {
        clear();
        return null;
      }
      accessStore.setAccessToken(persistedToken);
    }
    try {
      return await hydrateAccessEnvelope();
    } catch {
      clear();
      return null;
    }
  }

  async function fetchVbenUserInfo() {
    return await hydrateAccessEnvelope();
  }

  async function logout() {
    try {
      await logoutApi();
    } finally {
      clear();
    }
  }

  async function switchAuthority(authorityId: number) {
    const data = await setUserAuthorityApi(authorityId);
    accessStore.setAccessToken(null);
    resetAccessContext();
    await hydrateAccessEnvelope(data.user);
    return userStore.userInfo as LegacyUserInfo & VbenUserInfo;
  }

  function clear() {
    stopSessionKeepAlive();
    clearClientAuthState();
    policyPaths.value = [];
    bootstrapped.value = true;
  }

  function $reset() {
    stopSessionKeepAlive();
    stopAuthSync();
    bootstrapped.value = false;
    loginLoading.value = false;
    policyPaths.value = [];
    clearClientAuthState();
  }

  return {
    $reset,
    bootstrapped,
    bootstrap,
    clear,
    defaultRouterName,
    ensureFreshSession,
    fetchVbenUserInfo,
    hydrateAccessEnvelope,
    isLoggedIn,
    login,
    loginLoading,
    logout,
    policyPaths,
    switchAuthority,
    token,
    userInfo,
  };
});
