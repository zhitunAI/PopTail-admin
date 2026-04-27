import type { Router } from 'vue-router';

import { LOGIN_PATH } from '@vben/constants';
import { preferences } from '@vben/preferences';
import { useAccessStore, useUserStore } from '@vben/stores';
import { startProgress, stopProgress } from '@vben/utils';

import { accessRoutes, coreRouteNames } from '#/router/routes';
import { getStoredToken } from '#/api/pop-tail/client';
import {
  collectButtonAccessCodesFromRoutes,
  isAuthenticationPath,
  normalizeAuthRoutePath,
  useAuthStore,
} from '#/store/pop-tail/auth';
import { useNavigationStore } from '#/store/pop-tail/navigation';

import { generateAccess } from './access';

function safeDecodeURIComponent(value: string) {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function resolveSafeRedirectPath(input: string, fallback: string) {
  const normalized = normalizeAuthRoutePath(safeDecodeURIComponent(input) || fallback);
  if (isAuthenticationPath(normalized)) {
    return fallback;
  }
  return normalized;
}

/**
 * 通用守卫配置
 * @param router
 */
function setupCommonGuard(router: Router) {
  const loadedPaths = new Set<string>();

  router.beforeEach((to) => {
    to.meta.loaded = loadedPaths.has(to.path);

    if (!to.meta.loaded && preferences.transition.progress) {
      startProgress();
    }
    return true;
  });

  router.afterEach((to) => {
    loadedPaths.add(to.path);

    if (preferences.transition.progress) {
      stopProgress();
    }
  });
}

/**
 * 权限访问守卫配置
 * @param router
 */
function setupAccessGuard(router: Router) {
  router.beforeEach(async (to, from) => {
    const accessStore = useAccessStore();
    const userStore = useUserStore();
    const authStore = useAuthStore();
    const navigationStore = useNavigationStore();
    const persistedToken = getStoredToken();
    const activeToken = accessStore.accessToken || persistedToken;

    if (coreRouteNames.includes(to.name as string)) {
      if (to.path === LOGIN_PATH && activeToken) {
        if (!accessStore.accessToken && persistedToken) {
          accessStore.setAccessToken(persistedToken);
        }
        const restoredUser = userStore.userInfo ?? (await authStore.bootstrap());
        const fallbackPath = normalizeAuthRoutePath(
          restoredUser?.homePath || preferences.app.defaultHomePath,
        );
        const rawRedirect =
          typeof to.query?.redirect === 'string' ? to.query.redirect : '';
        const normalizedRedirect = resolveSafeRedirectPath(rawRedirect, fallbackPath);
        const resolved = router.resolve(normalizedRedirect);
        return resolved.matched.length > 0 ? normalizedRedirect : fallbackPath;
      }
      return true;
    }

    if (!activeToken) {
      if (to.meta.ignoreAccess) {
        return true;
      }

      if (to.fullPath !== LOGIN_PATH) {
        return {
          path: LOGIN_PATH,
          query:
            to.fullPath === preferences.app.defaultHomePath || isAuthenticationPath(to.fullPath)
              ? {}
              : { redirect: encodeURIComponent(normalizeAuthRoutePath(to.fullPath)) },
          replace: true,
        };
      }
      return to;
    }

    if (!accessStore.accessToken && persistedToken) {
      accessStore.setAccessToken(persistedToken);
    }

    if (!userStore.userInfo) {
      const restoredUser = await authStore.bootstrap();
      if (!restoredUser) {
        return {
          path: LOGIN_PATH,
          replace: true,
        };
      }
    }

    if (accessStore.isAccessChecked) {
      return true;
    }

    const userInfo = userStore.userInfo || (await authStore.hydrateAccessEnvelope());
    const userRoles = userInfo.roles ?? [];

    const { accessibleMenus, accessibleRoutes } = await generateAccess({
      roles: userRoles,
      router,
      routes: accessRoutes,
    });

    try {
      const backendMenus = await navigationStore.loadAccessMenus(accessibleRoutes);
      accessStore.setAccessMenus(backendMenus.length > 0 ? backendMenus : accessibleMenus);
    } catch {
      accessStore.setAccessMenus(accessibleMenus);
    }
    accessStore.setAccessRoutes(accessibleRoutes);
    accessStore.setAccessCodes([
      ...new Set([
        ...accessStore.accessCodes,
        ...collectButtonAccessCodesFromRoutes(accessibleRoutes),
      ]),
    ]);
    accessStore.setIsAccessChecked(true);

    const defaultHomePath = normalizeAuthRoutePath(
      userInfo.homePath || preferences.app.defaultHomePath,
    );
    const pendingRedirect =
      typeof from.query.redirect === 'string' && from.query.redirect
        ? from.query.redirect
        : '';
    const redirectPath = resolveSafeRedirectPath(
      pendingRedirect ||
        (to.path === preferences.app.defaultHomePath ? defaultHomePath : to.fullPath),
      defaultHomePath,
    );
    const resolved = router.resolve(redirectPath);

    return resolved.matched.length > 0
      ? {
          ...resolved,
          replace: true,
        }
      : {
          path: defaultHomePath,
          replace: true,
        };
  });
}

/**
 * 项目守卫配置
 * @param router
 */
function createRouterGuard(router: Router) {
  setupCommonGuard(router);
  setupAccessGuard(router);
}

export { createRouterGuard };
