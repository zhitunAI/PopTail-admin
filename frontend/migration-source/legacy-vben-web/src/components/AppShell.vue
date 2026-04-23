<template>
  <div class="vben-shell">
    <el-container class="vben-shell__frame">
      <el-aside v-if="!isMobile" :width="asideWidth" class="vben-shell__aside">
        <div class="vben-shell__brand">
          <div class="vben-shell__brand-mark">GAA</div>
          <div v-show="!collapsed" class="vben-shell__brand-copy">
            <strong>Gin AI Admin</strong>
            <span>Vue Vben 风格工作台</span>
          </div>
        </div>

        <el-scrollbar class="vben-shell__menu-scroll">
          <el-menu
            :collapse="collapsed"
            :collapse-transition="false"
            :default-active="activeMenuIndex"
            class="vben-shell__menu"
            router
            unique-opened
          >
            <AppMenuTree
              v-for="entry in visibleMenus"
              :key="entry.ID || entry.name || entry.path"
              :item="entry"
            />
          </el-menu>
        </el-scrollbar>

        <div class="vben-shell__aside-footer" v-show="!collapsed">
          <span>{{ runtimeLabel }}</span>
          <strong>{{ auth.policyPaths.length }} 条权限路径</strong>
        </div>
      </el-aside>

      <el-container class="vben-shell__container">
        <el-header class="vben-shell__header">
          <div class="vben-shell__header-left">
            <el-button circle plain @click="toggleSidebar">
              <el-icon><component :is="collapsed && !isMobile ? Expand : Fold" /></el-icon>
            </el-button>
            <div class="vben-shell__breadcrumb-wrap">
              <div class="vben-shell__eyebrow">{{ currentSection }}</div>
              <el-breadcrumb separator="/">
                <el-breadcrumb-item v-for="item in breadcrumbs" :key="item.path">
                  <RouterLink v-if="item.path && item.path !== route.path" :to="item.path">{{ item.label }}</RouterLink>
                  <span v-else>{{ item.label }}</span>
                </el-breadcrumb-item>
              </el-breadcrumb>
            </div>
          </div>

          <div class="vben-shell__header-right">
            <el-select
              v-if="authorityOptions.length > 1"
              :model-value="auth.userInfo?.authorityId"
              class="vben-shell__authority"
              placeholder="切换角色"
              @change="onAuthorityChange"
            >
              <el-option
                v-for="authority in authorityOptions"
                :key="authority.authorityId"
                :label="authority.authorityName"
                :value="authority.authorityId"
              />
            </el-select>
            <el-button plain @click="reload">
              <el-icon><Refresh /></el-icon>
              刷新权限
            </el-button>
            <el-dropdown trigger="click" @command="onUserCommand">
              <button class="vben-shell__user" type="button">
                <el-avatar :size="36">{{ userInitials }}</el-avatar>
                <div class="vben-shell__user-copy">
                  <strong>{{ auth.userInfo?.nickName || auth.userInfo?.userName || "访客" }}</strong>
                  <span>{{ auth.userInfo?.authority.authorityName || "未登录" }}</span>
                </div>
                <el-icon><ArrowDown /></el-icon>
              </button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="profile">个人资料</el-dropdown-item>
                  <el-dropdown-item command="about">关于系统</el-dropdown-item>
                  <el-dropdown-item divided command="logout">退出登录</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </el-header>

        <el-main class="vben-shell__main">
          <div class="vben-shell__page-head">
            <div>
              <h1>{{ pageTitle }}</h1>
              <p>{{ pageDescription }}</p>
            </div>
            <div class="vben-shell__page-tags">
              <el-tag effect="plain" round>{{ auth.userInfo?.authority.authorityName || "未登录" }}</el-tag>
              <el-tag effect="plain" round type="info">默认入口 {{ auth.defaultRouterName }}</el-tag>
              <el-tag effect="plain" round type="success">菜单 {{ visibleMenus.length }} 组</el-tag>
            </div>
          </div>
          <RouterView />
        </el-main>
      </el-container>
    </el-container>

    <el-drawer v-model="mobileOpen" :with-header="false" direction="ltr" size="280px">
      <div class="vben-shell__brand vben-shell__brand--drawer">
        <div class="vben-shell__brand-mark">GA</div>
        <div class="vben-shell__brand-copy">
          <strong>Gin AI Admin</strong>
          <span>Vue Vben 风格工作台</span>
        </div>
      </div>
      <el-scrollbar class="vben-shell__menu-scroll">
        <el-menu :default-active="activeMenuIndex" class="vben-shell__menu" router unique-opened @select="mobileOpen = false">
          <AppMenuTree
            v-for="entry in visibleMenus"
            :key="`mobile-${entry.ID || entry.name || entry.path}`"
            :item="entry"
          />
        </el-menu>
      </el-scrollbar>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { ArrowDown, Expand, Fold, Refresh } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";
import { useNavigationStore } from "../stores/navigation";
import type { MenuInfo } from "../types";
import AppMenuTree from "./AppMenuTree.vue";

const auth = useAuthStore();
const navigation = useNavigationStore();
const route = useRoute();
const router = useRouter();

