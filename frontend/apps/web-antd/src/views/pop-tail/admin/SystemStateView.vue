<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Alert,
  Button,
  Card,
  Col,
  Form,
  Input,
  Row,
  Select,
  Space,
  Statistic,
  Switch,
  Table,
  Tag,
  message,
} from 'ant-design-vue';

import {
  getApiTokenListApi,
  getAutoCodeRegistryApi,
  getErrorLogsApi,
  getLoginLogsApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getOperationLogsApi,
  getPluginInstallListApi,
  getRuntimeInfoApi,
  getSystemConfigApi,
} from '#/api/pop-tail/admin';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';
import type {
  McpServiceStatus,
  RuntimeInfo,
  SystemConfigInfo,
} from '#/types/pop-tail';

type HealthLevel = 'critical' | 'healthy' | 'warning';
type SourceFilter = '' | 'client' | 'config' | 'log' | 'runtime' | 'tooling';

type LevelFilter = '' | HealthLevel;

type ServiceRow = {
  detail: string;
  level: HealthLevel;
  name: string;
  source: Exclude<SourceFilter, ''>;
  status: string;
};

type SummaryRow = {
  detail: string;
  name: string;
};

const REFRESH_INTERVAL = 15_000;

const loading = ref(false);
const notice = ref('');
const loadError = ref('');
const lastRefresh = ref('-');
const autoRefresh = ref(true);
usePageRefreshLoading(reloadState);

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcpStatus = ref<McpServiceStatus | null>(null);

const metrics = reactive({
  apiTokens: 0,
  autoCode: 0,
  errorLogs: 0,
  loginFailed: 0,
  loginLogs: 0,
  mcpTools: 0,
  operationLogs: 0,
  plugins: 0,
});

const browser = reactive({
  hasToken: false,
  language: '-',
  online: true,
  path: '-',
  screen: '-',
  timezone: '-',
  userAgent: '-',
});

const filters = reactive({
  keyword: '',
  level: '' as LevelFilter,
  source: '' as SourceFilter,
});

const serviceColumns = [
  { title: '组件', dataIndex: 'name', key: 'name', width: 160 },
  { title: '状态', dataIndex: 'status', key: 'status', width: 120 },
  { title: '说明', dataIndex: 'detail', key: 'detail', ellipsis: true },
  { title: '数据源', dataIndex: 'source', key: 'source', width: 120 },
];

const sourceOptions = [
  { label: '全部来源', value: '' },
  { label: '运行时', value: 'runtime' },
  { label: '配置', value: 'config' },
  { label: '工具链', value: 'tooling' },
  { label: '日志', value: 'log' },
  { label: '客户端', value: 'client' },
];

const levelOptions = [
  { label: '全部等级', value: '' },
  { label: '健康', value: 'healthy' },
  { label: '预警', value: 'warning' },
  { label: '严重', value: 'critical' },
];

const serviceRows = computed<ServiceRow[]>(() => {
  const rows: ServiceRow[] = [
    {
      detail: runtime.value
        ? `${runtime.value.os} / ${runtime.value.cpuCores} cores / ${runtime.value.dbBackend}`
        : '尚未获取运行时信息',
      level: runtime.value ? 'healthy' : 'warning',
      name: '运行时',
      source: 'runtime',
      status: runtime.value ? '已同步' : '待同步',
    },
    {
      detail: config.value?.databaseUrl ? maskUrl(config.value.databaseUrl) : '未发现数据库连接',
      level: config.value?.databaseUrl ? 'healthy' : 'critical',
      name: '数据库配置',
      source: 'config',
      status: config.value?.databaseUrl ? '已配置' : '缺失',
    },
    {
      detail: config.value?.redisUrl ? maskUrl(config.value.redisUrl) : '未配置 Redis 地址',
      level: config.value?.redisUrl ? 'healthy' : 'warning',
      name: 'Redis',
      source: 'config',
      status: config.value?.redisUrl ? '已配置' : '未配置',
    },
    {
      detail: config.value?.multipointEnabled ? '多点登录已开启' : '当前为单点会话模式',
      level: config.value?.multipointEnabled ? 'healthy' : 'warning',
      name: '会话策略',
      source: 'config',
      status: config.value?.multipointEnabled ? '多点模式' : '单点模式',
    },
    {
      detail: mcpStatus.value?.message || mcpStatus.value?.lastError || '尚未同步 MCP 状态',
      level: mcpStatus.value?.reachable ? 'healthy' : mcpStatus.value ? 'warning' : 'critical',
      name: 'MCP 服务',
      source: 'tooling',
      status: mcpStatus.value?.state || '未知',
    },
    {
      detail: browser.online
        ? `UA ${browser.userAgent}`
        : '浏览器当前离线，请检查网络或本地代理设置',
      level: browser.online ? 'healthy' : 'critical',
      name: '浏览器联通',
      source: 'client',
      status: browser.online ? '在线' : '离线',
    },
    {
      detail:
        metrics.errorLogs > 0
          ? `最近采样到 ${metrics.errorLogs} 条错误日志`
          : '当前未采样到错误日志',
      level: metrics.errorLogs > 0 ? 'warning' : 'healthy',
      name: '错误日志',
      source: 'log',
      status: metrics.errorLogs > 0 ? '需关注' : '正常',
    },
    {
      detail:
        metrics.loginFailed > 0
          ? `存在 ${metrics.loginFailed} 条失败登录记录`
          : '当前未发现失败登录',
      level: metrics.loginFailed > 0 ? 'warning' : 'healthy',
      name: '登录信号',
      source: 'log',
      status: metrics.loginFailed > 0 ? '异常' : '正常',
    },
  ];

  return rows;
});

