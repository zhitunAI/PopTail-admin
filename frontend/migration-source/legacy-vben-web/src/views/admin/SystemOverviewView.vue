<template>
  <div class="stack overview-page">
    <div class="card hero-card">
      <div class="row space-between wrap gap-12">
        <div>
          <h3 class="title">管理总览</h3>
          <p class="subtitle">
            聚合管理域真实台账、系统治理信号与可进入入口，作为管理员日常巡检与分发工作台使用。
          </p>
        </div>
        <div class="row wrap gap-8">
          <label class="inline-check">
            <input v-model="autoRefresh" type="checkbox" />
            自动刷新
          </label>
          <button class="btn ghost" :disabled="loading" @click="reloadOverview">
            {{ loading ? "刷新中..." : "立即刷新" }}
          </button>
        </div>
      </div>
      <div class="hero-meta row wrap gap-12">
        <span>最近刷新：{{ lastRefresh }}</span>
        <span>治理覆盖：{{ coverageLabel }}</span>
        <span>运行底座：{{ runtimeSummary }}</span>
        <span v-if="loadError">最近错误：{{ loadError }}</span>
      </div>
    </div>

    <div class="muted-grid summary-grid">
      <div class="stat-card">
        <h4>管理台账</h4>
        <p class="stat-value">{{ summary.adminAssets }}</p>
        <p class="subtitle">用户 / 角色 / 菜单 / 接口 / 字典 / 参数</p>
      </div>
      <div class="stat-card">
        <h4>工具台账</h4>
        <p class="stat-value">{{ summary.toolAssets }}</p>
        <p class="subtitle">令牌 / 插件 / 技能 / MCP / 发布 / 自动代码</p>
      </div>
      <div class="stat-card">
        <h4>健康信号</h4>
        <p class="stat-value">{{ summary.healthScore }}%</p>
        <p class="subtitle">{{ summary.healthLabel }}</p>
      </div>
      <div class="stat-card">
        <h4>待关注项</h4>
        <p class="stat-value">{{ summary.attentionCount }}</p>
        <p class="subtitle">错误、失败登录、配置风险与服务不可达</p>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap gap-12">
        <div>
          <h3 class="title">管理域入口</h3>
          <p class="subtitle">每个入口都由真实数据回填状态，不再只是静态菜单说明。</p>
        </div>
        <div class="tag-list">
          <span class="tag" :class="summary.healthScore >= 80 ? 'good' : summary.healthScore >= 60 ? 'warn' : 'bad'">
            {{ summary.healthLabel }}
          </span>
          <span class="tag">在线用户数据：{{ usersEnabled }}/{{ counts.users }}</span>
          <span class="tag">MCP：{{ mcpStatusLabel }}</span>
        </div>
      </div>

      <div class="entry-grid">
        <RouterLink
          v-for="item in managementEntries"
          :key="item.key"
          class="entry-card"
          :to="item.path"
        >
          <div class="row space-between gap-12 wrap">
            <div>
              <h4>{{ item.title }}</h4>
              <p class="entry-summary">{{ item.summary }}</p>
            </div>
            <span class="tag" :class="item.level">{{ item.badge }}</span>
          </div>
          <p class="entry-count">{{ item.countLabel }}</p>
          <p class="subtitle">{{ item.detail }}</p>
          <div class="entry-footer row space-between wrap gap-8">
            <span>{{ item.signal }}</span>
            <span>进入管理</span>
          </div>
        </RouterLink>
      </div>
    </div>

    <div class="split-grid overview-layout">
      <div class="card">
        <h3 class="title">治理摘要</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>域</th>
                <th>数量</th>
                <th>摘要</th>
                <th>建议动作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in ledgerRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.count }}</td>
                <td>{{ item.summary }}</td>
                <td>{{ item.next }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">关键趋势 / 信号</h3>
        <div class="signal-list">
          <div v-for="item in signalRows" :key="item.title" class="signal-item">
            <div class="row space-between wrap gap-12">
              <strong>{{ item.title }}</strong>
              <span class="tag" :class="item.level">{{ item.levelLabel }}</span>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="split-grid overview-layout">
      <div class="card">
        <h3 class="title">近期管理活动</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>类别</th>
                <th>摘要</th>
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

      <div class="card">
        <h3 class="title">推荐巡检顺序</h3>
        <div class="signal-list">
          <div v-for="item in suggestionRows" :key="item.title" class="signal-item compact-item">
            <div class="row space-between wrap gap-12">
              <strong>{{ item.title }}</strong>
              <RouterLink class="btn ghost" :to="item.path">打开</RouterLink>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { RouterLink } from "vue-router";
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
  getParamsListApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getRuntimeInfoApi,
  getSkillListApi,
  getSystemConfigApi,
  getUserListApi,
} from "../../api/admin";
import type {
  McpServiceStatus,
  RuntimeInfo,
  SystemConfigInfo,
} from "../../types";

