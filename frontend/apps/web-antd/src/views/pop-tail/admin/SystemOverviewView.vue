<script setup lang="ts">
import type {
  WorkbenchProjectItem,
  WorkbenchQuickNavItem,
  WorkbenchTodoItem,
  WorkbenchTrendItem,
} from '@vben/common-ui';

import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import {
  AnalysisChartCard,
  Page,
  WorkbenchProject,
  WorkbenchQuickNav,
  WorkbenchTodo,
  WorkbenchTrends,
} from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { openWindow } from '@vben/utils';

import {
  Col,
  Row,
  Space,
  Statistic,
  Tag,
} from 'ant-design-vue';

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
} from '#/api/pop-tail/admin';
import type { McpServiceStatus, RuntimeInfo, SystemConfigInfo } from '#/types/pop-tail';

type HealthTone = 'error' | 'success' | 'warning';

interface OverviewEntry {
  badge: string;
  countLabel: string;
  detail: string;
  icon: string;
  key: string;
  level: HealthTone;
  path: string;
  signal: string;
  summary: string;
  title: string;
}

const router = useRouter();
const loading = ref(false);
const loadError = ref('');
const lastRefresh = ref('-');
const autoRefresh = ref(true);
let refreshTimer: ReturnType<typeof setInterval> | null = null;

const counts = reactive({
  apiTokens: 0,
  apis: 0,
  autoCode: 0,
  dictionaries: 0,
  errors: 0,
  logins: 0,
  loginFailures: 0,
  mcpTools: 0,
  menus: 0,
  operations: 0,
  params: 0,
  plugins: 0,
  releases: 0,
  roles: 0,
  skills: 0,
  users: 0,
});

const flags = reactive({
  dictionariesEnabled: 0,
  hiddenMenus: 0,
  usersEnabled: 0,
});

const latest = reactive({
  autoCode: '暂无蓝图记录',
  error: '暂无错误记录',
  login: '暂无登录记录',
  operation: '暂无操作记录',
  release: '暂无发布记录',
});

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcp = ref<McpServiceStatus | null>(null);

function formatTime(ts = Date.now()) {
  return new Intl.DateTimeFormat('zh-CN', {
    day: '2-digit',
    hour: '2-digit',
    hour12: false,
    minute: '2-digit',
    month: '2-digit',
    second: '2-digit',
  }).format(ts);
}

function healthLevel(ok: boolean, warning = false): HealthTone {
  if (ok) return 'success';
  return warning ? 'warning' : 'error';
}

const runtimeSummary = computed(() => {
  if (!runtime.value) {
    return '未获取运行时';
  }
  const redis = runtime.value.redisEnabled ? 'Redis 已接入' : 'Redis 未启用';
  return `${runtime.value.os} / ${runtime.value.dbBackend} / ${redis}`;
});

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
    attentionCount,
    healthLabel:
      healthScore >= 85
        ? '管理域状态稳定'
        : healthScore >= 65
          ? '管理域需巡检'
          : '管理域存在明显风险',
    healthScore,
    toolAssets,
  };
});

