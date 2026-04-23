import type { AxiosResponse, InternalAxiosRequestConfig } from 'axios';

import axios from 'axios';
import { useAccessStore, useUserStore } from '@vben/stores';
import { getActivePinia } from 'pinia';

const TOKEN_KEY = 'gaa-token';
const EXPIRES_AT_KEY = 'gaa-token-exp';
let handlingUnauthorized = false;

function getSessionStorage() {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.sessionStorage;
}

function readTokenStorage(key: string) {
  return getSessionStorage()?.getItem(key) ?? '';
}

function writeTokenStorage(key: string, value: string) {
  getSessionStorage()?.setItem(key, value);
}

function removeTokenStorage(key: string) {
  getSessionStorage()?.removeItem(key);
}

function removeLegacyLocalStorageToken(key: string) {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.removeItem(key);
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
  // cleanup legacy localStorage footprint
  removeLegacyLocalStorageToken(TOKEN_KEY);
  removeLegacyLocalStorageToken(EXPIRES_AT_KEY);
}

export function clearStoredToken(): void {
  removeTokenStorage(TOKEN_KEY);
  removeTokenStorage(EXPIRES_AT_KEY);
  removeLegacyLocalStorageToken(TOKEN_KEY);
  removeLegacyLocalStorageToken(EXPIRES_AT_KEY);
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

function handleUnauthorizedSession() {
  if (isHandlingUnauthorized()) {
    return;
  }

  handlingUnauthorized = true;
  clearAccessContext({ loginExpired: true });
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

function getStoredTokenExpireAt() {
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
    const storage = window.sessionStorage;
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
    import.meta.env.VITE_GAA_API_BASE ??
    import.meta.env.VITE_GLOB_API_URL ??
    'http://127.0.0.1:8888',
  timeout: 12000,
  withCredentials: true,
});

apiClient.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = resolveAuthToken();
  if (token) {
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
    const status = Number(error?.response?.status ?? 0);
    if (status === 401) {
      handleUnauthorizedSession();
    }
    return Promise.reject(error);
  },
);
