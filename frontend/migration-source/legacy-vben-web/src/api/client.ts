import axios from "axios";

const TOKEN_KEY = "gaa-token";
const EXPIRES_AT_KEY = "gaa-token-exp";

export function getStoredToken(): string {
  return localStorage.getItem(TOKEN_KEY) ?? "";
}

export function setStoredToken(token: string, expiresAt?: string): void {
  localStorage.setItem(TOKEN_KEY, token);
  if (expiresAt) {
    localStorage.setItem(EXPIRES_AT_KEY, expiresAt);
  }
}

export function clearStoredToken(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(EXPIRES_AT_KEY);
}

export const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE ?? "http://127.0.0.1:8888",
  timeout: 12000,
});

apiClient.interceptors.request.use((config) => {
  const token = getStoredToken();
  if (token) {
    config.headers["x-token"] = token;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response) => {
    const newToken = response.headers["new-token"];
    const newExpiresAt = response.headers["new-expires-at"];
    if (typeof newToken === "string" && newToken.length > 0) {
      setStoredToken(newToken, String(newExpiresAt ?? ""));
    }
    return response;
  },
  (error) => Promise.reject(error),
);
