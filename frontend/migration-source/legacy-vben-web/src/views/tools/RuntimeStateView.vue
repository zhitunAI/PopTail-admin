<template>
  <div class="stack">
    <div class="card">
      <div class="toolbar">
        <div>
          <h3 class="title">运行态控制台</h3>
          <p class="subtitle">
            聚合运行时、配置、工具链和示例任务的真实状态，并提供可直接执行的运行操作。
          </p>
        </div>
        <div class="toolbar-actions">
          <label class="inline-field">
            <span>自动刷新</span>
            <input v-model="autoRefresh" type="checkbox" />
          </label>
          <label class="inline-field">
            <span>间隔</span>
            <select v-model.number="refreshInterval">
              <option :value="5000">5 秒</option>
              <option :value="10000">10 秒</option>
              <option :value="30000">30 秒</option>
            </select>
          </label>
          <button class="btn ghost" :disabled="loading" @click="reloadState">立即刷新</button>
        </div>
      </div>
      <div class="tag-list">
        <span class="tag">{{ loading ? "state: refreshing" : "state: live" }}</span>
        <span class="tag">last refresh: {{ lastRefresh }}</span>
        <span class="tag">next poll: {{ autoRefresh ? `${refreshInterval / 1000}s` : "manual" }}</span>
        <span class="tag">{{ runtimeBadge }}</span>
        <span class="tag">{{ clientBadge }}</span>
      </div>
      <p v-if="feedback" class="subtitle">{{ feedback }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>Runtime</h4>
        <p class="stat-value">{{ runtimeSummary.os }}</p>
        <p class="subtitle">{{ runtimeSummary.version }}</p>
      </div>
      <div class="stat-card">
        <h4>Storage</h4>
        <p class="stat-value">{{ runtimeSummary.storage }}</p>
        <p class="subtitle">{{ runtimeSummary.redis }}</p>
      </div>
      <div class="stat-card">
        <h4>MCP</h4>
        <p class="stat-value">{{ toolSummary.mcp }}</p>
        <p class="subtitle">{{ toolSummary.mcpMessage }}</p>
      </div>
      <div class="stat-card">
        <h4>Pending</h4>
        <p class="stat-value">{{ pendingSummary.total }}</p>
        <p class="subtitle">{{ pendingSummary.label }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">服务与配置</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>操作系统</td><td>{{ runtime?.os ?? "-" }}</td></tr>
              <tr><td>CPU 核数</td><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><td>服务版本</td><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><td>数据库后端</td><td>{{ runtime?.dbBackend ?? "-" }}</td></tr>
              <tr><td>Redis 开关</td><td>{{ runtime?.redisEnabled ? "已启用" : "未启用" }}</td></tr>
              <tr><td>监听地址</td><td>{{ config?.bindAddress ?? "-" }}</td></tr>
              <tr><td>数据库连接</td><td>{{ config?.databaseUrl ?? "-" }}</td></tr>
              <tr><td>Redis 连接</td><td>{{ config?.redisUrl || "-" }}</td></tr>
              <tr><td>多点登录</td><td>{{ config?.multipointEnabled ? "开启" : "关闭" }}</td></tr>
              <tr>
                <td>兼容刷新头</td>
                <td>{{ config?.compatibilityRefreshHeaders ? "开启" : "关闭" }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">浏览器观测</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>User Agent</td><td>{{ browser.userAgent }}</td></tr>
              <tr><td>语言</td><td>{{ browser.language }}</td></tr>
              <tr><td>屏幕</td><td>{{ browser.screen }}</td></tr>
              <tr><td>时区</td><td>{{ browser.timezone }}</td></tr>
              <tr><td>在线状态</td><td>{{ browser.online ? "在线" : "离线" }}</td></tr>
              <tr><td>页面可见性</td><td>{{ browser.visibility }}</td></tr>
              <tr><td>硬件并发</td><td>{{ browser.hardwareConcurrency }}</td></tr>
              <tr><td>最新采样</td><td>{{ lastRefresh }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">工具链状态</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>模块</th>
                <th>状态</th>
                <th>摘要</th>
                <th>入口</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in toolRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.state }}</td>
                <td>{{ item.summary }}</td>
                <td><RouterLink class="btn ghost" :to="item.path">打开</RouterLink></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">即时操作</h3>
        <div class="action-grid">
          <button class="btn primary" :disabled="actionBusy" @click="startMcp">启动 MCP</button>
          <button class="btn ghost" :disabled="actionBusy" @click="stopMcp">停用 MCP</button>
          <button class="btn ghost" :disabled="actionBusy" @click="recoverResume">恢复续传</button>
          <button class="btn ghost" :disabled="actionBusy" @click="advanceResume">推进续传</button>
          <button class="btn ghost danger" :disabled="actionBusy" @click="clearTokens">清空令牌</button>
          <RouterLink class="btn ghost" to="/system/tools/system-config">系统配置</RouterLink>
          <RouterLink class="btn ghost" to="/system/tools/autocode/mcp-test">MCP 测试</RouterLink>
          <RouterLink class="btn ghost" to="/examples/breakpoint">断点续传</RouterLink>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">待处理队列</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>类别</th>
                <th>当前值</th>
                <th>建议动作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in pendingRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.value }}</td>
                <td>{{ item.action }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">健康标签</h3>
        <div class="tag-list">
          <span v-for="item in healthTags" :key="item" class="tag">{{ item }}</span>
        </div>
        <h3 class="title secondary-title">最近操作</h3>
        <ul class="activity-list">
          <li v-for="item in activityLog" :key="item.id">
            <strong>{{ item.title }}</strong>
            <span>{{ item.detail }}</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import {
  advanceResumeUploadApi,
  clearApiTokenApi,
  getApiTokenListApi,
  getAutoCodeRegistryApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getResumeUploadListApi,
  getRuntimeInfoApi,
  getScanSessionApi,
  getSkillListApi,
  getSystemConfigApi,
  recoverResumeUploadApi,
  startMcpServiceApi,
  stopMcpServiceApi,
} from "../../api/admin";
import type { McpServiceStatus, RuntimeInfo, SystemConfigInfo } from "../../types";

type ActivityEntry = {
  id: string;
  title: string;
  detail: string;
};

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcpStatus = ref<McpServiceStatus>({
  managed: true,
  state: "stopped",
  reachable: false,
  baseURL: "http://127.0.0.1:8889/mcp",
  healthURL: "http://127.0.0.1:8889/healthz",
  startedAt: null,
  lastError: "",
  message: "",
});
const metrics = reactive({
  mcpTools: 0,
  skills: 0,
  plugins: 0,
  autoCode: 0,
  tokens: 0,
  releases: 0,
  resumePending: 0,
  scanStatus: "-",
});
const browser = reactive({
  userAgent: navigator.userAgent,
  language: navigator.language,
  screen: `${window.screen.width} × ${window.screen.height}`,
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  online: navigator.onLine,
  visibility: document.visibilityState,
  hardwareConcurrency: navigator.hardwareConcurrency || 0,
});
const loading = ref(false);
const actionBusy = ref(false);
const autoRefresh = ref(true);
const refreshInterval = ref(10000);
const lastRefresh = ref("-");
const feedback = ref("");
const activityLog = ref<ActivityEntry[]>([]);

let timer: number | undefined;

const runtimeSummary = computed(() => ({
  os: runtime.value?.os ?? "-",
  version: runtime.value?.rustVersion ? `Rust ${runtime.value.rustVersion}` : "版本未知",
  storage: runtime.value?.dbBackend ?? "-",
  redis: runtime.value?.redisEnabled ? "Redis 已启用" : "Redis 未启用",
}));

const toolSummary = computed(() => ({
  mcp: `${metrics.mcpTools} / ${mcpStatus.value.reachable ? "online" : "offline"}`,
  mcpMessage: mcpStatus.value.message || "未登记说明",
}));

const pendingSummary = computed(() => {
  const total = metrics.resumePending + (mcpStatus.value.reachable ? 0 : 1);
  return {
    total,
    label: `${metrics.resumePending} 个续传任务待处理`,
  };
});

const runtimeBadge = computed(() =>
  runtime.value?.os && config.value?.bindAddress ? "runtime: healthy" : "runtime: partial",
);
const clientBadge = computed(() => (browser.online ? "client: online" : "client: offline"));

const toolRows = computed(() => [
  {
    name: "MCP 服务",
    state: mcpStatus.value.state,
    summary: `${metrics.mcpTools} 个工具，${mcpStatus.value.reachable ? "可达" : "未启动"}`,
    path: "/system/tools/autocode/mcp-test",
  },
  {
    name: "技能管理",
    state: metrics.skills > 0 ? "ready" : "empty",
    summary: `${metrics.skills} 个技能定义`,
    path: "/system/tools/skills",
  },
  {
    name: "插件安装",
    state: metrics.plugins > 0 ? "active" : "idle",
    summary: `${metrics.plugins} 条安装记录`,
    path: "/system/tools/install-plugin",
  },
  {
    name: "自动代码台账",
    state: metrics.autoCode > 0 ? "active" : "empty",
    summary: `${metrics.autoCode} 条蓝图`,
    path: "/system/tools/autocode-admin",
  },
  {
    name: "发布记录",
    state: metrics.releases > 0 ? "tracked" : "empty",
    summary: `${metrics.releases} 条版本记录`,
    path: "/system/tools/version",
  },
]);

const pendingRows = computed(() => [
  {
    name: "续传任务",
    value: `${metrics.resumePending} 项进行中`,
    action: metrics.resumePending ? "可直接点击“恢复续传”或“推进续传”" : "当前无阻塞任务",
  },
  {
    name: "扫码会话",
    value: metrics.scanStatus,
    action: metrics.scanStatus === "ready" ? "可切换到扫码上传页继续验收" : "检查扫码上传会话",
  },
  {
    name: "MCP 服务",
    value: mcpStatus.value.reachable ? "运行中" : "未启动",
    action: mcpStatus.value.reachable ? "可前往 MCP 测试页继续联调" : "可直接在本页启动服务",
  },
  {
    name: "API 令牌",
    value: `${metrics.tokens} 条记录`,
    action: metrics.tokens ? "如为脏数据可在本页清空" : "暂无待清理令牌",
  },
]);

const healthTags = computed(() => [
  runtime.value?.cpuCores ? `cpu: ${runtime.value.cpuCores}` : "cpu: unknown",
  runtime.value?.dbBackend ? `db: ${runtime.value.dbBackend}` : "db: unknown",
  runtime.value?.redisEnabled ? "redis: active" : "redis: inactive",
  config.value?.multipointEnabled ? "session: multipoint on" : "session: multipoint off",
  mcpStatus.value.reachable ? "mcp: reachable" : "mcp: stopped",
  metrics.skills ? `skills: ${metrics.skills}` : "skills: empty",
  browser.online ? "browser: online" : "browser: offline",
  metrics.resumePending ? `resume: ${metrics.resumePending} pending` : "resume: clear",
]);

function recordActivity(title: string, detail: string) {
  activityLog.value = [
    {
      id: `${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
      title,
      detail,
    },
    ...activityLog.value,
  ].slice(0, 8);
}

function updateBrowserSignals() {
  browser.online = navigator.onLine;
  browser.visibility = document.visibilityState;
}

function syncPolling() {
  if (timer) {
    window.clearInterval(timer);
    timer = undefined;
  }
  if (!autoRefresh.value) {
    return;
  }
  timer = window.setInterval(() => {
    void performReloadState(false);
  }, refreshInterval.value);
}

async function performReloadState(showMessage = true) {
  loading.value = true;
  updateBrowserSignals();
  try {
    const [
      nextRuntime,
      nextConfig,
      nextMcpStatus,
      nextMcpTools,
      nextSkills,
      nextPlugins,
      nextAutoCode,
      nextTokens,
      nextReleases,
      nextResume,
      nextScan,
    ] = await Promise.all([
      getRuntimeInfoApi(),
      getSystemConfigApi(),
      getMcpStatusApi(),
      getMcpToolListApi(),
      getSkillListApi(),
      getPluginInstallListApi(),
      getAutoCodeRegistryApi(),
      getApiTokenListApi(),
      getReleaseListApi(),
      getResumeUploadListApi(),
      getScanSessionApi(),
    ]);

    runtime.value = nextRuntime;
    config.value = nextConfig;
    mcpStatus.value = nextMcpStatus;
    metrics.mcpTools = nextMcpTools.length;
    metrics.skills = nextSkills.length;
    metrics.plugins = nextPlugins.Total;
    metrics.autoCode = nextAutoCode.Total;
    metrics.tokens = nextTokens.Total;
    metrics.releases = nextReleases.Total;
    metrics.resumePending = nextResume.List.filter((item) => item.progress < 100).length;
    metrics.scanStatus = nextScan.status;
    lastRefresh.value = new Date().toLocaleString("zh-CN");
    if (showMessage) {
      feedback.value = "运行态信息已刷新。";
      recordActivity("刷新状态", `已同步运行时、工具链与示例队列，时间 ${lastRefresh.value}`);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : "运行态刷新失败";
    feedback.value = message;
    recordActivity("刷新失败", message);
  } finally {
    loading.value = false;
  }
}

async function reloadState() {
  await performReloadState(true);
}

async function runAction(title: string, handler: () => Promise<unknown>, successDetail: string) {
  actionBusy.value = true;
  try {
    await handler();
    feedback.value = successDetail;
    recordActivity(title, successDetail);
    await performReloadState(false);
  } catch (error) {
    const message = error instanceof Error ? error.message : `${title}失败`;
    feedback.value = message;
    recordActivity(`${title}失败`, message);
  } finally {
    actionBusy.value = false;
  }
}

async function startMcp() {
  await runAction("启动 MCP", async () => {
    mcpStatus.value = await startMcpServiceApi();
  }, "MCP 服务已启动，并已刷新页面状态。");
}

async function stopMcp() {
  await runAction("停用 MCP", async () => {
    mcpStatus.value = await stopMcpServiceApi();
  }, "MCP 服务已停用，并已刷新页面状态。");
}

async function recoverResume() {
  await runAction(
    "恢复续传",
    async () => {
      await recoverResumeUploadApi();
    },
    "已触发续传恢复，并重新拉取队列状态。",
  );
}

async function advanceResume() {
  await runAction(
    "推进续传",
    async () => {
      await advanceResumeUploadApi();
    },
    "已推进续传任务，并重新拉取队列状态。",
  );
}

async function clearTokens() {
  await runAction(
    "清空令牌",
    async () => {
      await clearApiTokenApi();
    },
    "API Token 台账已清空，并重新同步运行态。",
  );
}

watch([autoRefresh, refreshInterval], () => {
  syncPolling();
});

onMounted(async () => {
  await performReloadState();
  syncPolling();
  window.addEventListener("online", updateBrowserSignals);
  window.addEventListener("offline", updateBrowserSignals);
  document.addEventListener("visibilitychange", updateBrowserSignals);
});

onUnmounted(() => {
  window.removeEventListener("online", updateBrowserSignals);
  window.removeEventListener("offline", updateBrowserSignals);
  document.removeEventListener("visibilitychange", updateBrowserSignals);
  if (timer) {
    window.clearInterval(timer);
  }
});
</script>

<style scoped>
.toolbar {
  align-items: flex-start;
  display: flex;
  gap: 16px;
  justify-content: space-between;
}

.toolbar-actions {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.inline-field {
  align-items: center;
  display: inline-flex;
  gap: 8px;
}

.inline-field select {
  min-width: 92px;
}

.action-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.activity-list {
  display: grid;
  gap: 10px;
  margin: 16px 0 0;
  padding-left: 18px;
}

.activity-list li {
  display: grid;
  gap: 4px;
}

.danger {
  border-color: #ef4444;
  color: #b91c1c;
}

.secondary-title {
  margin-top: 20px;
}

@media (max-width: 960px) {
  .toolbar {
    flex-direction: column;
  }

  .action-grid {
    grid-template-columns: 1fr;
  }
}
</style>
