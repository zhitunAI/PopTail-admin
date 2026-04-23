<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h1 class="title">初始化引导工作台</h1>
          <p class="subtitle">以真实台账、配置与运行态信号展示当前恢复基线，帮助判断现在该从哪里继续验收。</p>
        </div>
        <button class="btn ghost" :disabled="loading" @click="reload">
          {{ loading ? "刷新中..." : "重新拉取初始化状态" }}
        </button>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>用户</h4>
        <p class="stat-value">{{ summary.users }}</p>
        <p class="subtitle">当前可登录或可管理账户数</p>
      </div>
      <div class="stat-card">
        <h4>角色</h4>
        <p class="stat-value">{{ summary.authorities }}</p>
        <p class="subtitle">角色与默认路由已恢复</p>
      </div>
      <div class="stat-card">
        <h4>菜单</h4>
        <p class="stat-value">{{ summary.menus }}</p>
        <p class="subtitle">可分配导航与页面入口</p>
      </div>
      <div class="stat-card">
        <h4>就绪度</h4>
        <p class="stat-value">{{ readinessScore }}%</p>
        <p class="subtitle">{{ readinessLabel }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">恢复状态</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><th>登录</th><td>已恢复</td></tr>
              <tr><th>核心管理模块</th><td>已恢复并持续做深交互</td></tr>
              <tr><th>系统工具页</th><td>{{ summary.mcpTools ? "已接入真实后端" : "持续推进中" }}</td></tr>
              <tr><th>页面 parity</th><td>{{ readinessLabel }}</td></tr>
              <tr><th>Tauri 2 重构</th><td>Web 验收完成后进入下一阶段</td></tr>
              <tr><th>当前存储</th><td>{{ storageLabel }}</td></tr>
              <tr><th>最近刷新</th><td>{{ lastRefresh }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">建议入口</h3>
        <div class="tag-list">
          <RouterLink class="tag" to="/dashboard">仪表盘</RouterLink>
          <RouterLink class="tag" to="/system/overview">管理总览</RouterLink>
          <RouterLink class="tag" to="/system/state">系统状态</RouterLink>
          <RouterLink class="tag" to="/system/tools/runtime-state">运行控制台</RouterLink>
          <RouterLink class="tag" to="/examples">示例中心</RouterLink>
        </div>
        <div class="section top-gap">
          <h4>下一步建议</h4>
          <ul class="bullet-list">
            <li v-for="item in nextActions" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">关键资产概览</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>资产</th>
                <th>数量</th>
                <th>状态</th>
              </tr>
            </thead>
            <tbody>
              <tr><td>技能</td><td>{{ summary.skills }}</td><td>{{ summary.skills ? "已接入" : "待补齐" }}</td></tr>
              <tr><td>MCP 工具</td><td>{{ summary.mcpTools }}</td><td>{{ summary.mcpTools ? "已定义" : "待补定义" }}</td></tr>
              <tr><td>数据库后端</td><td>-</td><td>{{ runtimeBackend }}</td></tr>
              <tr><td>Redis</td><td>-</td><td>{{ redisLabel }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">阶段判断</h3>
        <div class="signal-list">
          <div v-for="item in phaseSignals" :key="item.title" class="signal-item">
            <div class="row between wrap">
              <strong>{{ item.title }}</strong>
              <span class="tag">{{ item.level }}</span>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { RouterLink } from "vue-router";
import {
  getAuthorityListApi,
  getMcpToolListApi,
  getMenuListApi,
  getRuntimeInfoApi,
  getSkillListApi,
  getUserListApi,
} from "../api/admin";

const summary = reactive({
  users: 0,
  authorities: 0,
  menus: 0,
  skills: 0,
  mcpTools: 0,
});

const loading = ref(false);
const message = ref("");
const error = ref("");
const storageLabel = ref("-");
const lastRefresh = ref("未刷新");
const runtimeBackend = ref("-");
const redisLabel = ref("-");

const readinessScore = computed(() => {
  let score = 40;
  if (summary.users > 0) score += 10;
  if (summary.authorities > 0) score += 10;
  if (summary.menus > 0) score += 10;
  if (summary.skills > 0) score += 10;
  if (summary.mcpTools > 0) score += 10;
  if (redisLabel.value === "已接入") score += 10;
  return Math.min(100, score);
});

const readinessLabel = computed(() => {
  if (readinessScore.value >= 90) return "已进入深验收阶段";
  if (readinessScore.value >= 70) return "主体恢复完成，仍需补深交互";
  return "仍在恢复基线搭建阶段";
});

const nextActions = computed(() => {
  const items = [
    "先从仪表盘或管理总览核对角色、菜单与权限包是否一致。",
    "再进入系统工具与运行态页面，确认工具链、配置与服务状态。",
  ];
  if (!summary.mcpTools) {
    items.push("当前 MCP 工具仍少，建议继续补齐工具定义与联调链路。");
  }
  if (redisLabel.value !== "已接入") {
    items.push("Redis 尚未就绪，建议优先核对会话与状态化链路。");
  }
  return items;
});

const phaseSignals = computed(() => [
  {
    title: "账号与角色基线",
    level: summary.users && summary.authorities ? "已具备" : "待补齐",
    detail: summary.users && summary.authorities ? "用户、角色已可用于继续权限回归。" : "先补全用户和角色基线再推进页面验收。",
  },
  {
    title: "导航与入口",
    level: summary.menus ? "已恢复" : "待恢复",
    detail: summary.menus ? "菜单入口已可分配，适合继续做页面链路验收。" : "菜单尚不足，建议优先恢复入口配置。",
  },
  {
    title: "工具链深度",
    level: summary.skills || summary.mcpTools ? "推进中" : "待建设",
    detail: summary.skills || summary.mcpTools ? "技能与 MCP 已接入一部分，可继续补实。" : "工具链资产还少，建议继续恢复。", 
  },
]);

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [users, authorities, menus, skills, mcpTools, runtime] = await Promise.all([
      getUserListApi(),
      getAuthorityListApi(),
      getMenuListApi(),
      getSkillListApi(),
      getMcpToolListApi(),
      getRuntimeInfoApi(),
    ]);
    summary.users = users.Total;
    summary.authorities = authorities.length;
    summary.menus = menus.Total;
    summary.skills = skills.length;
    summary.mcpTools = mcpTools.length;
    runtimeBackend.value = runtime.dbBackend;
    redisLabel.value = runtime.redisEnabled ? "已接入" : "未接入";
    storageLabel.value = `${runtime.dbBackend}${runtime.redisEnabled ? " + Redis" : ""}`;
    lastRefresh.value = new Date().toLocaleString("zh-CN");
    message.value = "已同步初始化引导所需的资产、工具与运行态信息。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取初始化状态失败";
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void reload();
});
</script>
