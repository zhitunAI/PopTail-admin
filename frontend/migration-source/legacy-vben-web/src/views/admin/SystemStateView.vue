<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">系统状态</h3>
          <p class="subtitle">
            聚合运行时、系统配置、工具链与最近日志，作为环境/配置/健康观测页使用。
          </p>
        </div>
        <div class="row wrap">
          <label class="inline-check">
            <input v-model="autoRefresh" type="checkbox" />
            自动刷新
          </label>
          <button class="btn ghost" :disabled="loading" @click="reloadState">
            {{ loading ? "刷新中..." : "立即刷新" }}
          </button>
        </div>
      </div>
      <p class="subtitle">
        最近刷新：{{ lastRefresh }}
        <span v-if="loadError">｜最近错误：{{ loadError }}</span>
      </p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>总体健康</h4>
        <p class="stat-value">{{ overview.healthScore }}%</p>
        <p class="subtitle">{{ overview.healthLabel }}</p>
      </div>
      <div class="stat-card">
        <h4>服务就绪</h4>
        <p class="stat-value">{{ overview.readyServices }}/{{ serviceRows.length }}</p>
        <p class="subtitle">运行链路可用项</p>
      </div>
      <div class="stat-card">
        <h4>配置风险</h4>
        <p class="stat-value">{{ overview.riskCount }}</p>
        <p class="subtitle">需关注项</p>
      </div>
      <div class="stat-card">
        <h4>活动信号</h4>
        <p class="stat-value">{{ overview.activityCount }}</p>
        <p class="subtitle">最近日志 / 台账汇总</p>
      </div>
    </div>

    <div class="split-grid status-layout">
      <div class="card">
        <h3 class="title">服务健康矩阵</h3>
        <p class="subtitle">用真实接口回填当前服务、存储、会话与工具链状态。</p>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>组件</th>
                <th>状态</th>
                <th>说明</th>
                <th>数据源</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in serviceRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>
                  <span class="tag" :class="statusClass(item.level)">{{ item.status }}</span>
                </td>
                <td>{{ item.detail }}</td>
                <td>{{ item.source }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">风险与建议</h3>
        <p class="subtitle">根据当前配置和运行信号自动归纳，不再只是静态文案。</p>
        <div class="stack compact">
          <div v-if="risks.length" class="tag-list">
            <span v-for="item in risks" :key="item" class="tag warn">{{ item }}</span>
          </div>
          <p v-else class="subtitle">当前未发现明显配置风险。</p>
          <div class="data-table">
            <table>
              <tbody>
                <tr><td>建议优先级</td><td>{{ overview.riskCount > 0 ? "先处理风险项" : "保持巡检频率" }}</td></tr>
                <tr><td>会话策略</td><td>{{ config?.multipointEnabled ? "多点登录已开启" : "单点会话模式" }}</td></tr>
                <tr><td>工具链状态</td><td>{{ mcpStatus?.message ?? "未接入" }}</td></tr>
                <tr><td>浏览器联通</td><td>{{ browser.online ? "在线" : "离线" }}</td></tr>
                <tr><td>日志信号</td><td>{{ logSummary }}</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>

    <div class="split-grid status-layout">
      <div class="card">
        <h3 class="title">环境与配置快照</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>服务端操作系统</td><td>{{ runtime?.os ?? "-" }}</td></tr>
              <tr><td>CPU 核数</td><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><td>Rust 版本</td><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><td>数据库后端</td><td>{{ runtime?.dbBackend ?? "-" }}</td></tr>
              <tr><td>Bind Address</td><td>{{ config?.bindAddress ?? "-" }}</td></tr>
              <tr><td>Database URL</td><td>{{ maskedDatabaseUrl }}</td></tr>
              <tr><td>Redis URL</td><td>{{ maskedRedisUrl }}</td></tr>
              <tr><td>Compatibility Headers</td><td>{{ config?.compatibilityRefreshHeaders ? "enabled" : "disabled" }}</td></tr>
              <tr><td>配置来源</td><td>/system/getSystemConfig</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">浏览器与前端观测</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>User Agent</td><td>{{ browser.userAgent }}</td></tr>
              <tr><td>Language</td><td>{{ browser.language }}</td></tr>
              <tr><td>Screen</td><td>{{ browser.screen }}</td></tr>
              <tr><td>Timezone</td><td>{{ browser.timezone }}</td></tr>
              <tr><td>Online</td><td>{{ browser.online ? "true" : "false" }}</td></tr>
              <tr><td>当前路径</td><td>{{ browser.path }}</td></tr>
              <tr><td>Token 状态</td><td>{{ browser.hasToken ? "已写入" : "未发现" }}</td></tr>
              <tr><td>刷新策略</td><td>{{ autoRefresh ? "10 秒轮询" : "手动刷新" }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="split-grid status-layout">
      <div class="card">
        <h3 class="title">能力台账</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>模块</th>
                <th>数量</th>
                <th>摘要</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in inventoryRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.count }}</td>
                <td>{{ item.detail }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">最近活动</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>信号</th>
                <th>详情</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in activityRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.detail }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="title">健康标签</h3>
      <div class="tag-list">
        <span class="tag" :class="statusClass(overview.healthScore >= 85 ? 'good' : overview.healthScore >= 60 ? 'warn' : 'bad')">
          health: {{ overview.healthLabel }}
        </span>
        <span class="tag" :class="statusClass(runtime?.cpuCores ? 'good' : 'warn')">
          cpu: {{ runtime?.cpuCores ? `${runtime.cpuCores} cores` : "unknown" }}
        </span>
        <span class="tag" :class="statusClass(runtime?.dbBackend ? 'good' : 'bad')">
          storage: {{ runtime?.dbBackend ?? "unknown" }}
        </span>
        <span class="tag" :class="statusClass(config?.multipointEnabled ? 'good' : 'warn')">
          session: {{ config?.multipointEnabled ? "multipoint on" : "multipoint off" }}
        </span>
        <span class="tag" :class="statusClass(mcpStatus?.reachable ? 'good' : 'warn')">
          mcp: {{ mcpStatus?.state ?? "unknown" }}
        </span>
        <span class="tag" :class="statusClass(browser.online ? 'good' : 'bad')">
          client: {{ browser.online ? "online" : "offline" }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import {
  getApiTokenListApi,
  getAutoCodeRegistryApi,
  getErrorLogsApi,
  getLoginLogsApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getOperationLogsApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getRuntimeInfoApi,
  getSkillListApi,
  getSystemConfigApi,
} from "../../api/admin";
import type {
  ErrorLogInfo,
  LoginLogInfo,
  McpServiceStatus,
  OperationLogInfo,
  RuntimeInfo,
  SystemConfigInfo,
} from "../../types";

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcpStatus = ref<McpServiceStatus | null>(null);
const loading = ref(false);
const loadError = ref("");
const lastRefresh = ref("-");
const autoRefresh = ref(true);
const counts = reactive({
  apiTokens: 0,
  plugins: 0,
  autoCode: 0,
  skills: 0,
  mcpTools: 0,
  releases: 0,
});
const recent = reactive<{
  operations: OperationLogInfo[];
  logins: LoginLogInfo[];
  errors: ErrorLogInfo[];
}>({
  operations: [],
  logins: [],
  errors: [],
});
const browser = reactive({
  userAgent: navigator.userAgent,
  language: navigator.language,
  screen: `${window.screen.width} x ${window.screen.height}`,
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  online: navigator.onLine,
  path: window.location.pathname,
  hasToken: Boolean(localStorage.getItem("token")),
});

let timer: number | undefined;

const serviceRows = computed(() => {
  const rows = [
    {
      name: "Rust Server",
      level: runtime.value?.rustVersion ? "good" : "bad",
      status: runtime.value?.rustVersion ? "ready" : "unknown",
      detail: runtime.value?.rustVersion
        ? `运行于 ${runtime.value.os}，版本 ${runtime.value.rustVersion}`
        : "未读取到运行时信息",
      source: "/system/getServerInfo",
    },
    {
      name: "Database",
      level: runtime.value?.dbBackend ? "good" : "bad",
      status: runtime.value?.dbBackend ?? "unknown",
      detail: config.value?.databaseUrl
        ? `连接目标 ${maskedDatabaseUrl.value}`
        : "缺少数据库连接配置",
      source: "/system/getServerInfo + /system/getSystemConfig",
    },
    {
      name: "Redis",
      level: runtime.value?.redisEnabled ? "good" : config.value?.redisUrl ? "warn" : "bad",
      status: runtime.value?.redisEnabled ? "enabled" : "disabled",
      detail: config.value?.redisUrl ? maskedRedisUrl.value : "未配置 Redis URL",
      source: "/system/getServerInfo + /system/getSystemConfig",
    },
    {
      name: "Session Policy",
      level: config.value?.multipointEnabled ? "good" : "warn",
      status: config.value?.multipointEnabled ? "multipoint" : "single",
      detail: config.value?.compatibilityRefreshHeaders
        ? "刷新头兼容已开启"
        : "刷新头兼容已关闭",
      source: "/system/getSystemConfig",
    },
    {
      name: "MCP Service",
      level: mcpStatus.value?.reachable ? "good" : "warn",
      status: mcpStatus.value?.state ?? "unknown",
      detail: mcpStatus.value?.message ?? "未接入 MCP 状态",
      source: "/autoCode/mcpStatus",
    },
    {
      name: "Browser Client",
      level: browser.online ? "good" : "bad",
      status: browser.online ? "online" : "offline",
      detail: browser.hasToken ? "检测到本地 token" : "未检测到本地 token",
      source: "Navigator / localStorage",
    },
  ];

  return rows;
});

const risks = computed(() => {
  const items: string[] = [];
  if (!config.value?.databaseUrl) {
    items.push("缺少数据库连接地址，持久化链路不可核对");
  }
  if (!config.value?.redisUrl) {
    items.push("未配置 Redis URL，多点登录/失效广播无法接线 Docker Redis");
  }
  if (!runtime.value?.redisEnabled && config.value?.multipointEnabled) {
    items.push("多点登录已开启但 Redis 运行态未启用");
  }
  if (!browser.online) {
    items.push("当前浏览器离线，前端观测存在盲区");
  }
  if ((recent.errors[0]?.status ?? "").includes("待处理")) {
    items.push("存在待处理错误日志，建议先排查最新错误");
  }
  if (!mcpStatus.value?.reachable && counts.mcpTools > 0) {
    items.push("MCP 已登记工具但服务不可达，联调链路未闭环");
  }
  return items;
});

const overview = computed(() => {
  const readyServices = serviceRows.value.filter((item) => item.level === "good").length;
  const total = serviceRows.value.length || 1;
  const riskPenalty = risks.value.length * 8;
  const serviceScore = Math.round((readyServices / total) * 100);
  const healthScore = Math.max(0, Math.min(100, serviceScore - riskPenalty));
  return {
    readyServices,
    riskCount: risks.value.length,
    activityCount: counts.apiTokens + counts.plugins + counts.autoCode + recent.operations.length,
    healthScore,
    healthLabel:
      healthScore >= 85 ? "稳定" : healthScore >= 60 ? "需关注" : "风险较高",
  };
});

const inventoryRows = computed(() => [
  {
    name: "API Tokens",
    count: counts.apiTokens,
    detail: counts.apiTokens ? "已登记令牌，可支撑工具接入" : "尚未登记令牌",
  },
  {
    name: "Plugin Installs",
    count: counts.plugins,
    detail: counts.plugins ? "存在已安装插件记录" : "暂无插件安装记录",
  },
  {
    name: "Auto Code Blueprints",
    count: counts.autoCode,
    detail: counts.autoCode ? "自动代码台账已落库" : "尚未登记自动代码蓝图",
  },
  {
    name: "Skills",
    count: counts.skills,
    detail: counts.skills ? "技能清单已接后端" : "暂无技能定义",
  },
  {
    name: "MCP Tools",
    count: counts.mcpTools,
    detail: counts.mcpTools ? "存在可测工具定义" : "暂无 MCP 工具定义",
  },
  {
    name: "Releases",
    count: counts.releases,
    detail: counts.releases ? "存在发布记录" : "暂无发布记录",
  },
]);

const activityRows = computed(() => {
  const latestOperation = recent.operations[0];
  const latestLogin = recent.logins[0];
  const latestError = recent.errors[0];
  return [
    {
      name: "最近操作日志",
      detail: latestOperation
        ? `${latestOperation.method} ${latestOperation.path} · ${latestOperation.status} · ${latestOperation.latency}ms`
        : "暂无操作日志",
    },
    {
      name: "最近登录日志",
      detail: latestLogin
        ? `${latestLogin.username} · ${latestLogin.status ? "成功" : "失败"} · ${latestLogin.ip}`
        : "暂无登录日志",
    },
    {
      name: "最近错误日志",
      detail: latestError
        ? `${latestError.path} · ${latestError.status} · ${latestError.error}`
        : "暂无错误日志",
    },
    {
      name: "MCP 观测",
      detail: mcpStatus.value
        ? `${mcpStatus.value.state} · ${mcpStatus.value.baseURL}`
        : "暂无 MCP 状态",
    },
  ];
});

const logSummary = computed(() => {
  const errorCount = recent.errors.length;
  const failedLoginCount = recent.logins.filter((item) => !item.status).length;
  return `操作 ${recent.operations.length} / 登录失败 ${failedLoginCount} / 错误 ${errorCount}`;
});

const maskedDatabaseUrl = computed(() => sanitizeConnection(config.value?.databaseUrl));
const maskedRedisUrl = computed(() => sanitizeConnection(config.value?.redisUrl));

function sanitizeConnection(value?: string) {
  if (!value) {
    return "-";
  }
  try {
    const parsed = new URL(value);
    const auth = parsed.username ? `${parsed.username}:***@` : "";
    const path = parsed.pathname && parsed.pathname !== "/" ? parsed.pathname : "";
    return `${parsed.protocol}//${auth}${parsed.host}${path}`;
  } catch {
    return value.replace(/:\/\/([^:@/]+):([^@/]+)@/, "://$1:***@");
  }
}

function statusClass(level: string) {
  return {
    ok: level === "good",
    warn: level === "warn",
    bad: level === "bad",
  };
}

async function reloadState() {
  loading.value = true;
  loadError.value = "";
  const results = await Promise.allSettled([
    getRuntimeInfoApi(),
    getSystemConfigApi(),
    getMcpStatusApi(),
    getApiTokenListApi(),
    getPluginInstallListApi(),
    getAutoCodeRegistryApi(),
    getSkillListApi(),
    getMcpToolListApi(),
    getReleaseListApi(),
    getOperationLogsApi(),
    getLoginLogsApi(),
    getErrorLogsApi(),
  ]);

  const [
    runtimeResult,
    configResult,
    mcpStatusResult,
    tokenResult,
    pluginResult,
    autoCodeResult,
    skillResult,
    mcpToolsResult,
    releaseResult,
    operationResult,
    loginResult,
    errorResult,
  ] = results;

  applySettled(runtimeResult, (value) => {
    runtime.value = value;
  });
  applySettled(configResult, (value) => {
    config.value = value;
  });
  applySettled(mcpStatusResult, (value) => {
    mcpStatus.value = value;
  });
  applySettled(tokenResult, (value) => {
    counts.apiTokens = value.Total;
  });
  applySettled(pluginResult, (value) => {
    counts.plugins = value.Total;
  });
  applySettled(autoCodeResult, (value) => {
    counts.autoCode = value.Total;
  });
  applySettled(skillResult, (value) => {
    counts.skills = value.length;
  });
  applySettled(mcpToolsResult, (value) => {
    counts.mcpTools = value.length;
  });
  applySettled(releaseResult, (value) => {
    counts.releases = value.Total;
  });
  applySettled(operationResult, (value) => {
    recent.operations = value.List.slice(0, 5);
  });
  applySettled(loginResult, (value) => {
    recent.logins = value.List.slice(0, 5);
  });
  applySettled(errorResult, (value) => {
    recent.errors = value.List.slice(0, 5);
  });

  const rejected = results.find((item) => item.status === "rejected") as PromiseRejectedResult | undefined;
  if (rejected) {
    loadError.value = rejected.reason instanceof Error ? rejected.reason.message : "部分数据加载失败";
  }

  browser.hasToken = Boolean(localStorage.getItem("token"));
  browser.path = window.location.pathname;
  lastRefresh.value = new Date().toLocaleString("zh-CN");
  loading.value = false;
}

function applySettled<T>(
  result: PromiseSettledResult<T>,
  onFulfilled: (value: T) => void,
) {
  if (result.status === "fulfilled") {
    onFulfilled(result.value);
  }
}

function syncOnlineState() {
  browser.online = navigator.onLine;
}

function startTimer() {
  stopTimer();
  if (!autoRefresh.value) {
    return;
  }
  timer = window.setInterval(() => {
    void reloadState();
  }, 10000);
}

function stopTimer() {
  if (timer) {
    window.clearInterval(timer);
    timer = undefined;
  }
}

watch(autoRefresh, () => {
  startTimer();
});

onMounted(async () => {
  await reloadState();
  window.addEventListener("online", syncOnlineState);
  window.addEventListener("offline", syncOnlineState);
  startTimer();
});

onUnmounted(() => {
  stopTimer();
  window.removeEventListener("online", syncOnlineState);
  window.removeEventListener("offline", syncOnlineState);
});
</script>

<style scoped>
.space-between {
  justify-content: space-between;
}

.wrap {
  flex-wrap: wrap;
}

.status-layout {
  align-items: start;
}

.inline-check {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: #475569;
}

.compact {
  gap: 12px;
}

.tag.ok {
  background: #dcfce7;
  color: #166534;
}

.tag.warn {
  background: #fef3c7;
  color: #92400e;
}

.tag.bad {
  background: #fee2e2;
  color: #991b1b;
}
</style>
