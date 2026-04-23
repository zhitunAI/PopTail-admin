import { defineStore } from "pinia";
import { clearStoredToken, getStoredToken } from "../api/client";
import {
  getPolicyPathByAuthorityId,
  getUserInfoApi,
  loginApi,
  logoutApi,
  setUserAuthorityApi,
} from "../api/auth";
import { useNavigationStore } from "./navigation";
import type { PolicyPath, UserInfo } from "../types";

interface AuthState {
  token: string;
  userInfo: UserInfo | null;
  policyPaths: PolicyPath[];
  bootstrapped: boolean;
}

export const useAuthStore = defineStore("auth", {
  state: (): AuthState => ({
    token: getStoredToken(),
    userInfo: null,
    policyPaths: [],
    bootstrapped: false,
  }),
  getters: {
    isLoggedIn(state): boolean {
      return Boolean(state.token);
    },
    defaultRouterName(state): string {
      return state.userInfo?.authority?.defaultRouter ?? "dashboard";
    },
  },
  actions: {
    async hydrateAccessEnvelope(user?: UserInfo) {
      const navigation = useNavigationStore();
      const effectiveUser = user ?? (await getUserInfoApi());
      this.token = getStoredToken();
      this.userInfo = effectiveUser;
      this.policyPaths = await getPolicyPathByAuthorityId(effectiveUser.authorityId);
      await navigation.loadMenus();
    },
    async login(username: string, password: string) {
      const data = await loginApi(username, password);
      await this.hydrateAccessEnvelope(data.user);
      this.bootstrapped = true;
    },
    async bootstrap() {
      if (!this.token) {
        this.bootstrapped = true;
        return;
      }
      try {
        await this.hydrateAccessEnvelope();
      } catch {
        this.clear();
      } finally {
        this.bootstrapped = true;
      }
    },
    async logout() {
      const navigation = useNavigationStore();
      await logoutApi();
      navigation.clear();
      this.clear();
    },
    async switchAuthority(authorityId: number) {
      const data = await setUserAuthorityApi(authorityId);
      await this.hydrateAccessEnvelope(data.user);
    },
    clear() {
      clearStoredToken();
      this.token = "";
      this.userInfo = null;
      this.policyPaths = [];
      this.bootstrapped = true;
    },
  },
});