const managementEntries = computed<OverviewEntry[]>(() => [
  {
    badge: flags.usersEnabled === counts.users ? '全部启用' : `${flags.usersEnabled} 启用`,
    countLabel: `${counts.users} 个账户`,
    detail: counts.users
      ? `启用 ${flags.usersEnabled} / 停用 ${Math.max(counts.users - flags.usersEnabled, 0)}`
      : '暂无用户数据',
    icon: 'mdi:account-group-outline',
    key: 'users',
    level: healthLevel(counts.users > 0 && flags.usersEnabled > 0, counts.users > 0),
    path: '/system/users',
    signal: latest.login,
    summary: '账号启停、联系方式、角色挂载与登录入口治理',
    title: '用户管理',
  },
  {
    badge: `${counts.roles} 角色`,
    countLabel: `${counts.roles} 个角色定义`,
    detail: counts.roles > 0 ? '角色层级已接后端，可直接维护' : '角色定义为空',
    icon: 'mdi:shield-account-outline',
    key: 'roles',
    level: healthLevel(counts.roles > 0, true),
    path: '/system/authorities',
    signal: counts.roles > 0 ? '建议检查默认路由与菜单映射' : '建议补角色结构',
    summary: '默认路由、继承关系与权限包切换入口',
    title: '角色管理',
  },
  {
    badge: flags.hiddenMenus > 0 ? `${flags.hiddenMenus} 隐藏` : '导航稳定',
    countLabel: `${counts.menus} 个菜单节点`,
    detail: counts.menus > 0 ? `隐藏 ${flags.hiddenMenus} 个节点` : '暂无菜单定义',
    icon: 'mdi:view-grid-outline',
    key: 'menus',
    level: healthLevel(counts.menus > 0, true),
    path: '/system/menus',
    signal: counts.menus > 0 ? '可继续核对默认首页链路' : '需要补导航骨架',
    summary: '导航结构、组件挂载、排序与显示策略',
    title: '菜单管理',
  },
  {
    badge: `${counts.apis} 接口`,
    countLabel: `${counts.apis} 条接口台账`,
    detail: counts.apis > 0 ? latest.operation : '暂无接口台账',
    icon: 'mdi:api',
    key: 'apis',
    level: healthLevel(counts.apis > 0, true),
    path: '/system/apis',
    signal: counts.errors > 0 ? latest.error : '近期未发现错误信号',
    summary: '接口路径、分组、方法与说明统一维护',
    title: '接口管理',
  },
  {
    badge: `${counts.dictionaries} 字典`,
    countLabel: `${counts.dictionaries} 个字典主表`,
    detail: `启用 ${flags.dictionariesEnabled} / 停用 ${Math.max(counts.dictionaries - flags.dictionariesEnabled, 0)}`,
    icon: 'mdi:book-alphabet-outline',
    key: 'dictionary',
    level: healthLevel(counts.dictionaries > 0, true),
    path: '/system/dictionaries',
    signal: counts.dictionaries > 0 ? '适合先核对业务枚举完整性' : '需要补业务字典',
    summary: '字典主表与树形详情统一治理',
    title: '字典管理',
  },
  {
    badge: `${counts.params} 参数`,
    countLabel: `${counts.params} 条系统参数`,
    detail: config.value
      ? `${config.value.multipointEnabled ? '多点登录开启' : '单点会话模式'}`
      : '尚未加载配置',
    icon: 'mdi:tune-variant',
    key: 'params',
    level: healthLevel(counts.params > 0, true),
    path: '/system/params',
    signal: runtimeSummary.value,
    summary: '系统开关、默认值与行为参数维护入口',
    title: '参数配置',
  },
  {
    badge: summary.value.attentionCount > 0 ? `${summary.value.attentionCount} 待关注` : '稳定',
    countLabel: `${counts.operations + counts.logins + counts.errors} 条近端信号`,
    detail: `${counts.operations} 操作 / ${counts.logins} 登录 / ${counts.errors} 错误`,
    icon: 'mdi:monitor-dashboard',
    key: 'logs',
    level: healthLevel(summary.value.attentionCount === 0, summary.value.attentionCount < 3),
    path: '/system/state',
    signal: mcp.value?.message || '未接入',
    summary: '系统态、操作日志、登录日志、错误日志联动巡检',
    title: '日志与系统态',
  },
]);

const suggestionRows = computed(() => {
  const result: Array<{ detail: string; path: string; title: string }> = [];
  if (counts.errors > 0 || counts.loginFailures > 0) {
    result.push({
      detail: `当前有 ${counts.errors} 条错误和 ${counts.loginFailures} 次失败登录，建议先看系统状态页。`,
      path: '/system/state',
      title: '优先巡检系统状态',
    });
  }
  if (!mcp.value?.reachable || counts.mcpTools === 0) {
    result.push({
      detail: 'MCP 服务未完全就绪或工具定义不足，建议先检查系统工具页。',
      path: '/system/tools',
      title: '恢复工具链可达性',
    });
  }
  result.push(
    {
      detail: `用户 ${counts.users}、角色 ${counts.roles}，建议抽查管理员与默认路由配置。`,
      path: '/system/users',
      title: '核对账号与权限包',
    },
    {
      detail: `菜单 ${counts.menus}、接口 ${counts.apis}，避免页面入口和后端接口脱节。`,
      path: '/system/menus',
      title: '核对导航与接口映射',
    },
    {
      detail: `字典 ${counts.dictionaries}、参数 ${counts.params}，确保后台表单和业务文案一致。`,
      path: '/system/dictionaries',
      title: '检查业务枚举与参数',
    },
  );
  return result.slice(0, 4);
});