const filteredServiceRows = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();

  return serviceRows.value.filter((item) => {
    const matchesKeyword =
      !keyword ||
      item.name.toLowerCase().includes(keyword) ||
      item.detail.toLowerCase().includes(keyword) ||
      item.status.toLowerCase().includes(keyword);
    const matchesLevel = !filters.level || item.level === filters.level;
    const matchesSource = !filters.source || item.source === filters.source;

    return matchesKeyword && matchesLevel && matchesSource;
  });
});

const risks = computed(() => {
  const list: string[] = [];
  if (!config.value?.databaseUrl) {
    list.push('数据库连接缺失');
  }
  if (!config.value?.redisUrl) {
    list.push('Redis 未配置，缓存和会话恢复能力受限');
  }
  if (!mcpStatus.value?.reachable) {
    list.push('MCP 服务不可达');
  }
  if (metrics.errorLogs > 0) {
    list.push(`最近存在 ${metrics.errorLogs} 条错误日志`);
  }
  if (metrics.loginFailed > 0) {
    list.push(`最近存在 ${metrics.loginFailed} 条失败登录`);
  }
  if (!browser.online) {
    list.push('浏览器当前处于离线状态');
  }
  return list;
});

const overview = computed(() => {
  const total = serviceRows.value.length;
  const healthy = serviceRows.value.filter((item) => item.level === 'healthy').length;
  const warning = serviceRows.value.filter((item) => item.level === 'warning').length;
  const critical = serviceRows.value.filter((item) => item.level === 'critical').length;
  const scoreBase = total ? healthy * 100 / total : 0;
  const score = Math.max(0, Math.round(scoreBase - warning * 6 - critical * 12));
  const label = score >= 85 ? '健康' : score >= 60 ? '需关注' : '风险较高';

  return {
    activityCount: metrics.operationLogs + metrics.loginLogs + metrics.errorLogs,
    healthLabel: label,
    healthScore: score,
    readyServices: healthy,
    riskCount: risks.value.length,
    totalServices: total,
  };
});

const inventoryRows = computed<SummaryRow[]>(() => [
  {
    detail: `API 令牌 ${metrics.apiTokens} 个，插件安装 ${metrics.plugins} 条`,
    name: '接入与授权',
  },
  {
    detail: `MCP 工具 ${metrics.mcpTools} 个，自动代码台账 ${metrics.autoCode} 条`,
    name: '工具链',
  },
  {
    detail: `操作日志 ${metrics.operationLogs} 条，登录日志 ${metrics.loginLogs} 条`,
    name: '审计日志',
  },
  {
    detail: `错误日志 ${metrics.errorLogs} 条，失败登录 ${metrics.loginFailed} 条`,
    name: '风险信号',
  },
]);

