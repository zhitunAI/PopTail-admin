<template>
  <template v-if="visibleChildren.length">
    <el-sub-menu :index="resolvedPath">
      <template #title>
        <div class="vben-shell__menu-title">
          <span class="vben-shell__menu-dot"></span>
          <span>{{ item.meta.title }}</span>
        </div>
      </template>
      <AppMenuTree
        v-for="child in visibleChildren"
        :key="child.ID || child.name || child.path"
        :item="child"
        :parent-path="resolvedPath"
      />
    </el-sub-menu>
  </template>
  <el-menu-item v-else :index="resolvedPath">
    <div class="vben-shell__menu-title">
      <span class="vben-shell__menu-dot"></span>
      <span>{{ item.meta.title }}</span>
    </div>
  </el-menu-item>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { MenuInfo } from "../types";

const props = defineProps<{
  item: MenuInfo;
  parentPath?: string;
}>();

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

const resolvedPath = computed(() => normalizeMenuPath(props.item.path, props.parentPath));
const visibleChildren = computed(() => (props.item.children ?? []).filter((child) => !child.hidden));
</script>
