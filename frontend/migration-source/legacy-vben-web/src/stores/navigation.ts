import { defineStore } from "pinia";
import { getMenuTreeApi } from "../api/admin";
import type { MenuInfo } from "../types";

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