const activityRows = computed<SummaryRow[]>(() => [
  {
    detail: metrics.operationLogs > 0 ? `最近采样到 ${metrics.operationLogs} 条操作轨迹` : '暂无操作日志样本',
    name: '操作日志',
  },
  {
    detail: metrics.loginLogs > 0 ? `最近采样到 ${metrics.loginLogs} 条登录记录` : '暂无登录日志样本',
    name: '登录日志',
  },
  {
    detail: metrics.errorLogs > 0 ? `最近采样到 ${metrics.errorLogs} 条错误记录` : '暂无错误日志样本',
    name: '错误日志',
  },
]);

const insights = computed(() => [
  {
    label: '当前建议',
    value: risks.value.length > 0 ? '优先处理风险项，再复核工具链与日志信号。' : '当前整体稳定，保持定期巡检即可。',
  },
  {
    label: '刷新策略',
    value: autoRefresh.value ? `自动轮询 ${REFRESH_INTERVAL / 1000} 秒` : '手动刷新',
  },
  {
    label: '最近刷新',
    value: lastRefresh.value,
  },
]);

let refreshTimer: ReturnType<typeof setInterval> | null = null;

function maskUrl(value: string) {
  if (!value) {
    return '-';
  }
  return value.replace(/:\/\/([^:@/]+):([^@/]+)@/, '://$1:***@');
}

function levelColor(level: HealthLevel) {
  switch (level) {
    case 'healthy': {
      return 'success';
    }
    case 'warning': {
      return 'warning';
    }
    default: {
      return 'error';
    }
  }
}

function updateBrowserSnapshot() {
  if (typeof window === 'undefined') {
    return;
  }
  browser.userAgent = navigator.userAgent;
  browser.language = navigator.language;
  browser.online = navigator.onLine;
  browser.path = window.location.pathname;
  browser.screen = `${window.screen.width} × ${window.screen.height}`;
  browser.timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  browser.hasToken = Boolean(localStorage.getItem('token') || sessionStorage.getItem('token'));
}

function syncAutoRefreshTimer() {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
  if (autoRefresh.value) {
    refreshTimer = setInterval(() => {
      void reloadState();
    }, REFRESH_INTERVAL);
  }
}

function resetFilters() {
  filters.keyword = '';
  filters.level = '';
  filters.source = '';
  notice.value = '已重置状态筛选条件。';
}

async function reloadState() {
  loading.value = true;
  loadError.value = '';

  try {
    updateBrowserSnapshot();

    const [
      runtimeResult,
      configResult,
      mcpStatusResult,
      operationResult,
      loginResult,
      errorResult,
      tokenResult,
      pluginResult,
      mcpToolResult,
      autoCodeResult,
    ] = await Promise.allSettled([
      getRuntimeInfoApi(),
      getSystemConfigApi(),
      getMcpStatusApi(),
      getOperationLogsApi(),
      getLoginLogsApi(),
      getErrorLogsApi(),
      getApiTokenListApi(),
      getPluginInstallListApi(),
      getMcpToolListApi(),
      getAutoCodeRegistryApi(),
    ]);

    if (runtimeResult.status === 'fulfilled') {
      runtime.value = runtimeResult.value;
    }
    if (configResult.status === 'fulfilled') {
      config.value = configResult.value;
    }
    if (mcpStatusResult.status === 'fulfilled') {
      mcpStatus.value = mcpStatusResult.value;
    }
    if (operationResult.status === 'fulfilled') {
      metrics.operationLogs = operationResult.value.Total;
    }
    if (loginResult.status === 'fulfilled') {
      metrics.loginLogs = loginResult.value.Total;
      metrics.loginFailed = loginResult.value.List.filter((item) => !item.status).length;
    }
    if (errorResult.status === 'fulfilled') {
      metrics.errorLogs = errorResult.value.Total;
    }
    if (tokenResult.status === 'fulfilled') {
      metrics.apiTokens = tokenResult.value.Total;
    }
    if (pluginResult.status === 'fulfilled') {
      metrics.plugins = pluginResult.value.Total;
    }
    if (mcpToolResult.status === 'fulfilled') {
      metrics.mcpTools = mcpToolResult.value.length;
    }
    if (autoCodeResult.status === 'fulfilled') {
      metrics.autoCode = autoCodeResult.value.Total;
    }

    const failures = [
      runtimeResult,
      configResult,
      mcpStatusResult,
      operationResult,
      loginResult,
      errorResult,
      tokenResult,
      pluginResult,
      mcpToolResult,
      autoCodeResult,
    ].filter((result) => result.status === 'rejected');

    if (failures.length > 0) {
      loadError.value = `${failures.length} 个数据源同步失败`;
    }

    lastRefresh.value = new Date().toLocaleString();
    notice.value = '系统状态已更新。';
  } catch (error) {
    const text = error instanceof Error ? error.message : '刷新系统状态失败';
    loadError.value = text;
    notice.value = text;
    message.error(text);
  } finally {
    loading.value = false;
  }
}