const loading = ref(false);
const loadError = ref("");
const lastRefresh = ref("-");
const autoRefresh = ref(true);
let refreshTimer: ReturnType<typeof setInterval> | null = null;

const counts = reactive({
  users: 0,
  roles: 0,
  menus: 0,
  apis: 0,
  dictionaries: 0,
  params: 0,
  apiTokens: 0,
  plugins: 0,
  autoCode: 0,
  skills: 0,
  mcpTools: 0,
  releases: 0,
  operations: 0,
  logins: 0,
  loginFailures: 0,
  errors: 0,
});

const flags = reactive({
  usersEnabled: 0,
  hiddenMenus: 0,
  dictionariesEnabled: 0,
});

const latest = reactive({
  operation: "暂无操作记录",
  login: "暂无登录记录",
  error: "暂无错误记录",
  release: "暂无发布记录",
  autoCode: "暂无蓝图记录",
});

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcp = ref<McpServiceStatus | null>(null);

function formatTime(ts = Date.now()) {
  return new Intl.DateTimeFormat("zh-CN", {
    hour12: false,
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(ts);
}

function healthLevel(ok: boolean, warning = false) {
  if (ok) return "good";
  return warning ? "warn" : "bad";
}

const runtimeSummary = computed(() => {
  if (!runtime.value) {
    return "未获取运行时";
  }
  const redis = runtime.value.redisEnabled ? "Redis 已接入" : "Redis 未启用";
  return `${runtime.value.os} / ${runtime.value.dbBackend} / ${redis}`;
});

const coverageLabel = computed(() => {
  const covered = [
    counts.users,
    counts.roles,
    counts.menus,
    counts.apis,
    counts.dictionaries,
    counts.params,
    counts.apiTokens,
    counts.plugins,
    counts.autoCode,
    counts.skills,
    counts.mcpTools,
    counts.releases,
  ].filter((item) => item >= 0).length;
  return `${covered}/12 个核心域已回填`;
});

const usersEnabled = computed(() => flags.usersEnabled);
const mcpStatusLabel = computed(() => mcp.value?.message || "未接入");

const summary = computed(() => {
  const adminAssets =
    counts.users +
    counts.roles +
    counts.menus +
    counts.apis +
    counts.dictionaries +
    counts.params;
  const toolAssets =
    counts.apiTokens +
    counts.plugins +
    counts.autoCode +
    counts.skills +
    counts.mcpTools +
    counts.releases;
  const penalties = [
    counts.errors > 0 ? 16 : 0,
    counts.loginFailures > 0 ? 12 : 0,
    mcp.value && !mcp.value.reachable ? 10 : 0,
    config.value && !config.value.redisUrl ? 8 : 0,
    config.value && !config.value.databaseUrl ? 12 : 0,
    counts.users > 0 && flags.usersEnabled === 0 ? 18 : 0,
  ].reduce((total, current) => total + current, 0);
  const healthScore = Math.max(25, 100 - penalties);
  const attentionCount = [
    counts.errors > 0,
    counts.loginFailures > 0,
    Boolean(mcp.value && !mcp.value.reachable),
    Boolean(config.value && !config.value.redisUrl),
    Boolean(config.value && !config.value.databaseUrl),
    counts.users > 0 && flags.usersEnabled === 0,
  ].filter(Boolean).length;

  return {
    adminAssets,
    toolAssets,
    healthScore,
    attentionCount,
    healthLabel:
      healthScore >= 85 ? "管理域状态稳定" : healthScore >= 65 ? "管理域需巡检" : "管理域存在明显风险",
  };
});

const managementEntries = computed(() => [
  {
    key: "users",
    title: "用户管理",
    path: "/system/users",
    badge: flags.usersEnabled === counts.users ? "全部启用" : `${flags.usersEnabled} 启用`,
    level: healthLevel(counts.users > 0 && flags.usersEnabled > 0, counts.users > 0),
    countLabel: `${counts.users} 个账户`,
    summary: "账号启停、联系方式、角色挂载与登录入口治理",
    detail: counts.users
      ? `启用 ${flags.usersEnabled} / 停用 ${Math.max(counts.users - flags.usersEnabled, 0)}`
      : "暂无用户数据",
    signal: latest.login,
  },
  {
    key: "roles",
    title: "角色管理",
    path: "/system/authorities",
    badge: `${counts.roles} 角色`,
    level: healthLevel(counts.roles > 0, true),
    countLabel: `${counts.roles} 个角色定义`,
    summary: "默认路由、继承关系与权限包切换入口",
    detail: counts.roles > 0 ? "角色层级已接后端，可直接维护" : "角色定义为空",
    signal: counts.roles > 0 ? "建议检查默认路由与菜单映射" : "建议补角色结构",
  },
  {
    key: "menus",
    title: "菜单管理",
    path: "/system/menus",
    badge: flags.hiddenMenus > 0 ? `${flags.hiddenMenus} 隐藏` : "导航稳定",
    level: healthLevel(counts.menus > 0, true),
    countLabel: `${counts.menus} 个菜单节点`,
    summary: "导航结构、组件挂载、排序与显示策略",
    detail: counts.menus > 0 ? `隐藏 ${flags.hiddenMenus} 个节点` : "暂无菜单定义",
    signal: counts.menus > 0 ? "可继续核对默认首页链路" : "需要补导航骨架",
  },
  {
    key: "apis",
    title: "接口管理",
    path: "/system/apis",
    badge: `${counts.apis} 接口`,
    level: healthLevel(counts.apis > 0, true),
    countLabel: `${counts.apis} 条接口台账`,
    summary: "接口路径、分组、方法与说明统一维护",
    detail: counts.apis > 0 ? latest.operation : "暂无接口台账",
    signal: counts.errors > 0 ? latest.error : "近期未发现错误信号",
  },
  {
    key: "dictionary",
    title: "字典管理",
    path: "/system/dictionaries",
    badge: `${counts.dictionaries} 字典`,
    level: healthLevel(counts.dictionaries > 0, true),
    countLabel: `${counts.dictionaries} 个字典主表`,
    summary: "字典主表与树形详情统一治理",
    detail: `启用 ${flags.dictionariesEnabled} / 停用 ${Math.max(counts.dictionaries - flags.dictionariesEnabled, 0)}`,
    signal: counts.dictionaries > 0 ? "适合先核对业务枚举完整性" : "需要补业务字典",
  },
  {
    key: "params",
    title: "参数配置",
    path: "/system/params",
    badge: `${counts.params} 参数`,
    level: healthLevel(counts.params > 0, true),
    countLabel: `${counts.params} 条系统参数`,
    summary: "系统开关、默认值与行为参数维护入口",
    detail: config.value
      ? `${config.value.multipointEnabled ? "多点登录开启" : "单点会话模式"}`
      : "尚未加载配置",
    signal: runtimeSummary.value,
  },
  {
    key: "logs",
    title: "日志与系统态",
    path: "/system/state",
    badge: summary.value.attentionCount > 0 ? `${summary.value.attentionCount} 待关注` : "稳定",
    level: healthLevel(summary.value.attentionCount === 0, summary.value.attentionCount < 3),
    countLabel: `${counts.operations + counts.logins + counts.errors} 条近端信号`,
    summary: "系统态、操作日志、登录日志、错误日志联动巡检",
    detail: `${counts.operations} 操作 / ${counts.logins} 登录 / ${counts.errors} 错误`,
    signal: mcpStatusLabel.value,
  },
]);

const ledgerRows = computed(() => [
  {
    name: "账号与身份",
    count: `${counts.users} / ${counts.roles}`,
    summary: `用户 ${counts.users}，角色 ${counts.roles}，启用账号 ${flags.usersEnabled}`,
    next: counts.loginFailures > 0 ? "优先排查失败登录来源" : "核对默认角色与首页跳转",
  },
  {
    name: "导航与接口",
    count: `${counts.menus} / ${counts.apis}`,
    summary: `菜单 ${counts.menus}，接口 ${counts.apis}，隐藏节点 ${flags.hiddenMenus}`,
    next: counts.errors > 0 ? "结合错误日志检查菜单到接口链路" : "抽查高频入口是否齐备",
  },
  {
    name: "业务字典",
    count: `${counts.dictionaries} / ${counts.params}`,
    summary: `字典 ${counts.dictionaries}，参数 ${counts.params}`,
    next: counts.dictionaries > 0 ? "校对关键枚举和系统参数默认值" : "优先恢复业务枚举",
  },
  {
    name: "工具域",
    count: `${counts.skills} / ${counts.mcpTools}`,
    summary: `技能 ${counts.skills}，MCP 工具 ${counts.mcpTools}，插件 ${counts.plugins}`,
    next: mcp.value?.reachable ? "可继续联调生成链路" : "先恢复 MCP 服务可达性",
  },
  {
    name: "交付域",
    count: `${counts.autoCode} / ${counts.releases}`,
    summary: `自动代码 ${counts.autoCode}，发布记录 ${counts.releases}`,
    next: counts.autoCode > 0 ? "抽查最新蓝图与发布注记是否匹配" : "继续沉淀生成资产",
  },
]);

const signalRows = computed(() => [
  {
    title: "运行时底座",
    level: healthLevel(Boolean(runtime.value?.dbBackend), !runtime.value?.redisEnabled),
    levelLabel: runtime.value?.redisEnabled ? "已就绪" : "需补强",
    detail: runtime.value
      ? `服务运行在 ${runtime.value.os}，数据库后端 ${runtime.value.dbBackend}，Rust ${runtime.value.rustVersion}，CPU ${runtime.value.cpuCores} 核。`
      : "尚未获取运行时信息。",
  },
  {
    title: "会话与配置",
    level: healthLevel(Boolean(config.value?.databaseUrl), !config.value?.redisUrl),
    levelLabel: config.value?.multipointEnabled ? "多点登录" : "单点会话",
    detail: config.value
      ? `监听 ${config.value.bindAddress}，数据库 ${config.value.databaseUrl || "未配置"}，Redis ${config.value.redisUrl || "未配置"}。`
      : "尚未获取系统配置。",
  },
  {
    title: "MCP 服务",
    level: healthLevel(Boolean(mcp.value?.reachable), Boolean(mcp.value?.managed)),
    levelLabel: mcp.value?.state || "未知",
    detail: mcp.value
      ? `${mcp.value.message}，Base URL：${mcp.value.baseURL || "-"}`
      : "尚未获取 MCP 状态。",
  },
  {
    title: "访问/错误趋势",
    level: healthLevel(counts.errors === 0 && counts.loginFailures === 0, counts.errors === 0),
    levelLabel: counts.errors > 0 ? "有错误" : counts.loginFailures > 0 ? "有失败登录" : "稳定",
    detail: `最近共 ${counts.operations} 条操作日志、${counts.logins} 条登录日志、${counts.errors} 条错误日志，失败登录 ${counts.loginFailures} 次。`,
  },
]);

const activityRows = computed(() => [
  { name: "最近操作", detail: latest.operation },
  { name: "最近登录", detail: latest.login },
  { name: "最近错误", detail: latest.error },
  { name: "最近发布", detail: latest.release },
  { name: "最近蓝图", detail: latest.autoCode },
  {
    name: "工具台账",
    detail: `令牌 ${counts.apiTokens} / 插件 ${counts.plugins} / 技能 ${counts.skills} / MCP 工具 ${counts.mcpTools}`,
  },
]);

const suggestionRows = computed(() => {
  const rows = [] as Array<{ title: string; detail: string; path: string }>;
  if (counts.errors > 0 || counts.loginFailures > 0) {
    rows.push({
      title: "优先巡检系统状态",
      detail: `当前有 ${counts.errors} 条错误和 ${counts.loginFailures} 次失败登录，建议先看系统状态页。`,
      path: "/system/state",
    });
  }
  if (!mcp.value?.reachable || counts.mcpTools === 0) {
    rows.push({
      title: "恢复工具链可达性",
      detail: "MCP 服务未完全就绪或工具定义不足，建议先检查系统工具页。",
      path: "/system/tools",
    });
  }
  rows.push(
    {
      title: "核对账号与权限包",
      detail: `用户 ${counts.users}、角色 ${counts.roles}，建议抽查管理员与默认路由配置。`,
      path: "/system/users",
    },
    {
      title: "核对导航与接口映射",
      detail: `菜单 ${counts.menus}、接口 ${counts.apis}，避免页面入口和后端接口脱节。`,
      path: "/system/menus",
    },
    {
      title: "检查业务枚举与参数",
      detail: `字典 ${counts.dictionaries}、参数 ${counts.params}，确保后台表单和业务文案一致。`,
      path: "/system/dictionaries",
    },
  );
  return rows.slice(0, 4);
});

async function reloadOverview() {
  loading.value = true;
  loadError.value = "";
  try {
    const [
      users,
      authorities,
      menus,
      apis,
      dictionaries,
      params,
      runtimeInfo,
      systemConfig,
      operationLogs,
      loginLogs,
      errorLogs,
      apiTokens,
      pluginInstalls,
      autoCodes,
      skills,
      mcpTools,
      releases,
      mcpStatus,
    ] = await Promise.all([
      getUserListApi(),
      getAuthorityListApi(),
      getMenuListApi(),
      getApiListApi(),
      getDictionaryListApi(),
      getParamsListApi(),
      getRuntimeInfoApi(),
      getSystemConfigApi(),
      getOperationLogsApi(),
      getLoginLogsApi(),
      getErrorLogsApi(),
      getApiTokenListApi(),
      getPluginInstallListApi(),
      getAutoCodeRegistryApi(),
      getSkillListApi(),
      getMcpToolListApi(),
      getReleaseListApi(),
      getMcpStatusApi(),
    ]);

    counts.users = users.Total;
    counts.roles = authorities.length;
    counts.menus = menus.Total;
    counts.apis = apis.Total;
    counts.dictionaries = dictionaries.Total;
    counts.params = params.Total;
    counts.operations = operationLogs.Total;
    counts.logins = loginLogs.Total;
    counts.errors = errorLogs.Total;
    counts.apiTokens = apiTokens.Total;
    counts.plugins = pluginInstalls.Total;
    counts.autoCode = autoCodes.Total;
    counts.skills = skills.length;
    counts.mcpTools = mcpTools.length;
    counts.releases = releases.Total;
    counts.loginFailures = loginLogs.List.filter((item) => !item.status).length;

    flags.usersEnabled = users.List.filter((item) => item.enable === 1).length;
    flags.hiddenMenus = menus.List.filter((item: { hidden?: boolean }) => item.hidden).length;
    flags.dictionariesEnabled = dictionaries.List.filter((item) => item.status).length;

    latest.operation = operationLogs.List[0]
      ? `${operationLogs.List[0].method} ${operationLogs.List[0].path}（状态 ${operationLogs.List[0].status}）`
      : "暂无操作记录";
    latest.login = loginLogs.List[0]
      ? `${loginLogs.List[0].username} @ ${loginLogs.List[0].ip}（${loginLogs.List[0].status ? "成功" : `失败：${loginLogs.List[0].errorMessage || "未知原因"}`}）`
      : "暂无登录记录";
    latest.error = errorLogs.List[0]
      ? `${errorLogs.List[0].path}（${errorLogs.List[0].status}）`
      : "暂无错误记录";
    latest.release = releases.List[0]
      ? `${formatTime(releases.List[0].createdAt)}：${releases.List[0].note}`
      : "暂无发布记录";
    latest.autoCode = autoCodes.List[0]
      ? `${String(autoCodes.List[0].payload.entity || autoCodes.List[0].payload.table || "未命名蓝图")} / ${String(autoCodes.List[0].payload.module || "未分组")}`
      : "暂无蓝图记录";

    runtime.value = runtimeInfo;
    config.value = systemConfig;
    mcp.value = mcpStatus;
    lastRefresh.value = formatTime();
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : "管理总览加载失败";
  } finally {
    loading.value = false;
  }
}

function startRefreshTimer() {
  if (refreshTimer) {
    clearInterval(refreshTimer);
  }
  if (!autoRefresh.value) {
    refreshTimer = null;
    return;
  }
  refreshTimer = setInterval(() => {
    void reloadOverview();
  }, 15000);
}

watch(autoRefresh, () => {
  startRefreshTimer();
});

onMounted(async () => {
  await reloadOverview();
  startRefreshTimer();
});

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer);
  }
});
</script>

<style scoped>
.overview-page {
  gap: 16px;
}

.hero-card,
.entry-card,
.signal-item {
  border: 1px solid rgba(148, 163, 184, 0.18);
}

.hero-meta {
  margin-top: 12px;
  color: #64748b;
  font-size: 13px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.entry-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.entry-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px;
  border-radius: 14px;
  background: #fff;
  color: inherit;
  text-decoration: none;
  transition: transform 0.16s ease, box-shadow 0.16s ease;
}

.entry-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.06);
}

.entry-summary {
  margin: 6px 0 0;
  color: #334155;
}

.entry-count {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: #0f172a;
}

.entry-footer {
  margin-top: auto;
  color: #64748b;
  font-size: 13px;
}

.overview-layout {
  align-items: start;
}

.signal-list {
  display: grid;
  gap: 12px;
}

.signal-item {
  padding: 14px;
  border-radius: 12px;
  background: #f8fafc;
}

.signal-item p {
  margin: 8px 0 0;
  color: #475569;
  line-height: 1.6;
}

.compact-item {
  background: #fff;
}

.gap-8 {
  gap: 8px;
}

.gap-12 {
  gap: 12px;
}
</style>
