<template>
  <div class="page-shell">
    <div class="stack">
      <div class="card">
        <div class="row between wrap">
          <div>
            <h1 class="title">页面未找到</h1>
            <p class="subtitle">当前地址未命中已恢复页面，或者当前主角色还没有拿到对应菜单与权限。</p>
          </div>
          <button class="btn ghost" @click="goBack">返回上一页</button>
        </div>

        <div class="muted-grid">
          <div class="stat-card">
            <h4>当前地址</h4>
            <p class="stat-value">{{ route.fullPath }}</p>
            <p class="subtitle">请核对是否存在拼写或旧路由残留</p>
          </div>
          <div class="stat-card">
            <h4>当前角色</h4>
            <p class="stat-value">{{ authStore.userInfo?.authority?.authorityName || "-" }}</p>
            <p class="subtitle">默认入口：{{ authStore.userInfo?.authority?.defaultRouter || "dashboard" }}</p>
          </div>
          <div class="stat-card">
            <h4>权限路径</h4>
            <p class="stat-value">{{ authStore.policyPaths.length }}</p>
            <p class="subtitle">{{ permissionHint }}</p>
          </div>
          <div class="stat-card">
            <h4>推荐入口</h4>
            <p class="stat-value">{{ suggestedRoute?.label || "dashboard" }}</p>
            <p class="subtitle">{{ suggestedRoute?.reason || "可先返回首页继续巡检" }}</p>
          </div>
        </div>

        <div class="data-table top-gap">
          <table>
            <tbody>
              <tr><td>当前地址</td><td>{{ route.fullPath }}</td></tr>
              <tr><td>当前角色</td><td>{{ authStore.userInfo?.authority?.authorityName || "-" }}</td></tr>
              <tr><td>热点分段</td><td>{{ routeSegments.join(" / ") || "无" }}</td></tr>
              <tr><td>建议动作</td><td>{{ suggestionText }}</td></tr>
            </tbody>
          </table>
        </div>

        <div class="row wrap top-gap">
          <RouterLink class="btn primary" to="/dashboard">返回仪表盘</RouterLink>
          <RouterLink class="btn ghost" to="/system/overview">打开管理总览</RouterLink>
          <RouterLink class="btn ghost" to="/system/tools/runtime-state">查看运行态</RouterLink>
        </div>
      </div>

      <div class="split-grid">
        <div class="card">
          <h3 class="title">可能想去的页面</h3>
          <div class="signal-list">
            <div v-for="item in prioritizedLinks" :key="item.path" class="signal-item">
              <div class="row between wrap">
                <strong>{{ item.label }}</strong>
                <RouterLink class="btn ghost" :to="item.path">打开</RouterLink>
              </div>
              <p>{{ item.reason }}</p>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="title">已恢复入口</h3>
          <div class="tag-list">
            <RouterLink v-for="item in quickLinks" :key="item.path" class="tag" :to="item.path">{{ item.label }}</RouterLink>
          </div>
          <div class="section top-gap">
            <h4>排查提示</h4>
            <ul class="bullet-list">
              <li>确认当前主角色是否已分配对应菜单。</li>
              <li>确认访问的是新路由，而不是 legacy 路径或旧收藏夹地址。</li>
              <li>如果刚切换角色，可先去首页或错误恢复页重新拉取权限包。</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const routeSegments = computed(() => route.path.split("/").filter(Boolean));

const quickLinks = [
  { label: "管理总览", path: "/system/overview" },
  { label: "系统工具", path: "/system/tools/runtime-state" },
  { label: "示例中心", path: "/examples" },
  { label: "系统状态", path: "/system/state" },
  { label: "控制台", path: "/dashboard" },
];

const prioritizedLinks = computed(() => {
  const path = route.fullPath.toLowerCase();
  const candidates = [
    { label: "用户管理", path: "/system/users", reason: "当前路径与账号、用户或权限段可能相关。" },
    { label: "角色管理", path: "/system/authorities", reason: "如果是鉴权或菜单问题，可先核对角色分配。" },
    { label: "系统工具", path: "/system/tools/runtime-state", reason: "若怀疑状态同步异常，可先查看运行态工作台。" },
    { label: "管理总览", path: "/system/overview", reason: "从总览页重新进入已恢复入口，避免旧地址残留。" },
  ];

  if (path.includes("upload") || path.includes("scan")) {
    return [
      { label: "上传示例", path: "/examples/upload", reason: "当前地址疑似上传链路，可回到上传工作台。" },
      { label: "扫码上传", path: "/scan-upload", reason: "如果你在验证扫码链路，可直接进入扫码上传页。" },
      ...candidates.slice(2),
    ];
  }

  if (path.includes("tool") || path.includes("mcp") || path.includes("auto")) {
    return [
      { label: "系统工具", path: "/system/tools/runtime-state", reason: "当前地址疑似工具链路，可先回到工具总览。" },
      { label: "自动代码", path: "/system/tools/autocode", reason: "若是生成链路问题，可从自动代码工作台重新进入。" },
      ...candidates.slice(2),
    ];
  }

  return candidates;
});

const suggestedRoute = computed(() => prioritizedLinks.value[0] ?? null);

const permissionHint = computed(() => {
  if (!authStore.policyPaths.length) {
    return "当前权限包为空，建议先刷新鉴权";
  }
  if ((authStore.userInfo?.authorities?.length ?? 0) > 1) {
    return "当前账号支持多角色，可尝试切换后重进";
  }
  return "权限包已加载，可核对目标菜单是否已分配";
});

const suggestionText = computed(() => {
  if (!authStore.policyPaths.length) {
    return "先返回首页刷新权限包，再检查角色与菜单分配。";
  }
  return suggestedRoute.value?.reason || "建议从管理总览重新进入目标页面。";
});

async function goBack() {
  await router.go(-1);
}
</script>