watch(autoRefresh, () => {
  syncAutoRefreshTimer();
});

onMounted(() => {
  updateBrowserSnapshot();
  syncAutoRefreshTimer();
  void reloadState();
  window.addEventListener('online', updateBrowserSnapshot);
  window.addEventListener('offline', updateBrowserSnapshot);
});

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer);
  }
  window.removeEventListener('online', updateBrowserSnapshot);
  window.removeEventListener('offline', updateBrowserSnapshot);
});
</script>

<template>
  <Page
    title="系统状态"
  >
    <div class="page-stack relative flex h-full min-h-0 flex-1 flex-col">
      <Card :bordered="false">
        <template #title>
          <div class="panel-title-block">
            <div class="panel-title">系统状态工作台</div>
            <div class="panel-subtitle">聚合运行时、配置、工具链和日志信号，作为环境与健康观测页使用。</div>
          </div>
        </template>
        <template #extra>
          <Space wrap>
            <Space align="center">
              <span class="stat-note">自动刷新</span>
              <Switch v-model:checked="autoRefresh" />
            </Space>
            <Button :loading="loading" type="primary" @click="reloadState">立即刷新</Button>
          </Space>
        </template>

        <Alert
          v-if="notice || loadError"
          :message="loadError || notice"
          banner
          show-icon
          :type="loadError ? 'warning' : 'info'"
        />
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="6" :md="12" :span="24">
          <Card :bordered="false">
            <Statistic :value="overview.healthScore" suffix="%" title="总体健康" />
            <div class="stat-note">{{ overview.healthLabel }}</div>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :span="24">
          <Card :bordered="false">
            <Statistic :value="overview.readyServices" :suffix="`/ ${overview.totalServices}`" title="服务就绪" />
            <div class="stat-note">运行链路可用项</div>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :span="24">
          <Card :bordered="false">
            <Statistic :value="overview.riskCount" title="配置风险" />
            <div class="stat-note">需关注项</div>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :span="24">
          <Card :bordered="false">
            <Statistic :value="overview.activityCount" title="活动信号" />
            <div class="stat-note">最近日志与台账汇总</div>
          </Card>
        </Col>
      </Row>

      <Card :bordered="false" title="筛选与风险聚焦">
        <Form layout="vertical">
          <Row :gutter="[16, 0]">
            <Col :lg="10" :md="12" :span="24">
              <Form.Item label="关键词">
                <Input v-model:value="filters.keyword" allow-clear placeholder="组件 / 状态 / 说明" />
              </Form.Item>
            </Col>
            <Col :lg="7" :md="12" :span="24">
              <Form.Item label="健康等级">
                <Select v-model:value="filters.level" :options="levelOptions" />
              </Form.Item>
            </Col>
            <Col :lg="7" :md="12" :span="24">
              <Form.Item label="来源">
                <Select v-model:value="filters.source" :options="sourceOptions" />
              </Form.Item>
            </Col>
          </Row>
        </Form>

        <div class="action-row">
          <Space wrap>
            <Button :loading="loading" type="primary" @click="reloadState">同步数据</Button>
            <Button @click="resetFilters">重置筛选</Button>
          </Space>
          <div class="action-summary">{{ insights[0]?.value }}</div>
        </div>

        <div class="risk-list">
          <Tag v-for="risk in risks" :key="risk" color="warning">{{ risk }}</Tag>
          <Tag v-if="risks.length === 0" color="success">当前未发现明显风险</Tag>
        </div>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :xl="16" :span="24">
          <Card :bordered="false" title="服务健康矩阵">
            <Table
              :columns="serviceColumns"
              :data-source="filteredServiceRows"
              :pagination="false"
              :row-key="'name'"
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'status'">
                  <Tag :color="levelColor(record.level)">{{ record.status }}</Tag>
                </template>
                <template v-else-if="column.key === 'source'">
                  <Tag color="blue">{{ record.source }}</Tag>
                </template>
              </template>
            </Table>
          </Card>
        </Col>

        <Col :xl="8" :span="24">
          <Card :bordered="false" title="详情 / 说明">
            <div class="detail-list">
              <div class="detail-item">
                <span class="detail-label">最近刷新</span>
                <span>{{ lastRefresh }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">MCP 状态</span>
                <span>{{ mcpStatus?.message || mcpStatus?.state || '未接入' }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">浏览器联通</span>
                <span>{{ browser.online ? '在线' : '离线' }}</span>
              </div>
              <div class="detail-item detail-item-block">
                <span class="detail-label">当前建议</span>
                <span>{{ insights[0]?.value }}</span>
              </div>
            </div>

            <div class="insight-list">
              <div v-for="row in insights" :key="row.label" class="insight-item">
                <div class="insight-label">{{ row.label }}</div>
                <div class="insight-value">{{ row.value }}</div>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :xl="12" :span="24">
          <Card :bordered="false" title="环境与配置快照">
            <div class="detail-list">
              <div class="detail-item"><span class="detail-label">服务端操作系统</span><span>{{ runtime?.os || '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">CPU 核数</span><span>{{ runtime?.cpuCores || '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">Rust 版本</span><span>{{ runtime?.rustVersion || '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">数据库后端</span><span>{{ runtime?.dbBackend || '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">Bind Address</span><span>{{ config?.bindAddress || '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">Database URL</span><span>{{ config?.databaseUrl ? maskUrl(config.databaseUrl) : '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">Redis URL</span><span>{{ config?.redisUrl ? maskUrl(config.redisUrl) : '-' }}</span></div>
              <div class="detail-item"><span class="detail-label">Compatibility Headers</span><span>{{ config?.compatibilityRefreshHeaders ? 'enabled' : 'disabled' }}</span></div>
            </div>
          </Card>
        </Col>

        <Col :xl="12" :span="24">
          <Card :bordered="false" title="浏览器与前端观测">
            <div class="detail-list">
              <div class="detail-item"><span class="detail-label">User Agent</span><span>{{ browser.userAgent }}</span></div>
              <div class="detail-item"><span class="detail-label">Language</span><span>{{ browser.language }}</span></div>
              <div class="detail-item"><span class="detail-label">Screen</span><span>{{ browser.screen }}</span></div>
              <div class="detail-item"><span class="detail-label">Timezone</span><span>{{ browser.timezone }}</span></div>
              <div class="detail-item"><span class="detail-label">Online</span><span>{{ browser.online ? 'true' : 'false' }}</span></div>
              <div class="detail-item"><span class="detail-label">当前路径</span><span>{{ browser.path }}</span></div>
              <div class="detail-item"><span class="detail-label">Token 状态</span><span>{{ browser.hasToken ? '已写入' : '未发现' }}</span></div>
            </div>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :xl="12" :span="24">
          <Card :bordered="false" title="能力台账">
            <div class="detail-list">
              <div v-for="row in inventoryRows" :key="row.name" class="detail-item detail-item-block">
                <span class="detail-label">{{ row.name }}</span>
                <span>{{ row.detail }}</span>
              </div>
            </div>
          </Card>
        </Col>

        <Col :xl="12" :span="24">
          <Card :bordered="false" title="最近活动">
            <div class="detail-list">
              <div v-for="row in activityRows" :key="row.name" class="detail-item detail-item-block">
                <span class="detail-label">{{ row.name }}</span>
                <span>{{ row.detail }}</span>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<style scoped>
.page-stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-title-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.panel-title {
  font-size: 16px;
  font-weight: 600;
}

.panel-subtitle,
.stat-note,
.action-summary,
.insight-value {
  color: rgb(100 116 139);
  font-size: 13px;
  line-height: 1.6;
}

.auto-select {
  min-width: 120px;
}

.action-row {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: space-between;
  margin-bottom: 12px;
}

.risk-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.detail-list,
.insight-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-item,
.insight-item {
  background: rgb(248 250 252);
  border: 1px solid rgb(226 232 240);
  border-radius: 12px;
  padding: 12px;
}

.detail-item {
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.detail-item-block {
  align-items: flex-start;
}

.detail-label,
.insight-label {
  color: rgb(15 23 42);
  font-size: 13px;
  font-weight: 600;
}
</style>
