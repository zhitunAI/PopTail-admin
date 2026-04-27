import type { Recordable } from '@vben/types';

import { useRouter } from 'vue-router';

import { LOGIN_PATH } from '@vben/constants';
import { preferences } from '@vben/preferences';
import { resetAllStores, useAccessStore } from '@vben/stores';

import { notification } from 'ant-design-vue';
import { defineStore } from 'pinia';

import { $t } from '#/locales';
import {
  isAuthenticationPath,
  normalizeAuthRoutePath,
  useAuthStore as useGinAuthStore,
} from '#/store/pop-tail/auth';

export const useAuthStore = defineStore('auth', () => {
  const ginAuthStore = useGinAuthStore();
  const accessStore = useAccessStore();
  const router = useRouter();

  async function authLogin(
    params: Recordable<any>,
    onSuccess?: () => Promise<void> | void,
  ) {
    const username = String(params?.username ?? '');
    const password = String(params?.password ?? '');
    const userInfo = await ginAuthStore.login(username, password);

    if (accessStore.loginExpired) {
      accessStore.setLoginExpired(false);
      const currentRoute = router.currentRoute.value;
      const refreshKey = `relogin-${Date.now()}`;
      await router.replace({
        hash: currentRoute.hash,
        path: currentRoute.path,
        query: {
          ...currentRoute.query,
          __relogin: refreshKey,
        },
      });
      await router.replace({
        hash: currentRoute.hash,
        path: currentRoute.path,
        query: currentRoute.query,
      });
    } else {
      onSuccess
        ? await onSuccess?.()
        : await router.push(
            normalizeAuthRoutePath(
              userInfo.homePath || preferences.app.defaultHomePath,
            ),
          );
    }

    notification.success({
      description: `${$t('authentication.loginSuccessDesc')}:${userInfo.realName}`,
      duration: 3,
      message: $t('authentication.loginSuccess'),
    });

    return { userInfo };
  }

  async function logout(redirect: boolean = true) {
    const currentPath = normalizeAuthRoutePath(router.currentRoute.value.fullPath);
    const shouldPreserveRedirect =
      redirect && currentPath !== LOGIN_PATH && !isAuthenticationPath(currentPath);

    await ginAuthStore.logout();
    resetAllStores();
    accessStore.setLoginExpired(false);

    await router.replace({
      path: LOGIN_PATH,
      query: shouldPreserveRedirect
        ? {
            redirect: encodeURIComponent(currentPath),
          }
        : {},
    });
  }

  async function fetchUserInfo() {
    return await ginAuthStore.hydrateAccessEnvelope();
  }

  function $reset() {
    accessStore.setLoginExpired(false);
  }

  return {
    $reset,
    authLogin,
    fetchUserInfo,
    loginLoading: ginAuthStore.loginLoading,
    logout,
  };
});
