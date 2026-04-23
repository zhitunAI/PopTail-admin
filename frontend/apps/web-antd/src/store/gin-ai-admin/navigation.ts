import { defineStore } from "pinia";
import { getMenuTreeApi } from "#/api/gin-ai-admin/admin";
import type { MenuInfo } from "#/types/gin-ai-admin";

interface NavigationState {
  menus: MenuInfo[];
}

export const useNavigationStore = defineStore("navigation", {
  state: (): NavigationState => ({
    menus: [],
  }),
  actions: {
    async loadMenus() {
      this.menus = await getMenuTreeApi();
    },
    clear() {
      this.menus = [];
    },
  },
});
