import type { AxiosResponse, InternalAxiosRequestConfig } from 'axios';

import axios from 'axios';
import { useAccessStore, useUserStore } from '@vben/stores';
import { getActivePinia } from 'pinia';

const TOKEN_KEY = 'pop-tail-token';
const EXPIRES_AT_KEY = 'pop-tail-token-exp';
const AUTH_EVENT_KEY = 'pop-tail-auth-event';
let handlingUnauthorized = false;
let refreshInFlight: null | Promise<string> = null;

type AuthSyncEvent = {
  at: number;
  loginExpired?: boolean;
  type: 'signed-in' | 'signed-out' | 'token-refreshed';
};

type GinAuthRequestConfig = InternalAxiosRequestConfig & {
  _retry?: boolean;
  _skipAuthRefresh?: boolean;
};

function getLocalStorage() {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.localStorage;
}

function readTokenStorage(key: string) {
  return getLocalStorage()?.getItem(key) ?? '';
}

function writeTokenStorage(key: string, value: string) {
  getLocalStorage()?.setItem(key, value);
}

function removeTokenStorage(key: string) {
  getLocalStorage()?.removeItem(key);
}

function removeLegacySessionStorageToken(key: string) {
  if (typeof window === 'undefined') {
    return;
  }
  window.sessionStorage.removeItem(key);
}

function publishAuthEvent(event: AuthSyncEvent) {
  try {
    writeTokenStorage(AUTH_EVENT_KEY, JSON.stringify(event));
  } catch {
    // ignore cross-tab sync failures
  }
}

export function readAuthSyncEvent(raw: null | string): AuthSyncEvent | null {
  if (!raw) {
    return null;
  }
  try {
    const parsed = JSON.parse(raw) as Partial<AuthSyncEvent>;
    if (
      (parsed.type === 'signed-in' ||
        parsed.type === 'signed-out' ||
        parsed.type === 'token-refreshed') &&
      typeof parsed.at === 'number'
    ) {
      return {
        at: parsed.at,
        loginExpired: Boolean(parsed.loginExpired),
        type: parsed.type,
      };
    }
  } catch {
    // ignore malformed payloads
  }
  return null;
}

function isHandlingUnauthorized() {
  if (handlingUnauthorized) {
    return true;
  }

  try {
    return Boolean(getActivePinia() && useAccessStore().loginExpired);
  } catch {
    return false;
  }
}

export function getStoredToken(): string {
  return readTokenStorage(TOKEN_KEY);
}

export function setStoredToken(token: string, expiresAt?: string): void {
  writeTokenStorage(TOKEN_KEY, token);
  if (expiresAt) {
    writeTokenStorage(EXPIRES_AT_KEY, expiresAt);
  }
  // cleanup legacy sessionStorage footprint from the old auth implementation
  removeLegacySessionStorageToken(TOKEN_KEY);
  removeLegacySessionStorageToken(EXPIRES_AT_KEY);
  publishAuthEvent({
    at: Date.now(),
    type: 'signed-in',
  });
}

export function clearStoredToken(): void {
  removeTokenStorage(TOKEN_KEY);
  removeTokenStorage(EXPIRES_AT_KEY);
  removeLegacySessionStorageToken(TOKEN_KEY);
  removeLegacySessionStorageToken(EXPIRES_AT_KEY);
}

function clearPersistedAccessStoreTokens() {
  if (typeof window === 'undefined') {
    return;
  }
  const storages: Storage[] = [window.localStorage, window.sessionStorage];
  for (const storage of storages) {
    for (let index = storage.length - 1; index >= 0; index -= 1) {
      const key = storage.key(index);
      if (!key || !key.endsWith('-core-access')) {
        continue;
      }
      storage.removeItem(key);
    }
  }
}

function clearAccessContext({ loginExpired = false } = {}) {
  clearStoredToken();
  clearPersistedAccessStoreTokens();

  try {
    if (!getActivePinia()) {
      return;
    }
    const accessStore = useAccessStore();
    accessStore.setAccessToken(null);
    accessStore.setAccessCodes([]);
    accessStore.setAccessMenus([]);
    accessStore.setAccessRoutes([]);
    accessStore.setIsAccessChecked(false);
    accessStore.setLoginExpired(loginExpired);
    useUserStore().setUserInfo(null);
  } catch {
    // ignore pinia state cleanup failure
  }
}

export function clearClientAuthState({
  loginExpired = false,
  publish = true,
}: {
  loginExpired?: boolean;
  publish?: boolean;
} = {}) {
  handlingUnauthorized = loginExpired;
  refreshInFlight = null;
  clearAccessContext({ loginExpired });
  if (publish) {
    publishAuthEvent({
      at: Date.now(),
      loginExpired,
      type: 'signed-out',
    });
  }
}

function handleUnauthorizedSession() {
  if (isHandlingUnauthorized()) {
    return;
  }

  clearClientAuthState({ loginExpired: true });
}

function parseJwtPayload(token: string): null | Record<string, unknown> {
  try {
    const parts = token.split('.');
    if (parts.length < 2) {
      return null;
    }
    const base64 = parts[1]!.replace(/-/g, '+').replace(/_/g, '/');
    const normalized = base64.padEnd(Math.ceil(base64.length / 4) * 4, '=');
    const json = atob(normalized);
    return JSON.parse(json) as Record<string, unknown>;
  } catch {
    return null;
  }
}

