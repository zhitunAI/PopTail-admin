<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">关于系统</h3>
          <p class="subtitle">
            基于当前 Rust / Vben 实际数据生成项目、架构与交付概览，不再只是静态说明页。
          </p>
        </div>
        <button class="btn ghost" :disabled="loading" @click="reloadOverview">
          {{ loading ? "刷新中..." : "刷新概览" }}
        </button>
      </div>
      <p class="subtitle">
        最近同步：{{ lastRefresh }}
        <span v-if="loadError">｜最近错误：{{ loadError }}</span>
      </p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>恢复域数</h4>
        <p class="stat-value">{{ overview.restoredDomains }}</p>
        <p class="subtitle">已接入真实读写/状态化模块</p>
      </div>
      <div class="stat-card">
        <h4>架构就绪度</h4>
        <p class="stat-value">{{ overview.architectureScore }}%</p>
        <p class="subtitle">{{ architectureLabel }}</p>
      </div>
      <div class="stat-card">
        <h4>交付信号</h4>
        <p class="stat-value">{{ overview.deliverySignals }}</p>
        <p class="subtitle">发布 / 自动代码 / 插件 / 技能</p>
      </div>
      <div class="stat-card">
        <h4>运行观测</h4>
        <p class="stat-value">{{ overview.observabilitySignals }}</p>
        <p class="subtitle">日志 / 队列 / MCP / 浏览器在线</p>
      </div>
    </div>

    <div class="split-grid about-layout">
      <div class="card">
        <h3 class="title">项目基线</h3>
        <p class="subtitle">页面根据实际运行态和仓库当前约束组织展示。</p>
        <div class="data-table">
          <table>
            <tbody>
              <tr><th>项目代号</th><td>gin-ai-admin</td></tr>
              <tr><th>当前路径</th><td>{{ browser.path }}</td></tr>
              <tr><th>前端实现</th><td>Vue 3 + Vite + Pinia + Vben 风格页面</td></tr>
              <tr><th>后端实现</th><td>Rust + Axum + 状态化管理接口</td></tr>
              <tr><th>运行平台</th><td>{{ runtime?.os ?? "未获取" }}</td></tr>
              <tr><th>CPU 核数</th><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><th>Rust 版本</th><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><th>数据库后端</th><td>{{ runtime?.dbBackend ?? "-" }}</td></tr>
              <tr><th>Redis 能力</th><td>{{ runtime?.redisEnabled ? "已启用" : "未启用" }}</td></tr>
              <tr><th>服务监听</th><td>{{ config?.bindAddress ?? "-" }}</td></tr>
              <tr><th>数据库连接</th><td>{{ maskedDatabaseUrl }}</td></tr>
              <tr><th>Redis 连接</th><td>{{ maskedRedisUrl }}</td></tr>
              <tr><th>桌面路线</th><td>继续保留 Rust + Tauri 2 重构路线</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">架构与约束</h3>
        <p class="subtitle">由当前配置、服务状态与既有约束共同形成。</p>
        <div class="tag-list">
          <span class="tag success">功能 1:1 对齐目标</span>
          <span class="tag success">clean-room 命名</span>
          <span class="tag success">不改 legacy server/web</span>
          <span class="tag" :class="runtime?.redisEnabled ? 'success' : 'warn'">
            Redis {{ runtime?.redisEnabled ? "ready" : "pending" }}
          </span>
          <span class="tag" :class="config?.multipointEnabled ? 'success' : 'warn'">
            {{ config?.multipointEnabled ? "多点登录开启" : "单点会话模式" }}
          </span>
          <span class="tag" :class="mcpRunning ? 'success' : 'warn'">
            MCP {{ mcpRunning ? "running" : "not running" }}
          </span>
          <span class="tag" :class="browser.online ? 'success' : 'danger'">
            浏览器{{ browser.online ? "在线" : "离线" }}
          </span>
        </div>
        <div class="data-table">
          <table>
            <tbody>
              <tr><th>会话策略</th><td>{{ config?.multipointEnabled ? "多点并发会话已接入" : "当前仍按单点行为工作" }}</td></tr>
              <tr><th>刷新兼容</th><td>{{ config?.compatibilityRefreshHeaders ? "兼容刷新头已开启" : "兼容刷新头关闭" }}</td></tr>
              <tr><th>MCP 服务</th><td>{{ mcpStatusText }}</td></tr>
              <tr><th>技能资产</th><td>{{ metrics.skills }} 项技能 / {{ metrics.skillTools }} 个工具定义</td></tr>
              <tr><th>插件能力</th><td>{{ metrics.pluginInstalls }} 条安装记录 / {{ metrics.packages }} 个包定义</td></tr>
              <tr><th>自动代码</th><td>{{ metrics.autoCode }} 条台账 / {{ metrics.mcpTools }} 个 MCP 工具</td></tr>
              <tr><th>示例链路</th><td>{{ exampleSummary }}</td></tr>
              <tr><th>交付基线</th><td>{{ latestReleaseText }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="title">恢复覆盖矩阵</h3>
      <p class="subtitle">通过现有后端接口回填关键域规模，用于判断 1:1 恢复推进程度。</p>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>域</th>
              <th>规模</th>
              <th>当前信号</th>
              <th>数据来源</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in coverageRows" :key="row.name">
              <td>{{ row.name }}</td>
              <td>{{ row.count }}</td>
              <td>{{ row.detail }}</td>
              <td>{{ row.source }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="split-grid about-layout">
      <div class="card">
        <h3 class="title">交付与运行看板</h3>
        <p class="subtitle">用最近发布、日志、队列状态生成当前交付快照。</p>
        <div class="data-table">
          <table>
            <tbody>
              <tr><th>最近发布</th><td>{{ latestReleaseText }}</td></tr>
              <tr><th>最近操作</th><td>{{ latestOperationText }}</td></tr>
              <tr><th>最近登录</th><td>{{ latestLoginText }}</td></tr>
              <tr><th>最近错误</th><td>{{ latestErrorText }}</td></tr>
              <tr><th>上传队列</th><td>{{ uploadSummary }}</td></tr>
              <tr><th>断点续传</th><td>{{ resumeSummary }}</td></tr>
              <tr><th>扫码会话</th><td>{{ scanSummary }}</td></tr>
              <tr><th>Token 台账</th><td>{{ tokenSummary }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">交付说明</h3>
        <p class="subtitle">不是版权说明，而是当前重构版的真实交付边界与事实摘要。</p>
        <div class="stack compact">
          <div class="tag-list">
            <span v-for="note in deliveryNotes" :key="note" class="tag">
              {{ note }}
            </span>
          </div>
          <div class="data-table">
            <table>
              <tbody>
                <tr><th>前端在线状态</th><td>{{ browser.online ? "online" : "offline" }}</td></tr>
                <tr><th>浏览器环境</th><td>{{ browser.language }} / {{ browser.timezone }}</td></tr>
                <tr><th>屏幕尺寸</th><td>{{ browser.screen }}</td></tr>
                <tr><th>能力路线</th><td>Web 先恢复一致行为，后续进入 Rust + Tauri 2</td></tr>
                <tr><th>数据面</th><td>{{ maskedDatabaseUrl }} ｜ {{ maskedRedisUrl }}</td></tr>
                <tr><th>变更基调</th><td>最小必要依赖、避免回退并行代理工作</td></tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  getApiListApi,
  getApiTokenListApi,
  getAuthorityListApi,
  getAutoCodeRegistryApi,
  getDictionaryListApi,
  getErrorLogsApi,
  getLoginLogsApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getMenuListApi,
  getOperationLogsApi,
  getPackageListApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getResumeUploadListApi,
  getRuntimeInfoApi,
  getScanSessionApi,
  getSkillListApi,
  getSkillToolsApi,
  getSystemConfigApi,
  getUploadQueueApi,
  getUserListApi,
} from "../api/admin";
import type {
  ErrorLogInfo,
  LoginLogInfo,
  McpServiceStatus,
  OperationLogInfo,
  ReleaseRecord,
  ResumeUploadRecord,
  RuntimeInfo,
  ScanSessionRecord,
  SystemConfigInfo,
  UploadFileRecord,
} from "../types";

const loading = ref(false);
const loadError = ref("");
const lastRefresh = ref("-");

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcpStatus = ref<McpServiceStatus | null>(null);
const releases = ref<ReleaseRecord[]>([]);
const uploads = ref<UploadFileRecord[]>([]);
const resumes = ref<ResumeUploadRecord[]>([]);
const scanSession = ref<ScanSessionRecord | null>(null);
const operations = ref<OperationLogInfo[]>([]);
const logins = ref<LoginLogInfo[]>([]);
const errors = ref<ErrorLogInfo[]>([]);

const metrics = reactive({
  users: 0,
  authorities: 0,
  menus: 0,
  apis: 0,
  dictionaries: 0,
  packages: 0,
  pluginInstalls: 0,
  autoCode: 0,
  apiTokens: 0,
  skills: 0,
  skillTools: 0,
  mcpTools: 0,
});

const browser = reactive({
  path: window.location.pathname,
  online: navigator.onLine,
  language: navigator.language,
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  screen: `${window.innerWidth} × ${window.innerHeight}`,
});

function formatDate(value?: number | null) {
  if (!value) return "-";
  return new Date(value).toLocaleString();
}

function maskConnectionString(value?: string) {
  if (!value) return "-";
  return value.replace(/:\/\/([^:@/]+)(?::([^@/]+))?@/, "://***:***@");
}

function settledLength<T>(result: PromiseSettledResult<{ List: T[] } | T[]>) {
  if (result.status !== "fulfilled") return 0;
  const value = result.value;
  return Array.isArray(value) ? value.length : value.List.length;
}

async function reloadOverview() {
  loading.value = true;
  loadError.value = "";
  browser.path = window.location.pathname;
  browser.online = navigator.onLine;
  browser.screen = `${window.innerWidth} × ${window.innerHeight}`;

  const results = await Promise.allSettled([
    getRuntimeInfoApi(),
    getSystemConfigApi(),
    getMcpStatusApi(),
    getUserListApi(),
    getAuthorityListApi(),
    getMenuListApi(),
    getApiListApi(),
    getDictionaryListApi(),
    getPackageListApi(),
    getPluginInstallListApi(),
    getAutoCodeRegistryApi(),
    getApiTokenListApi(),
    getSkillListApi(),
    getSkillToolsApi(),
    getMcpToolListApi(),
    getReleaseListApi(),
    getUploadQueueApi(),
    getResumeUploadListApi(),
    getScanSessionApi(),
    getOperationLogsApi(),
    getLoginLogsApi(),
    getErrorLogsApi(),
  ]);

  const [
    runtimeResult,
    configResult,
    mcpResult,
    userResult,
    authorityResult,
    menuResult,
    apiResult,
    dictionaryResult,
    packageResult,
    pluginResult,
    autoCodeResult,
    tokenResult,
    skillResult,
    skillToolResult,
    mcpToolResult,
    releaseResult,
    uploadResult,
    resumeResult,
    scanResult,
    operationResult,
    loginResult,
    errorResult,
  ] = results;

  if (runtimeResult.status === "fulfilled") runtime.value = runtimeResult.value;
  if (configResult.status === "fulfilled") config.value = configResult.value;
  if (mcpResult.status === "fulfilled") mcpStatus.value = mcpResult.value;
  if (releaseResult.status === "fulfilled") releases.value = releaseResult.value.List;
  if (uploadResult.status === "fulfilled") uploads.value = uploadResult.value.List;
  if (resumeResult.status === "fulfilled") resumes.value = resumeResult.value.List;
  if (scanResult.status === "fulfilled") scanSession.value = scanResult.value;
  if (operationResult.status === "fulfilled") operations.value = operationResult.value.List;
  if (loginResult.status === "fulfilled") logins.value = loginResult.value.List;
  if (errorResult.status === "fulfilled") errors.value = errorResult.value.List;

  metrics.users = settledLength(userResult as PromiseSettledResult<{ List: unknown[] }>);
  metrics.authorities = settledLength(
    authorityResult as PromiseSettledResult<unknown[]>,
  );
  metrics.menus = settledLength(menuResult as PromiseSettledResult<{ List: unknown[] }>);
  metrics.apis = settledLength(apiResult as PromiseSettledResult<{ List: unknown[] }>);
  metrics.dictionaries = settledLength(
    dictionaryResult as PromiseSettledResult<{ List: unknown[] }>,
  );
  metrics.packages = settledLength(packageResult as PromiseSettledResult<{ List: unknown[] }>);
  metrics.pluginInstalls = settledLength(
    pluginResult as PromiseSettledResult<{ List: unknown[] }>,
  );
  metrics.autoCode = settledLength(
    autoCodeResult as PromiseSettledResult<{ List: unknown[] }>,
  );
  metrics.apiTokens = settledLength(tokenResult as PromiseSettledResult<{ List: unknown[] }>);
  metrics.skills = settledLength(skillResult as PromiseSettledResult<unknown[]>);
  metrics.skillTools = skillToolResult.status === "fulfilled" ? skillToolResult.value.length : 0;
  metrics.mcpTools = mcpToolResult.status === "fulfilled" ? mcpToolResult.value.length : 0;

  const firstRejected = results.find(
    (entry): entry is PromiseRejectedResult => entry.status === "rejected",
  );
  if (firstRejected) {
    loadError.value =
      firstRejected.reason instanceof Error
        ? firstRejected.reason.message
        : String(firstRejected.reason);
  }

  lastRefresh.value = new Date().toLocaleString();
  loading.value = false;
}

const maskedDatabaseUrl = computed(() => maskConnectionString(config.value?.databaseUrl));
const maskedRedisUrl = computed(() => maskConnectionString(config.value?.redisUrl));

const mcpRunning = computed(
  () => mcpStatus.value?.reachable && ["running", "ready", "online"].includes(mcpStatus.value.state),
);

const architectureScore = computed(() => {
  let score = 35;
  if (runtime.value?.dbBackend) score += 15;
  if (runtime.value?.redisEnabled) score += 15;
  if (config.value?.bindAddress) score += 10;
  if (config.value?.databaseUrl) score += 10;
  if (config.value?.redisUrl) score += 5;
  if (mcpRunning.value) score += 10;
  return Math.min(score, 100);
});

const architectureLabel = computed(() => {
  if (architectureScore.value >= 85) return "数据库、Redis、MCP 与配置链路整体可见";
  if (architectureScore.value >= 60) return "基础架构已成型，仍有若干链路待补强";
  return "当前仍处于基础设施补齐阶段";
});

const overview = computed(() => ({
  restoredDomains: [
    metrics.users,
    metrics.authorities,
    metrics.menus,
    metrics.apis,
    metrics.dictionaries,
    metrics.skills,
    metrics.packages,
    metrics.pluginInstalls,
    metrics.autoCode,
  ].filter((count) => count > 0).length,
  architectureScore: architectureScore.value,
  deliverySignals:
    releases.value.length +
    metrics.autoCode +
    metrics.pluginInstalls +
    metrics.skills,
  observabilitySignals:
    operations.value.length +
    logins.value.length +
    errors.value.length +
    resumes.value.length +
    (mcpRunning.value ? 1 : 0) +
    (browser.online ? 1 : 0),
}));

const mcpStatusText = computed(() => {
  if (!mcpStatus.value) return "未获取状态";
  return `${mcpStatus.value.state}｜${mcpStatus.value.message || "无附加说明"}`;
});

const exampleSummary = computed(() => {
  const uploadPending = uploads.value.filter((item) => item.status !== "done").length;
  const resumePending = resumes.value.filter((item) => item.status !== "done").length;
  const scanText = scanSession.value ? `${scanSession.value.sessionId} / ${scanSession.value.status}` : "未发现会话";
  return `上传待处理 ${uploadPending}，续传待处理 ${resumePending}，扫码 ${scanText}`;
});

const latestReleaseText = computed(() => {
  const latest = releases.value[0];
  return latest ? `${latest.note}（${formatDate(latest.createdAt)}）` : "暂无发布记录";
});

const latestOperationText = computed(() => {
  const latest = operations.value[0];
  return latest
    ? `${latest.method} ${latest.path}｜status ${latest.status}｜${latest.ip}`
    : "暂无操作日志";
});

const latestLoginText = computed(() => {
  const latest = logins.value[0];
  return latest
    ? `${latest.username}｜${latest.status ? "成功" : "失败"}｜${latest.ip}${
        latest.errorMessage ? `｜${latest.errorMessage}` : ""
      }`
    : "暂无登录日志";
});

const latestErrorText = computed(() => {
  const latest = errors.value[0];
  return latest ? `${latest.path}｜${latest.error}｜${latest.status}` : "暂无错误日志";
});

const uploadSummary = computed(() => {
  if (!uploads.value.length) return "当前没有上传队列";
  const done = uploads.value.filter((item) => item.status === "done").length;
  return `${uploads.value.length} 个文件，已完成 ${done} 个`;
});

const resumeSummary = computed(() => {
  if (!resumes.value.length) return "暂无续传任务";
  const highest = resumes.value.reduce(
    (best, item) => (item.progress > best.progress ? item : best),
    resumes.value[0],
  );
  return `${resumes.value.length} 项任务，最高进度 ${highest.progress}%（${highest.name}）`;
});

const scanSummary = computed(() => {
  if (!scanSession.value) return "暂无扫码上传会话";
  return `${scanSession.value.sessionId}｜${scanSession.value.status}`;
});

const tokenSummary = computed(() => {
  if (!metrics.apiTokens) return "暂无 Token 台账";
  return `${metrics.apiTokens} 条令牌记录`;
});

const coverageRows = computed(() => [
  {
    name: "用户与权限",
    count: metrics.users + metrics.authorities,
    detail: `${metrics.users} 用户，${metrics.authorities} 角色`,
    source: "/user/getUserList + /authority/getAuthorityList",
  },
  {
    name: "菜单与接口",
    count: metrics.menus + metrics.apis,
    detail: `${metrics.menus} 菜单，${metrics.apis} API`,
    source: "/menu/getMenuList + /api/getApiList",
  },
  {
    name: "字典与配置",
    count: metrics.dictionaries + (config.value ? 1 : 0),
    detail: `${metrics.dictionaries} 字典，系统配置 ${config.value ? "已接入" : "未接入"}`,
    source: "/sysDictionary/getSysDictionaryList + /system/getSystemConfig",
  },
  {
    name: "工具与代码生成",
    count: metrics.packages + metrics.autoCode + metrics.mcpTools,
    detail: `${metrics.packages} 包定义，${metrics.autoCode} 自动代码，${metrics.mcpTools} MCP 工具`,
    source: "/tool/package/list + /tool/auto-code/list + /autoCode/mcpList",
  },
  {
    name: "技能与插件",
    count: metrics.skills + metrics.pluginInstalls,
    detail: `${metrics.skills} 技能，${metrics.pluginInstalls} 插件安装记录`,
    source: "/skills/getSkillList + /tool/plugin-install/list",
  },
  {
    name: "交付与示例链路",
    count: releases.value.length + uploads.value.length + resumes.value.length + (scanSession.value ? 1 : 0),
    detail: `${releases.value.length} 发布，${uploads.value.length} 上传队列，${resumes.value.length} 续传，${scanSession.value ? "1 扫码会话" : "0 扫码会话"}`,
    source: "/tool/release/list + /example/*",
  },
]);

const deliveryNotes = computed(() => {
  const notes = [
    "关于页已切换为真实数据概览",
    "不包含原项目版权文案",
    "以当前 Rust / Vben 实现为准",
    "后续路线保持 Rust + Tauri 2",
  ];
  if (runtime.value?.redisEnabled) notes.push("Redis 链路已接入运行态");
  if (mcpRunning.value) notes.push("MCP 服务已进入可用状态");
  if (releases.value.length) notes.push(`已有 ${releases.value.length} 条发布记录`);
  if (metrics.skills) notes.push(`技能中心已登记 ${metrics.skills} 项`);
  return notes;
});

onMounted(() => {
  window.addEventListener("online", reloadOverview);
  window.addEventListener("offline", reloadOverview);
  reloadOverview();
});
</script>

<style scoped>
.about-layout {
  align-items: start;
}

.compact {
  gap: 12px;
}

.success {
  background: #ecfdf3;
  color: #027a48;
}

.warn {
  background: #fffaeb;
  color: #b54708;
}

.danger {
  background: #fef3f2;
  color: #b42318;
}
</style>
