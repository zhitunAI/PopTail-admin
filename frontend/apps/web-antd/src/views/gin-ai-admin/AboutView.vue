
<template>
  <Page
    title="关于系统"
  >
    <div class="vben-page about-page">
      <Card :bordered="false" class="hero-card">
        <div class="hero-head">
          <div>
            <div class="hero-title-row">
              <IconifyIcon class="hero-icon" icon="carbon:information" />
              <div>
                <h2>项目、运行态与交付概览</h2>
                <p>
                  结合真实运行态、配置和台账，快速判断当前恢复范围、工具链状态与交付边界。
                </p>
              </div>
            </div>
          </div>
          <Space wrap>
            <Button :loading="loading" type="primary" @click="reloadOverview">
              <template #icon>
                <IconifyIcon icon="ant-design:reload-outlined" />
              </template>
              刷新概览
            </Button>
          </Space>
        </div>

        <Alert
          :description="loadError ? `最近错误：${loadError}` : '已同步当前浏览器环境、配置和后台服务信号。'"
          :message="`最近同步：${lastRefresh}`"
          :type="loadError ? 'warning' : 'info'"
          show-icon
        />
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="overview.restoredDomains" title="恢复域数">
              <template #prefix>
                <IconifyIcon icon="carbon:category" />
              </template>
            </Statistic>
            <p>已接入真实读写/状态化模块</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :precision="0" :value="overview.architectureScore" suffix="%" title="架构就绪度">
              <template #prefix>
                <IconifyIcon icon="carbon:ibm-cloud-projects" />
              </template>
            </Statistic>
            <p>{{ architectureLabel }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="overview.deliverySignals" title="交付信号">
              <template #prefix>
                <IconifyIcon icon="carbon:delivery" />
              </template>
            </Statistic>
            <p>发布 / 自动代码 / 插件 / 技能</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="overview.observabilitySignals" title="运行观测">
              <template #prefix>
                <IconifyIcon icon="carbon:monitoring" />
              </template>
            </Statistic>
            <p>日志 / 队列 / MCP / 浏览器在线</p>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="项目基线">
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="项目代号">gin-ai-admin</DescriptionsItem>
              <DescriptionsItem label="当前路径">{{ browser.path }}</DescriptionsItem>
              <DescriptionsItem label="前端实现">Vue 3 + Vite + Pinia + Vben / Ant Design Vue</DescriptionsItem>
              <DescriptionsItem label="后端实现">Rust + Axum + 状态化管理接口</DescriptionsItem>
              <DescriptionsItem label="运行平台">{{ runtime?.os ?? '未获取' }}</DescriptionsItem>
              <DescriptionsItem label="CPU 核数">{{ runtime?.cpuCores ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="Rust 版本">{{ runtime?.rustVersion ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="数据库后端">{{ runtime?.dbBackend ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="Redis 能力">{{ runtime?.redisEnabled ? '已启用' : '未启用' }}</DescriptionsItem>
              <DescriptionsItem label="服务监听">{{ config?.bindAddress ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="数据库连接">{{ maskedDatabaseUrl }}</DescriptionsItem>
              <DescriptionsItem label="Redis 连接">{{ maskedRedisUrl }}</DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="架构与约束">
            <Space class="tag-wrap" wrap>
              <Tag color="success">功能 1:1 对齐目标</Tag>
              <Tag color="success">clean-room 命名</Tag>
              <Tag color="success">不改 legacy server/web</Tag>
              <Tag :color="runtime?.redisEnabled ? 'success' : 'warning'">
                Redis {{ runtime?.redisEnabled ? 'ready' : 'pending' }}
              </Tag>
              <Tag :color="config?.multipointEnabled ? 'processing' : 'warning'">
                {{ config?.multipointEnabled ? '多点登录开启' : '单点会话模式' }}
              </Tag>
              <Tag :color="mcpRunning ? 'success' : 'warning'">
                MCP {{ mcpRunning ? 'running' : 'not running' }}
              </Tag>
              <Tag :color="browser.online ? 'success' : 'error'">
                浏览器{{ browser.online ? '在线' : '离线' }}
              </Tag>
            </Space>
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="会话策略">
                {{ config?.multipointEnabled ? '多点并发会话已接入' : '当前仍按单点行为工作' }}
              </DescriptionsItem>
              <DescriptionsItem label="刷新兼容">
                {{ config?.compatibilityRefreshHeaders ? '兼容刷新头已开启' : '兼容刷新头关闭' }}
              </DescriptionsItem>
              <DescriptionsItem label="MCP 服务">{{ mcpStatusText }}</DescriptionsItem>
              <DescriptionsItem label="技能资产">
                {{ metrics.skills }} 项技能 / {{ metrics.skillTools }} 个工具定义
              </DescriptionsItem>
              <DescriptionsItem label="插件能力">
                {{ metrics.pluginInstalls }} 条安装记录 / {{ metrics.packages }} 个包定义
              </DescriptionsItem>
              <DescriptionsItem label="自动代码">
                {{ metrics.autoCode }} 条台账 / {{ metrics.mcpTools }} 个 MCP 工具
              </DescriptionsItem>
              <DescriptionsItem label="示例链路">{{ exampleSummary }}</DescriptionsItem>
              <DescriptionsItem label="交付基线">{{ latestReleaseText }}</DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
      </Row>

      <Card :bordered="false" class="panel-card" title="恢复覆盖矩阵">
        <template #extra>
          <Tag color="processing">{{ coverageRows.length }} 个关键域</Tag>
        </template>
        <Table
          :columns="[
            { title: '域', dataIndex: 'name', key: 'name' },
            { title: '规模', dataIndex: 'count', key: 'count' },
            { title: '当前信号', dataIndex: 'detail', key: 'detail' },
            { title: '数据来源', dataIndex: 'source', key: 'source' },
          ]"
          :data-source="coverageRows"
          :pagination="false"
          row-key="name"
          size="small"
        />
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="交付与运行看板">
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="最近发布">{{ latestReleaseText }}</DescriptionsItem>
              <DescriptionsItem label="最近操作">{{ latestOperationText }}</DescriptionsItem>
              <DescriptionsItem label="最近登录">{{ latestLoginText }}</DescriptionsItem>
              <DescriptionsItem label="最近错误">{{ latestErrorText }}</DescriptionsItem>
              <DescriptionsItem label="上传队列">{{ uploadSummary }}</DescriptionsItem>
              <DescriptionsItem label="断点续传">{{ resumeSummary }}</DescriptionsItem>
              <DescriptionsItem label="扫码会话">{{ scanSummary }}</DescriptionsItem>
              <DescriptionsItem label="Token 台账">{{ tokenSummary }}</DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="交付说明">
            <Space class="tag-wrap" wrap>
              <Tag v-for="note in deliveryNotes" :key="note" color="default">
                {{ note }}
              </Tag>
            </Space>
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="前端在线状态">
                {{ browser.online ? 'online' : 'offline' }}
              </DescriptionsItem>
              <DescriptionsItem label="浏览器环境">
                {{ browser.language }} / {{ browser.timezone }}
              </DescriptionsItem>
              <DescriptionsItem label="屏幕尺寸">{{ browser.screen }}</DescriptionsItem>
              <DescriptionsItem label="能力路线">
                Web 先恢复一致行为，后续进入 Rust + Tauri 2
              </DescriptionsItem>
              <DescriptionsItem label="数据面">
                {{ maskedDatabaseUrl }} ｜ {{ maskedRedisUrl }}
              </DescriptionsItem>
              <DescriptionsItem label="变更基调">
                最小必要依赖、避免回退并行代理工作
              </DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<script setup lang="ts">
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { Alert, Button, Card, Col, Descriptions, DescriptionsItem, Row, Space, Statistic, Table, Tag } from 'ant-design-vue';
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
} from "#/api/gin-ai-admin/admin";
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
} from "#/types/gin-ai-admin";

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
  const [first, ...rest] = resumes.value;
  if (!first) return "暂无续传任务";
  const highest = rest.reduce(
    (best, item) => (item.progress > best.progress ? item : best),
    first,
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
.vben-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-card,
.panel-card,
.metric-card {
  border-radius: 16px;
}

.hero-head,
.hero-title-row,
.record-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.hero-title-row {
  align-items: center;
}

.hero-icon,
.result-icon {
  font-size: 28px;
  color: var(--ant-color-primary);
}

.hero-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric-card p,
.panel-card p,
.hero-card p,
.field-hint,
.soft-text,
.footnote,
.selection-card span {
  margin: 0;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.panel-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-wrap {
  width: 100%;
}

.alert-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.top-gap {
  margin-top: 12px;
}

.record-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.record-item,
.selection-card,
.inner-card :deep(.ant-card-body) {
  border: 1px solid var(--ant-color-border-secondary);
  border-radius: 14px;
  padding: 12px 14px;
  background: var(--ant-color-fill-quaternary);
}

.selection-card {
  text-align: left;
  width: 100%;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.selection-card:hover {
  border-color: var(--ant-color-primary);
  transform: translateY(-1px);
}

.selection-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-caption {
  margin: 0 0 8px;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.full-width {
  width: 100%;
}

.toolbar-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toolbar-field label,
.section-head h3,
.hero-card h2,
.inner-card h3 {
  margin: 0;
}

.section-head p {
  margin: 4px 0 0;
}

.profile-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.form-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.bullet-list {
  margin: 0;
  padding-left: 18px;
  color: var(--ant-color-text-description);
}

.bullet-list li + li {
  margin-top: 6px;
}

.session-summary {
  display: flex;
  gap: 12px;
  align-items: center;
}

.summary-avatar {
  background: color-mix(in srgb, var(--ant-color-primary) 12%, white);
  color: var(--ant-color-primary);
}

@media (max-width: 768px) {
  .hero-head,
  .hero-title-row,
  .record-head,
  .form-meta {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