const collapsed = ref(false);
const isMobile = ref(false);
const mobileOpen = ref(false);
let mediaQuery: MediaQueryList | null = null;

type BreadcrumbEntry = { label: string; path: string };
type FlatMenuEntry = { item: MenuInfo; path: string; chain: BreadcrumbEntry[] };

function normalizeMenuPath(path: string, parentPath = "") {
  const raw = (path || "").trim();
  if (!raw) {
    return parentPath || "/";
  }
  if (raw.startsWith("/")) {
    return raw.replace(/\/+/g, "/");
  }
  const base = parentPath ? `${parentPath.replace(/\/$/, "")}/${raw}` : `/${raw}`;
  return base.replace(/\/+/g, "/");
}

function flattenMenus(items: MenuInfo[], parentPath = "", chain: BreadcrumbEntry[] = []): FlatMenuEntry[] {
  return items.flatMap((item) => {
    const path = normalizeMenuPath(item.path, parentPath);
    const nextChain = [...chain, { label: item.meta.title, path }];
    const current = [{ item, path, chain: nextChain }];
    if (!item.children?.length) {
      return current;
    }
    return current.concat(flattenMenus(item.children.filter((child) => !child.hidden), path, nextChain));
  });
}

const visibleMenus = computed(() => navigation.menus.filter((item) => !item.hidden));
const authorityOptions = computed(() => auth.userInfo?.authorities ?? []);
const allMenuEntries = computed<FlatMenuEntry[]>(() => flattenMenus(visibleMenus.value));

const activeMenuEntry = computed(() => {
  const exact = allMenuEntries.value.find((entry) => entry.path === route.path);
  if (exact) {
    return exact;
  }
  return [...allMenuEntries.value]
    .filter((entry) => route.path === entry.path || route.path.startsWith(`${entry.path}/`))
    .sort((a, b) => b.path.length - a.path.length)[0];
});

const activeMenuIndex = computed(() => activeMenuEntry.value?.path || route.path);
const breadcrumbs = computed(() => activeMenuEntry.value?.chain || [{ label: "控制台", path: route.path }]);
const currentSection = computed(() => breadcrumbs.value[0]?.label || "控制台");
const pageTitle = computed(() => breadcrumbs.value[breadcrumbs.value.length - 1]?.label || "控制台");
const pageDescription = computed(() => {
  if (route.path.startsWith("/system/tools")) return "以 Vben Admin 风格统一工具链、状态、配置和生成工作流。";
  if (route.path.startsWith("/system/")) return "围绕管理域使用统一骨架、卡片、筛选和台账视图承载操作。";
  if (route.path.startsWith("/examples")) return "示例页统一进入 Vben 式工作台布局，保持验证链路可巡检。";
  if (route.path.startsWith("/profile")) return "聚合个人资料、偏好设置和角色态势。";
  return "已切换为 Vue Vben Admin 风格骨架，后续页面保持统一工作台体验。";
});
const runtimeLabel = computed(() => (auth.isLoggedIn ? "会话已接入" : "未建立会话"));
const asideWidth = computed(() => (collapsed.value ? "88px" : "268px"));
const userInitials = computed(() => {
  const name = auth.userInfo?.nickName || auth.userInfo?.userName || "GA";
  return name.slice(0, 2).toUpperCase();
});

function syncBreakpoint(event?: MediaQueryList | MediaQueryListEvent) {
  const matches = "matches" in (event ?? {}) ? (event as MediaQueryList | MediaQueryListEvent).matches : window.matchMedia("(max-width: 960px)").matches;
  isMobile.value = matches;
  if (matches) {
    mobileOpen.value = false;
    collapsed.value = false;
  }
}

function toggleSidebar() {
  if (isMobile.value) {
    mobileOpen.value = !mobileOpen.value;
    return;
  }
  collapsed.value = !collapsed.value;
}

async function logout() {
  await auth.logout();
  await router.push({ name: "login" });
}

async function reload() {
  await auth.bootstrap();
  ElMessage.success("已刷新当前会话与权限包");
}

async function onAuthorityChange(next: number) {
  if (!Number.isFinite(next) || next === auth.userInfo?.authorityId) {
    return;
  }
  await auth.switchAuthority(next);
  if (auth.defaultRouterName && router.hasRoute(auth.defaultRouterName)) {
    await router.push({ name: auth.defaultRouterName });
  }
}

async function onUserCommand(command: string) {
  if (command === "logout") {
    await logout();
    return;
  }
  if (command === "profile") {
    await router.push({ name: "profile" });
    return;
  }
  if (command === "about") {
    await router.push({ name: "about" });
  }
}

watch(
  () => route.fullPath,
  () => {
    mobileOpen.value = false;
  },
);

onMounted(() => {
  mediaQuery = window.matchMedia("(max-width: 960px)");
  syncBreakpoint(mediaQuery);
  mediaQuery.addEventListener("change", syncBreakpoint);
});

onBeforeUnmount(() => {
  mediaQuery?.removeEventListener("change", syncBreakpoint);
});
</script>
