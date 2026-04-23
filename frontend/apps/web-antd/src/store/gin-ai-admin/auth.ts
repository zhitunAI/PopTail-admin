import type { RouteRecordRaw } from 'vue-router';

import type { UserInfo as VbenUserInfo } from '@vben/types';

import type { UserInfo as LegacyUserInfo, PolicyPath } from '#/types/gin-ai-admin';

import { computed, ref } from 'vue';

import { preferences } from '@vben/preferences';
import { useAccessStore, useUserStore } from '@vben/stores';

import { defineStore } from 'pinia';

import {
  getPolicyPathByAuthorityId,
  getUserInfoApi,
  loginApi,
  logoutApi,
  setUserAuthorityApi,
} from '#/api/gin-ai-admin/auth';
import {
  clearStoredToken,
  getStoredToken,
  isTokenExpired,
  setStoredToken,
} from '#/api/gin-ai-admin/client';

const DEFAULT_HOME_PATH = '/dashboard';

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

function normalizeHomePath(path?: string) {
  return normalizeAuthRoutePath(path);
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

function toVbenUserInfo(
  user: LegacyUserInfo,
  token: string,
): LegacyUserInfo & VbenUserInfo {
  return {
    ...user,
    avatar: user.headerImg || preferences.app.defaultAvatar,
    desc: user.email || user.phone || user.authority?.authorityName || 'Gin AI Admin User',
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

export const useAuthStore = defineStore('gin-ai-admin-auth', () => {
  const accessStore = useAccessStore();
  const userStore = useUserStore();

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

  async function hydrateAccessEnvelope(user?: LegacyUserInfo) {
    const effectiveUser = user ?? (await getUserInfoApi());
    const currentToken = getStoredToken() || accessStore.accessToken || '';
    if (currentToken && isTokenExpired(currentToken)) {
      clear();
      throw new Error('登录状态已过期，请重新登录');
    }
    accessStore.setAccessToken(currentToken || null);
    userStore.setUserInfo(toVbenUserInfo(effectiveUser, currentToken));
    policyPaths.value = await getPolicyPathByAuthorityId(effectiveUser.authorityId);
    accessStore.setAccessCodes(buildPolicyAccessCodes(policyPaths.value));
    bootstrapped.value = true;
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
    clearStoredToken();
    accessStore.setAccessToken(null);
    userStore.setUserInfo(null);
    policyPaths.value = [];
    resetAccessContext();
    bootstrapped.value = true;
  }

  function $reset() {
    bootstrapped.value = false;
    loginLoading.value = false;
    policyPaths.value = [];
    clearStoredToken();
    accessStore.setAccessToken(null);
    accessStore.setAccessCodes([]);
    accessStore.setAccessMenus([]);
    accessStore.setAccessRoutes([]);
    accessStore.setIsAccessChecked(false);
    userStore.setUserInfo(null);
  }

  return {
    $reset,
    bootstrapped,
    bootstrap,
    clear,
    defaultRouterName,
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
