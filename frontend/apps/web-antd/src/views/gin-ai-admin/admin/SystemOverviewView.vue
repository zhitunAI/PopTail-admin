<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import {
  Alert,
  Button,
  Card,
  Col,
  Row,
  Space,
  Statistic,
  Switch,
  Table,
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
} from '#/api/gin-ai-admin/admin';
import type { McpServiceStatus, RuntimeInfo, SystemConfigInfo } from '#/types/gin-ai-admin';

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

const ledgerColumns = [
  { title: '域', dataIndex: 'name', key: 'name', width: 120 },
  { title: '数量', dataIndex: 'count', key: 'count', width: 120 },
  { title: '摘要', dataIndex: 'summary', key: 'summary', ellipsis: true },
  { title: '建议动作', dataIndex: 'next', key: 'next', ellipsis: true },
];

const activityColumns = [
  { title: '类别', dataIndex: 'name', key: 'name', width: 120 },
  { title: '摘要', dataIndex: 'detail', key: 'detail', ellipsis: true },
];

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

function toneColor(level: HealthTone) {
  if (level === 'success') return 'success';
  if (level === 'warning') return 'warning';
  return 'error';
}

const runtimeSummary = computed(() => {
  if (!runtime.value) {
    return '未获取运行时';
  }
  const redis = runtime.value.redisEnabled ? 'Redis 已接入' : 'Redis 未启用';
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
  ].filter((item) => item > 0).length;
  return `${covered}/12 个核心域已有真实台账`;
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

const ledgerRows = computed(() => [
  {
    count: `${counts.users} / ${counts.roles}`,
    name: '账号与身份',
    next: counts.loginFailures > 0 ? '优先排查失败登录来源' : '核对默认角色与首页跳转',
    summary: `用户 ${counts.users}，角色 ${counts.roles}，启用账号 ${flags.usersEnabled}`,
  },
  {
    count: `${counts.menus} / ${counts.apis}`,
    name: '导航与接口',
    next: counts.errors > 0 ? '结合错误日志检查菜单到接口链路' : '抽查高频入口是否齐备',
    summary: `菜单 ${counts.menus}，接口 ${counts.apis}，隐藏节点 ${flags.hiddenMenus}`,
  },
  {
    count: `${counts.dictionaries} / ${counts.params}`,
    name: '业务字典',
    next: counts.dictionaries > 0 ? '校对关键枚举和系统参数默认值' : '优先恢复业务枚举',
    summary: `字典 ${counts.dictionaries}，参数 ${counts.params}`,
  },
  {
    count: `${counts.skills} / ${counts.mcpTools}`,
    name: '工具域',
    next: mcp.value?.reachable ? '可继续联调生成链路' : '先恢复 MCP 服务可达性',
    summary: `技能 ${counts.skills}，MCP 工具 ${counts.mcpTools}，插件 ${counts.plugins}`,
  },
  {
    count: `${counts.autoCode} / ${counts.releases}`,
    name: '交付域',
    next: counts.autoCode > 0 ? '抽查最新蓝图与发布注记是否匹配' : '继续沉淀生成资产',
    summary: `自动代码 ${counts.autoCode}，发布记录 ${counts.releases}`,
  },
]);

const signalRows = computed(() => [
  {
    detail: runtime.value
      ? `服务运行在 ${runtime.value.os}，数据库后端 ${runtime.value.dbBackend}，Rust ${runtime.value.rustVersion}，CPU ${runtime.value.cpuCores} 核。`
      : '尚未获取运行时信息。',
    level: healthLevel(Boolean(runtime.value?.dbBackend), !runtime.value?.redisEnabled),
    levelLabel: runtime.value?.redisEnabled ? '已就绪' : '需补强',
    title: '运行时底座',
  },
  {
    detail: config.value
      ? `监听 ${config.value.bindAddress}，数据库 ${config.value.databaseUrl || '未配置'}，Redis ${config.value.redisUrl || '未配置'}。`
      : '尚未获取系统配置。',
    level: healthLevel(Boolean(config.value?.databaseUrl), !config.value?.redisUrl),
    levelLabel: config.value?.multipointEnabled ? '多点登录' : '单点会话',
    title: '会话与配置',
  },
  {
    detail: mcp.value
      ? `${mcp.value.message}，Base URL：${mcp.value.baseURL || '-'}`
      : '尚未获取 MCP 状态。',
    level: healthLevel(Boolean(mcp.value?.reachable), Boolean(mcp.value?.managed)),
    levelLabel: mcp.value?.state || '未知',
    title: 'MCP 服务',
  },
  {
    detail: `最近共 ${counts.operations} 条操作日志、${counts.logins} 条登录日志、${counts.errors} 条错误日志，失败登录 ${counts.loginFailures} 次。`,
    level: healthLevel(counts.errors === 0 && counts.loginFailures === 0, counts.errors === 0),
    levelLabel: counts.errors > 0 ? '有错误' : counts.loginFailures > 0 ? '有失败登录' : '稳定',
    title: '访问/错误趋势',
  },
]);

const activityRows = computed(() => [
  { detail: latest.operation, name: '最近操作' },
  { detail: latest.login, name: '最近登录' },
  { detail: latest.error, name: '最近错误' },
  { detail: latest.release, name: '最近发布' },
  { detail: latest.autoCode, name: '最近蓝图' },
  {
    detail: `令牌 ${counts.apiTokens} / 插件 ${counts.plugins} / 技能 ${counts.skills} / MCP 工具 ${counts.mcpTools}`,
    name: '工具台账',
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

function navigate(path: string) {
  router.push(path).catch(() => undefined);
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
  <Page
    title="管理总览"
  >
    <template #extra>
      <Space>
        <Space>
          <IconifyIcon icon="mdi:autorenew" />
          <span>自动刷新</span>
          <Switch v-model:checked="autoRefresh" />
        </Space>
        <Button :loading="loading" @click="reloadOverview">
          <template #icon>
            <IconifyIcon icon="mdi:refresh" />
          </template>
          立即刷新
        </Button>
      </Space>
    </template>

    <div class="page-stack">
      <Alert
        :message="`最近刷新：${lastRefresh}`"
        :description="`治理覆盖：${coverageLabel}；运行底座：${runtimeSummary}`"
        show-icon
        type="info"
      />
      <Alert v-if="loadError" :message="loadError" show-icon type="error" />

      <Row :gutter="[16, 16]" class="mb-4">
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <Card size="small">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-primary)]" icon="mdi:view-dashboard-outline" />
                <span>管理台账</span>
              </Space>
              <Tag color="processing">Admin</Tag>
            </div>
            <Statistic :value="summary.adminAssets" />
            <div class="metric-card-footer">用户 / 角色 / 菜单 / 接口 / 字典 / 参数</div>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <Card size="small">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-success)]" icon="mdi:tools" />
                <span>工具台账</span>
              </Space>
              <Tag color="success">Tools</Tag>
            </div>
            <Statistic :value="summary.toolAssets" />
            <div class="metric-card-footer">令牌 / 插件 / 自动代码 / 技能 / MCP / 发布</div>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <Card size="small">
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
          </Card>
        </Col>
        <Col :lg="6" :md="12" :sm="24" :xl="6" :xs="24">
          <Card size="small">
            <div class="metric-card-header">
              <Space>
                <IconifyIcon class="metric-icon text-[var(--ant-color-error)]" icon="mdi:bell-alert-outline" />
                <span>待关注项</span>
              </Space>
              <Tag :color="summary.attentionCount > 0 ? 'error' : 'success'">巡检</Tag>
            </div>
            <Statistic :value="summary.attentionCount" />
            <div class="metric-card-footer">错误、失败登录、配置风险与服务不可达</div>
          </Card>
        </Col>
      </Row>

      <Card size="small" title="管理域入口">
        <template #extra>
          <Tag :color="summary.healthScore >= 80 ? 'success' : summary.healthScore >= 60 ? 'warning' : 'error'">
            {{ summary.healthLabel }}
          </Tag>
        </template>
        <Row :gutter="[16, 16]">
          <Col
            v-for="item in managementEntries"
            :key="item.key"
            :lg="8"
            :md="12"
            :sm="24"
            :xs="24"
          >
            <Card class="entry-card" hoverable size="small">
              <div class="entry-header">
                <Space>
                  <IconifyIcon :icon="item.icon" class="metric-icon" />
                  <strong>{{ item.title }}</strong>
                </Space>
                <Tag :color="toneColor(item.level)">{{ item.badge }}</Tag>
              </div>
              <Statistic :value="item.countLabel" class="entry-stat" />
              <div class="metric-card-footer">{{ item.summary }}</div>
              <div class="entry-detail">{{ item.detail }}</div>
              <div class="entry-footer">
                <span class="metric-card-footer">{{ item.signal }}</span>
                <Button size="small" type="link" @click="navigate(item.path)">进入管理</Button>
              </div>
            </Card>
          </Col>
        </Row>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :xl="15" :xs="24">
          <Card class="mb-4" size="small" title="治理摘要">
            <Table
              :columns="ledgerColumns"
              :data-source="ledgerRows"
              :pagination="false"
              row-key="name"
              size="small"
            />
          </Card>
          <Card size="small" title="近期管理活动">
            <Table
              :columns="activityColumns"
              :data-source="activityRows"
              :pagination="false"
              row-key="name"
              size="small"
            />
          </Card>
        </Col>

        <Col :xl="9" :xs="24">
          <Card class="mb-4" size="small" title="关键趋势 / 信号">
            <div class="signal-list">
              <Card v-for="item in signalRows" :key="item.title" size="small">
                <div class="entry-header">
                  <strong>{{ item.title }}</strong>
                  <Tag :color="toneColor(item.level)">{{ item.levelLabel }}</Tag>
                </div>
                <div class="metric-card-footer">{{ item.detail }}</div>
              </Card>
            </div>
          </Card>

          <Card size="small" title="推荐巡检顺序">
            <div class="signal-list">
              <Card v-for="item in suggestionRows" :key="item.title" size="small">
                <div class="entry-header">
                  <strong>{{ item.title }}</strong>
                  <Button size="small" type="link" @click="navigate(item.path)">打开</Button>
                </div>
                <div class="metric-card-footer">{{ item.detail }}</div>
              </Card>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<style scoped>
.page-stack,
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
  color: rgb(100 116 139);
  font-size: 13px;
}

.entry-card,
.entry-stat {
  height: 100%;
}

.entry-detail {
  margin-top: 12px;
}

.entry-footer {
  margin-top: 16px;
}
</style>