const projectItems = computed<WorkbenchProjectItem[]>(() =>
  managementEntries.value.map((item, index) => ({
    color: [
      'var(--ant-color-primary)',
      'var(--ant-color-success)',
      'var(--ant-color-warning)',
      'var(--ant-color-info)',
      '#8b5cf6',
      'var(--ant-color-error)',
      'var(--ant-color-primary)',
    ][index % 7]!,
    content: item.summary,
    date: item.detail,
    group: item.badge,
    icon: item.icon,
    title: item.title,
    url: item.path,
  })),
);

const trendItems = computed<WorkbenchTrendItem[]>(() => [
  {
    avatar: 'svg:avatar-1',
    content: `最近登录 <a>${latest.login}</a>`,
    date: lastRefresh.value,
    title: '登录信号',
  },
  {
    avatar: 'svg:avatar-2',
    content: `最近操作 <a>${latest.operation}</a>`,
    date: lastRefresh.value,
    title: '操作信号',
  },
  {
    avatar: 'svg:avatar-3',
    content: `最近错误 <a>${latest.error}</a>`,
    date: lastRefresh.value,
    title: '错误信号',
  },
  {
    avatar: 'svg:avatar-4',
    content: `最近发布 <a>${latest.release}</a>`,
    date: lastRefresh.value,
    title: '发布信号',
  },
]);

const quickNavItems = computed<WorkbenchQuickNavItem[]>(() =>
  managementEntries.value.slice(0, 6).map((item, index) => ({
    color: [
      'var(--ant-color-primary)',
      'var(--ant-color-success)',
      'var(--ant-color-warning)',
      'var(--ant-color-info)',
      '#8b5cf6',
      'var(--ant-color-error)',
    ][index % 6]!,
    icon: item.icon,
    title: item.title,
    url: item.path,
  })),
);

const todoItems = computed<WorkbenchTodoItem[]>(() =>
  suggestionRows.value.map((item, index) => ({
    completed: index === 0 ? summary.value.attentionCount === 0 : false,
    content: item.detail,
    date: lastRefresh.value,
    title: item.title,
  })),
);

function navigate(path: string) {
  router.push(path).catch(() => undefined);
}

function navTo(item: WorkbenchProjectItem | WorkbenchQuickNavItem) {
  if (!item.url) return;
  if (item.url.startsWith('http')) {
    openWindow(item.url);
    return;
  }
  navigate(item.url);
}

async function reloadOverview() {
  loading.value = true;
  loadError.value = '';
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
      : '暂无操作记录';
    latest.login = loginLogs.List[0]
      ? `${loginLogs.List[0].username} @ ${loginLogs.List[0].ip}（${loginLogs.List[0].status ? '成功' : `失败：${loginLogs.List[0].errorMessage || '未知原因'}`}）`
      : '暂无登录记录';
    latest.error = errorLogs.List[0]
      ? `${errorLogs.List[0].path}（${errorLogs.List[0].status}）`
      : '暂无错误记录';
    latest.release = releases.List[0]
      ? `${formatTime(releases.List[0].createdAt)}：${releases.List[0].note}`
      : '暂无发布记录';
    latest.autoCode = autoCodes.List[0]
      ? `${String(autoCodes.List[0].payload.entity || autoCodes.List[0].payload.table || '未命名蓝图')} / ${String(autoCodes.List[0].payload.module || '未分组')}`
      : '暂无蓝图记录';

    runtime.value = runtimeInfo;
    config.value = systemConfig;
    mcp.value = mcpStatus;
    lastRefresh.value = formatTime();
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : '管理总览加载失败';
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
  }, 15_000);
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