export function getStoredTokenExpireAt() {
  const raw = readTokenStorage(EXPIRES_AT_KEY);
  if (!raw) {
    return 0;
  }
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

export function isTokenExpired(token: string) {
  if (!token) {
    return true;
  }

  const now = Date.now();
  const storedExpireAt = getStoredTokenExpireAt();
  if (storedExpireAt > 0) {
    return storedExpireAt <= now;
  }

  const payload = parseJwtPayload(token);
  const exp = Number(payload?.exp ?? 0);
  if (!Number.isFinite(exp) || exp <= 0) {
    return true;
  }
  return exp * 1000 <= now;
}

function getTokenFromPersistedAccessStore() {
  if (typeof window === 'undefined') {
    return '';
  }
  try {
    const storages: Storage[] = [window.localStorage, window.sessionStorage];
    for (const storage of storages) {
      for (let index = 0; index < storage.length; index += 1) {
        const key = storage.key(index);
        if (!key || !key.endsWith('-core-access')) {
          continue;
        }
        const raw = storage.getItem(key);
        if (!raw) {
          continue;
        }
        const parsed = JSON.parse(raw) as { accessToken?: unknown };
        if (typeof parsed.accessToken === 'string' && parsed.accessToken.length > 0) {
          return parsed.accessToken;
        }
      }
    }
  } catch {
    // ignore read/parse failures and fall back to other sources
  }
  return '';
}

function resolveAuthToken() {
  const directToken = getStoredToken();
  if (directToken && !isTokenExpired(directToken)) {
    return directToken;
  }
  if (directToken && isTokenExpired(directToken)) {
    clearStoredToken();
  }

  try {
    if (getActivePinia()) {
      const accessToken = useAccessStore().accessToken;
      if (
        typeof accessToken === 'string' &&
        accessToken.length > 0 &&
        !isTokenExpired(accessToken)
      ) {
        setStoredToken(accessToken);
        return accessToken;
      }
    }
  } catch {
    // ignore pinia access failures and try persisted fallback
  }

  const persistedToken = getTokenFromPersistedAccessStore();
  if (persistedToken && !isTokenExpired(persistedToken)) {
    setStoredToken(persistedToken);
    return persistedToken;
  }

  return '';
}

export const apiClient = axios.create({
  baseURL:
    import.meta.env.VITE_POP_TAIL_API_BASE ??
    import.meta.env.VITE_GLOB_API_URL ??
    'http://127.0.0.1:8888',
  timeout: 12000,
  withCredentials: true,
});

const refreshClient = axios.create({
  baseURL:
    import.meta.env.VITE_POP_TAIL_API_BASE ??
    import.meta.env.VITE_GLOB_API_URL ??
    'http://127.0.0.1:8888',
  timeout: 12000,
  withCredentials: true,
});

async function refreshAccessToken() {
  if (refreshInFlight) {
    return await refreshInFlight;
  }

  refreshInFlight = (async () => {
    const response = await refreshClient.post<{
      code: number;
      data?: { expiresAt?: number; token?: string };
      msg?: string;
    }>(
      '/base/refresh',
      { refreshToken: '' },
      { headers: { 'Content-Type': 'application/json' } },
    );
    if (response.data.code !== 0 || !response.data.data?.token) {
      throw new Error(response.data.msg || '刷新登录态失败');
    }
    const token = response.data.data.token;
    setStoredToken(token, String(response.data.data.expiresAt ?? ''));
    try {
      if (getActivePinia()) {
        useAccessStore().setAccessToken(token);
      }
    } catch {
      // ignore pinia sync failure
    }
    handlingUnauthorized = false;
    publishAuthEvent({
      at: Date.now(),
      type: 'token-refreshed',
    });
    return token;
  })();

  try {
    return await refreshInFlight;
  } finally {
    refreshInFlight = null;
  }
}

apiClient.interceptors.request.use((config: GinAuthRequestConfig) => {
  const token = resolveAuthToken();
  if (token && !config._skipAuthRefresh) {
    handlingUnauthorized = false;
    config.headers['x-token'] = token;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    const newToken = response.headers['new-token'];
    const newExpiresAt = response.headers['new-expires-at'];
    if (typeof newToken === 'string' && newToken.length > 0) {
      setStoredToken(newToken, String(newExpiresAt ?? ''));
      handlingUnauthorized = false;
    }
    return response;
  },
  async (error) => {
    const config = (error?.config ?? {}) as GinAuthRequestConfig;
    const status = Number(error?.response?.status ?? 0);
    const requestUrl = String(config?.url ?? '');
    const isRefreshRequest = requestUrl.includes('/base/refresh') || config._skipAuthRefresh;

    if (status === 401 && !config._retry && !isRefreshRequest) {
      config._retry = true;
      try {
        const nextToken = await refreshAccessToken();
        config.headers = config.headers ?? {};
        config.headers['x-token'] = nextToken;
        return await apiClient.request(config);
      } catch {
        handleUnauthorizedSession();
      }
    } else if (status === 401) {
      handleUnauthorizedSession();
    }
    return Promise.reject(error);
  },
);