<template>
  <Page>
    <div>


      <Row :gutter="[16, 16]" class="my-4">
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <div class="card-box stats-card">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-primary)]" icon="mdi:view-dashboard-outline" />
                <span>管理台账</span>
              </Space>
              <Tag color="processing">Admin</Tag>
            </div>
            <Statistic :value="summary.adminAssets" />
            <div class="metric-card-footer">用户 / 角色 / 菜单 / 接口 / 字典 / 参数</div>
          </div>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <div class="card-box stats-card">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-success)]" icon="mdi:tools" />
                <span>工具台账</span>
              </Space>
              <Tag color="success">Tools</Tag>
            </div>
            <Statistic :value="summary.toolAssets" />
            <div class="metric-card-footer">令牌 / 插件 / 自动代码 / 技能 / MCP / 发布</div>
          </div>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <div class="card-box stats-card">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-warning)]" icon="mdi:heart-pulse" />
                <span>健康信号</span>
              </Space>
              <Tag :color="summary.healthScore >= 85 ? 'success' : summary.healthScore >= 65 ? 'warning' : 'error'">
                {{ summary.healthLabel }}
              </Tag>
            </div>
            <Statistic :suffix="'%'" :value="summary.healthScore" />
            <div class="metric-card-footer">{{ summary.healthLabel }}</div>
          </div>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <div class="card-box stats-card">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-error)]" icon="mdi:bell-alert-outline" />
                <span>待关注项</span>
              </Space>
              <Tag :color="summary.attentionCount > 0 ? 'error' : 'success'">巡检</Tag>
            </div>
            <Statistic :value="summary.attentionCount" />
            <div class="metric-card-footer">错误、失败登录、配置风险与服务不可达</div>
          </div>
        </Col>
      </Row>

      <div class="mt-5 flex flex-col lg:flex-row">
        <div class="mr-4 w-full lg:w-3/5">
          <WorkbenchProject :items="projectItems" title="管理域入口" @click="navTo" />
          <WorkbenchTrends :items="trendItems" class="mt-5" title="最新动态" />
        </div>
        <div class="w-full lg:w-2/5">
          <WorkbenchQuickNav
            :items="quickNavItems"
            class="mt-5 lg:mt-0"
            title="快捷导航"
            @click="navTo"
          />
          <WorkbenchTodo :items="todoItems" class="mt-5" title="待办事项" />
          <AnalysisChartCard class="mt-5" title="管理信号">
            <div class="signal-overview">
              <div class="signal-row">
                <span class="signal-label">运行底座</span>
                <strong>{{ runtimeSummary }}</strong>
              </div>
              <div class="signal-row">
                <span class="signal-label">待关注项</span>
                <strong>{{ summary.attentionCount }}</strong>
              </div>
              <div class="signal-row">
                <span class="signal-label">健康评分</span>
                <strong>{{ summary.healthScore }}%</strong>
              </div>
              <div class="signal-row">
                <span class="signal-label">最近刷新</span>
                <strong>{{ lastRefresh }}</strong>
              </div>
            </div>
          </AnalysisChartCard>
        </div>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.page-stack {
  display: grid;
  gap: 20px;
  padding: 20px;
}

.stats-card {
  display: flex;
  flex-direction: column;

  padding: 10px;
}

.signal-list {
  display: grid;
  gap: 16px;
}

.metric-card-header,
.entry-header,
.entry-footer {
  align-items: center;
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.metric-icon {
  font-size: 18px;
}

.metric-card-footer,
.entry-detail {
  color: var(--ant-color-text-secondary);
  font-size: 13px;
  line-height: 1.65;
}

.stats-card :deep(.ant-statistic) {
  margin-top: 18px;
}

.stats-card :deep(.ant-statistic .ant-statistic-content) {
  color: var(--ant-color-text);
  font-size: 36px;
  font-weight: 600;
  line-height: 1;
}

.stats-card :deep(.ant-statistic .ant-statistic-content-suffix) {
  font-size: 28px;
}

.entry-card,
.entry-stat {
  height: 100%;
}

.page-stack :deep(.ant-alert) {
  border-radius: 12px;
}

.signal-overview {
  display: grid;
  gap: 12px;
}

.signal-row {
  align-items: center;
  border-bottom: 1px solid var(--ant-color-border-secondary);
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 0;
}

.signal-row:last-child {
  border-bottom: none;
}

.signal-label {
  color: var(--ant-color-text-secondary);
  font-size: 13px;
}
</style>
