
<template>
  <Page
    title="控制台总控工作台"
  >
    <div class="vben-page dashboard-page">
      <Card :bordered="false" class="hero-card">
        <div class="hero-head">
          <div>
            <div class="hero-title-row">
              <IconifyIcon class="hero-icon" icon="carbon:dashboard" />
              <div>
                <h2>首页巡检与角色切换中心</h2>
                <p>统一查看登录态、角色覆盖、运行底座与今日优先事项，减少在多个页面之间往返确认。</p>
              </div>
            </div>
          </div>
          <Space wrap>
            <Button :loading="loading" type="primary" @click="reload">
              <template #icon>
                <IconifyIcon icon="ant-design:reload-outlined" />
              </template>
              刷新鉴权与运行态
            </Button>
            <Button @click="openDefaultRoute">
              <template #icon>
                <IconifyIcon icon="carbon:launch" />
              </template>
              进入当前默认入口
            </Button>
            <Button :disabled="switching || !recoverableRole" @click="restorePreviousAuthority">
              <template #icon>
                <IconifyIcon icon="carbon:undo" />
              </template>
              恢复上一个角色
            </Button>
          </Space>
        </div>

        <Space class="tag-wrap" wrap>
          <Tag color="processing">登录态：{{ auth.isLoggedIn ? '已接入' : '未登录' }}</Tag>
          <Tag color="blue">主角色：{{ currentRoleLabel }}</Tag>
          <Tag color="default">备选角色：{{ alternateAuthorities.length }}</Tag>
          <Tag color="purple">权限覆盖：{{ coverageLabel }}</Tag>
          <Tag :color="runtime ? 'success' : 'warning'">运行态：{{ runtimeBadge }}</Tag>
          <Tag color="default">最近刷新：{{ lastRefresh }}</Tag>
        </Space>

        <Row :gutter="[16, 16]">
          <Col :lg="6" :md="12" :xs="24">
            <Card :bordered="false" class="metric-card">
              <Statistic :value="auth.userInfo?.userName ?? '-'" title="当前用户">
                <template #prefix>
                  <IconifyIcon icon="carbon:user-avatar" />
                </template>
              </Statistic>
              <p>{{ auth.userInfo?.nickName || '未设置昵称' }}</p>
            </Card>
          </Col>
          <Col :lg="6" :md="12" :xs="24">
            <Card :bordered="false" class="metric-card">
              <Statistic :value="auth.policyPaths.length" title="权限路径">
                <template #prefix>
                  <IconifyIcon icon="carbon:security" />
                </template>
              </Statistic>
              <p>{{ methodSummary }}</p>
            </Card>
          </Col>
          <Col :lg="6" :md="12" :xs="24">
            <Card :bordered="false" class="metric-card">
              <Statistic :precision="0" :value="healthScore" suffix="%" title="健康摘要">
                <template #prefix>
                  <IconifyIcon icon="carbon:activity" />
                </template>
              </Statistic>
              <p>{{ healthSummary }}</p>
            </Card>
          </Col>
          <Col :lg="6" :md="12" :xs="24">
            <Card :bordered="false" class="metric-card">
              <Statistic :value="todayFocus.length" title="今日关注">
                <template #prefix>
                  <IconifyIcon icon="carbon:task" />
                </template>
              </Statistic>
              <p>{{ focusLead }}</p>
            </Card>
          </Col>
        </Row>

        <div class="alert-stack">
          <Alert v-if="feedback" :message="feedback" show-icon type="success" />
          <Alert v-if="error" :message="error" show-icon type="error" />
        </div>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="可恢复角色切换">
            <template #extra>
              <Tag :color="recoverableRole ? 'warning' : 'success'">
                {{ recoverableRole ? '存在可恢复角色' : '当前无待恢复角色' }}
              </Tag>
            </template>
            <div class="toolbar-field">
              <label>切换到角色（AuthorityId）</label>
              <Select
                v-model:value="selectedAuthorityId"
                :disabled="switching || !auth.userInfo?.authorities?.length"
                :options="(auth.userInfo?.authorities ?? []).map((authority) => ({
                  label: `${authority.authorityName} (${authority.authorityId})`,
                  value: authority.authorityId,
                }))"
                class="full-width"
              />
            </div>
            <Space wrap>
              <Button
                :disabled="switching || selectedAuthorityId === auth.userInfo?.authorityId"
                :loading="switching"
                type="primary"
                @click="switchAuthority"
              >
                <template #icon>
                  <IconifyIcon icon="carbon:renew" />
                </template>
                切换角色并刷新权限包
              </Button>
              <Button :disabled="switching || !recoverableRole" @click="restorePreviousAuthority">
                回滚到上一个角色
              </Button>
            </Space>
            <Descriptions :column="1" bordered class="top-gap" size="small">
              <DescriptionsItem label="当前主角色">{{ currentRoleLabel }}</DescriptionsItem>
              <DescriptionsItem label="默认入口">{{ auth.userInfo?.authority?.defaultRouter ?? 'dashboard' }}</DescriptionsItem>
              <DescriptionsItem label="可恢复角色">
                {{ recoverableRole ? `${recoverableRole.authorityName} (${recoverableRole.authorityId})` : '暂无' }}
              </DescriptionsItem>
              <DescriptionsItem label="最近切换记录">
                {{ roleHistory.length ? `${roleHistory.length} 条` : '暂无' }}
              </DescriptionsItem>
            </Descriptions>

            <div v-if="alternateAuthorities.length" class="top-gap">
              <p class="section-caption">备选角色</p>
              <Space class="tag-wrap" wrap>
                <Tag v-for="authority in alternateAuthorities" :key="authority.authorityId" color="blue">
                  {{ authority.authorityName }} / {{ authority.authorityId }}
                </Tag>
              </Space>
            </div>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="健康摘要 / 风险提醒">
            <template #extra>
              <Tag :color="healthScore >= 85 ? 'success' : healthScore >= 60 ? 'warning' : 'error'">
                {{ healthSummary }}
              </Tag>
            </template>
            <Progress
              :percent="healthScore"
              :status="healthScore >= 85 ? 'success' : healthScore >= 60 ? 'active' : 'exception'"
            />
            <Descriptions :column="1" bordered class="top-gap" size="small">
              <DescriptionsItem label="服务环境">{{ runtime?.os ?? '未加载' }}</DescriptionsItem>
              <DescriptionsItem label="数据库底座">{{ runtime?.dbBackend ?? '未加载' }}</DescriptionsItem>
              <DescriptionsItem label="Redis 状态">{{ runtime?.redisEnabled ? '已接入' : '未启用' }}</DescriptionsItem>
              <DescriptionsItem label="CPU 核数">{{ runtime?.cpuCores ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="服务版本">{{ runtime?.rustVersion ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="权限分组">{{ permissionHotspots.length }} 组</DescriptionsItem>
            </Descriptions>
            <div class="record-list top-gap">
              <div v-for="item in attentionItems" :key="item.title" class="record-item">
                <div class="record-head">
                  <strong>{{ item.title }}</strong>
                  <Tag :color="item.level === 'bad' ? 'error' : item.level === 'warn' ? 'warning' : 'success'">
                    {{ item.levelLabel }}
                  </Tag>
                </div>
                <p>{{ item.detail }}</p>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Card :bordered="false" class="panel-card" title="快捷入口分组">
        <template #extra>
          <Tag color="processing">
            推荐入口 {{ quickEntryGroups.flatMap((group) => group.items).length }} 个
          </Tag>
        </template>
        <Row :gutter="[16, 16]">
          <Col v-for="group in quickEntryGroups" :key="group.title" :lg="12" :xs="24">
            <Card class="inner-card" size="small">
              <div class="record-head">
                <div>
                  <h3>{{ group.title }}</h3>
                  <p>{{ group.summary }}</p>
                </div>
                <Tag>{{ group.items.length }} 项</Tag>
              </div>
              <Space class="tag-wrap top-gap" wrap>
                <Button
                  v-for="item in group.items"
                  :key="item.name"
                  :disabled="!router.hasRoute(item.name)"
                  @click="jumpTo(item.name)"
                >
                  {{ item.label }}
                </Button>
              </Space>
              <p class="footnote">{{ group.detail }}</p>
            </Card>
          </Col>
        </Row>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="14" :xs="24">
          <Card :bordered="false" class="panel-card" title="权限热点">
            <Table
              :columns="[
                { title: '分组', dataIndex: 'name', key: 'name' },
                { title: '接口数', dataIndex: 'count', key: 'count' },
                { title: '方法', dataIndex: 'methods', key: 'methods' },
                { title: '热点路径', dataIndex: 'sample', key: 'sample' },
                { title: '关注级别', dataIndex: 'levelLabel', key: 'levelLabel' },
              ]"
              :data-source="permissionHotspots"
              :pagination="false"
              row-key="name"
              size="small"
            />
            <TypographyParagraph v-if="policiesText" class="top-gap" copyable>
              {{ policiesText }}
            </TypographyParagraph>
          </Card>
        </Col>
        <Col :lg="10" :xs="24">
          <Card :bordered="false" class="panel-card" title="今日关注面板">
            <div v-if="todayFocus.length" class="record-list">
              <div v-for="item in todayFocus" :key="item.title" class="record-item">
                <div class="record-head">
                  <strong>{{ item.title }}</strong>
                  <Button v-if="item.routeName" size="small" @click="jumpTo(item.routeName)">
                    打开
                  </Button>
                </div>
                <p>{{ item.detail }}</p>
              </div>
            </div>
            <Empty v-else description="当前无高优先事项" />
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="最近活动">
            <template #extra>
              <Space>
                <Tag color="default">{{ recentActivities.length }} 条</Tag>
                <Button size="small" @click="clearActivities">清空活动记录</Button>
              </Space>
            </template>
            <div v-if="recentActivities.length" class="record-list">
              <div v-for="item in recentActivities" :key="item.id" class="record-item">
                <div class="record-head">
                  <strong>{{ item.label }}</strong>
                  <Tag :color="item.result.includes('失败') ? 'error' : 'success'">{{ item.result }}</Tag>
                </div>
                <p>{{ item.time }}</p>
              </div>
            </div>
            <Empty v-else description="暂无活动记录" />
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="角色恢复台账">
            <div v-if="roleHistory.length" class="record-list">
              <div v-for="item in roleHistory" :key="item.id" class="record-item">
                <div class="record-head">
                  <strong>{{ item.authorityName }} ({{ item.authorityId }})</strong>
                  <span class="soft-text">{{ item.changedAt }}</span>
                </div>
                <p>{{ item.note }}</p>
              </div>
            </div>
            <Empty v-else description="暂无角色恢复台账" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<script setup lang="ts">
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { Alert, Button, Card, Col, Descriptions, DescriptionsItem, Empty, Progress, Row, Select, Space, Statistic, Table, Tag, TypographyParagraph } from 'ant-design-vue';
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "#/store/gin-ai-admin/auth";
import { getRuntimeInfoApi } from "#/api/gin-ai-admin/admin";
import type { AuthorityInfo, RuntimeInfo } from "#/types/gin-ai-admin";

type ActivityRecord = {
  id: string;
  time: string;
  label: string;
  result: string;
};

type RoleSnapshot = {
  id: string;
  authorityId: number;
  authorityName: string;
  changedAt: string;
  note: string;
};

type FocusItem = {
  title: string;
  detail: string;
  routeName?: string;
};

const ACTIVITY_KEY = "gaa-dashboard-activities";
const ROLE_HISTORY_KEY = "gaa-dashboard-role-history";

const auth = useAuthStore();
const router = useRouter();
const error = ref("");
const feedback = ref("");
const loading = ref(false);
const switching = ref(false);
const runtime = ref<RuntimeInfo | null>(null);
const selectedAuthorityId = ref(auth.userInfo?.authorityId ?? 888);
const recentActivities = ref<ActivityRecord[]>(readLocal<ActivityRecord[]>(ACTIVITY_KEY, []));
const roleHistory = ref<RoleSnapshot[]>(readLocal<RoleSnapshot[]>(ROLE_HISTORY_KEY, []));

const alternateAuthorities = computed(() =>
  (auth.userInfo?.authorities ?? []).filter((authority) => authority.authorityId !== auth.userInfo?.authorityId),
);

const currentRoleLabel = computed(
  () => auth.userInfo?.authority?.authorityName || auth.userInfo?.authorityId?.toString() || "-",
);

const policiesText = computed(() => auth.policyPaths.map((p) => `${p.method} ${p.path}`).join("\n"));

const runtimeBadge = computed(() => {
  if (!runtime.value) {
    return "待拉取";
  }
  const redis = runtime.value.redisEnabled ? "Redis 已接入" : "Redis 未启用";
  return `${runtime.value.os} / ${runtime.value.dbBackend} / ${redis}`;
});

const coverageLabel = computed(() => {
  const count = auth.policyPaths.length;
  if (count >= 20) return "高覆盖";
  if (count >= 8) return "中覆盖";
  if (count > 0) return "基础覆盖";
  return "待同步";
});

const methodSummary = computed(() => {
  const methods = new Set(auth.policyPaths.map((item) => item.method));
  return methods.size ? `涉及 ${methods.size} 类请求方法` : "尚未载入权限包";
});

const healthScore = computed(() => {
  let score = 100;
  if (!auth.isLoggedIn) score -= 40;
  if (!runtime.value) score -= 20;
  if (runtime.value && !runtime.value.redisEnabled) score -= 15;
  if (!auth.policyPaths.length) score -= 25;
  if ((auth.userInfo?.authorities?.length ?? 0) <= 1) score -= 5;
  if (recentActivities.value.slice(0, 3).some((item) => item.result.includes("失败"))) score -= 10;
  return Math.max(20, Math.min(100, score));
});

const healthSummary = computed(() => {
  if (healthScore.value >= 85) return "运行平稳";
  if (healthScore.value >= 65) return "需要留意";
  return "建议优先巡检";
});

const recoverableRole = computed(() => roleHistory.value[0] ?? null);

const permissionHotspots = computed(() => {
  const grouped = new Map<string, { count: number; methods: Set<string>; sample: string[] }>();
  for (const policy of auth.policyPaths) {
    const parts = policy.path.split("/").filter(Boolean);
    const key = parts.slice(0, 2).join("/") || "root";
    const bucket = grouped.get(key) ?? { count: 0, methods: new Set<string>(), sample: [] };
    bucket.count += 1;
    bucket.methods.add(policy.method);
    if (bucket.sample.length < 2) {
      bucket.sample.push(policy.path);
    }
    grouped.set(key, bucket);
  }

  return [...grouped.entries()]
    .map(([name, value]) => {
      const levelLabel = value.count >= 5 ? "高" : value.count >= 3 ? "中" : "低";
      return {
        name,
        count: value.count,
        methods: [...value.methods].join(" / "),
        sample: value.sample.join("；"),
        levelLabel,
      };
    })
    .sort((a, b) => b.count - a.count)
    .slice(0, 6);
});

const attentionItems = computed(() => {
  const items = [] as Array<{ title: string; detail: string; level: string; levelLabel: string }>;

  if (!runtime.value) {
    items.push({
      title: "运行态尚未同步",
      detail: "首页还没有拿到当前服务信息，建议先执行一次刷新，确认数据库与 Redis 底座状态。",
      level: "warn",
      levelLabel: "待同步",
    });
  }

  if (runtime.value && !runtime.value.redisEnabled) {
    items.push({
      title: "Redis 未启用",
      detail: "当前运行摘要显示 Redis 未接入，会影响会话、多点登录与状态化能力的完整性。",
      level: "bad",
      levelLabel: "重点",
    });
  }

  if (!auth.policyPaths.length) {
    items.push({
      title: "权限包为空",
      detail: "当前主角色没有加载到权限路径，可能是鉴权包尚未同步或角色权限配置不足。",
      level: "bad",
      levelLabel: "高风险",
    });
  }

  if ((auth.userInfo?.authorities?.length ?? 0) > 1) {
    items.push({
      title: "存在多角色可切换",
      detail: `当前账号可切换 ${alternateAuthorities.value.length} 个备选角色，适合在首页完成权限回归与入口核对。`,
      level: "good",
      levelLabel: "可用",
    });
  }

  if (recentActivities.value[0]?.result.includes("失败")) {
    items.push({
      title: "最近动作包含失败",
      detail: `最近一次首页动作“${recentActivities.value[0].label}”失败，建议先恢复角色或重新拉取鉴权。`,
      level: "warn",
      levelLabel: "关注",
    });
  }

  return items.length
    ? items
    : [
        {
          title: "当前无高风险项",
          detail: "鉴权包、角色切换和运行态看起来都比较平稳，可以继续处理业务配置与例行巡检。",
          level: "good",
          levelLabel: "稳定",
        },
      ];
});

const todayFocus = computed<FocusItem[]>(() => {
  const items: FocusItem[] = [];

  if (recoverableRole.value) {
    items.push({
      title: "角色回滚窗口仍可用",
      detail: `如果刚刚切换角色后发现权限不符，可一键恢复到 ${recoverableRole.value.authorityName}。`,
    });
  }

  if (!runtime.value?.redisEnabled) {
    items.push({
      title: "检查 Redis 运行态",
      detail: "首页显示 Redis 未启用，建议进入运行态或系统工具页核对状态化链路。",
      routeName: "runtimeState",
    });
  }

  if (permissionHotspots.value[0]) {
    items.push({
      title: `优先核对 ${permissionHotspots.value[0].name} 权限段`,
      detail: `当前最热点权限段共 ${permissionHotspots.value[0].count} 条接口，可进入对应管理页检查是否与主角色预期一致。`,
      routeName: permissionRouteSuggestion(permissionHotspots.value[0].name),
    });
  }

  if (alternateAuthorities.value.length) {
    items.push({
      title: "做一次多角色入口巡检",
      detail: `当前账号存在 ${alternateAuthorities.value.length} 个可切换角色，建议切换后验证默认路由和关键入口是否正确。`,
      routeName: "authorities",
    });
  }

  if (recentActivities.value.length) {
    items.push({
      title: "复盘最近管理动作",
      detail: `今日已记录 ${recentActivities.value.length} 条首页动作，可以对照结果做权限与运行态回归。`,
    });
  }

  return items.slice(0, 5);
});

const focusLead = computed(() => todayFocus.value[0]?.title ?? "暂无重点事项");

const quickEntryGroups = computed(() => [
  {
    title: "账号与权限",
    summary: "围绕用户、角色、菜单与个人入口的高频管理动作。",
    detail: `当前主角色为 ${currentRoleLabel.value}，适合先做账号面和权限面核对。`,
    items: [
      { name: "users", label: "用户管理" },
      { name: "authorities", label: "角色管理" },
      { name: "menus", label: "菜单管理" },
      { name: "profile", label: "个人资料" },
    ],
  },
  {
    title: "运行与治理",
    summary: "聚合运行态、系统状态、日志与工具台的入口。",
    detail: runtime.value
      ? `当前服务运行于 ${runtime.value.os}，数据库底座为 ${runtime.value.dbBackend}。`
      : "建议先刷新运行态，再进入系统工具和运行态详情页。",
    items: [
      { name: "runtimeState", label: "运行态" },
      { name: "systemTools", label: "系统工具" },
      { name: "systemState", label: "系统状态" },
      { name: "systemConfig", label: "系统配置" },
    ],
  },
  {
    title: "审计与处理",
    summary: "把操作日志、登录日志和错误收敛到同一组，便于巡检。",
    detail: recentActivities.value[0]?.result.includes("失败")
      ? "检测到最近动作失败，建议优先查看日志与错误处置页面。"
      : "适合在日常巡检时统一检查登录、操作和错误信号。",
    items: [
      { name: "operationLogs", label: "操作日志" },
      { name: "loginLogs", label: "登录日志" },
      { name: "sysErrors", label: "错误日志" },
      { name: "versionInfo", label: "版本发布" },
    ],
  },
  {
    title: "示例与恢复",
    summary: "首页直达上传、断点续传与扫码上传等恢复型能力。",
    detail: "适合在演示链路、上传链路和恢复场景回归时快速进入。",
    items: [
      { name: "uploadExample", label: "上传示例" },
      { name: "breakpointExample", label: "断点续传" },
      { name: "scanUpload", label: "扫码上传" },
      { name: "dashboard", label: "回到首页" },
    ],
  },
]);

const lastRefresh = computed(() => recentActivities.value[0]?.time ?? "本次会话尚未刷新");

watch(
  () => auth.userInfo?.authorityId,
  (value) => {
    if (value) {
      selectedAuthorityId.value = value;
    }
  },
  { immediate: true },
);

function readLocal<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function persistActivities() {
  localStorage.setItem(ACTIVITY_KEY, JSON.stringify(recentActivities.value));
}

function persistRoleHistory() {
  localStorage.setItem(ROLE_HISTORY_KEY, JSON.stringify(roleHistory.value));
}

function nowLabel() {
  return new Date().toLocaleString("zh-CN", { hour12: false });
}

function rememberActivity(label: string, result: string) {
  recentActivities.value = [
    {
      id: crypto.randomUUID(),
      time: nowLabel(),
      label,
      result,
    },
    ...recentActivities.value,
  ].slice(0, 12);
  persistActivities();
}

function rememberRole(authority: AuthorityInfo | undefined | null, note: string) {
  if (!authority) return;
  roleHistory.value = [
    {
      id: crypto.randomUUID(),
      authorityId: authority.authorityId,
      authorityName: authority.authorityName,
      changedAt: nowLabel(),
      note,
    },
    ...roleHistory.value.filter((item) => item.authorityId !== authority.authorityId),
  ].slice(0, 5);
  persistRoleHistory();
}

function clearActivities() {
  recentActivities.value = [];
  persistActivities();
  feedback.value = "已清空首页最近动作记录。";
}

function permissionRouteSuggestion(segment: string) {
  if (segment.includes("sys/user")) return "users";
  if (segment.includes("authority")) return "authorities";
  if (segment.includes("menu")) return "menus";
  if (segment.includes("operation")) return "operationLogs";
  if (segment.includes("login")) return "loginLogs";
  if (segment.includes("error")) return "sysErrors";
  if (segment.includes("system")) return "systemTools";
  return "dashboard";
}

async function reload() {
  error.value = "";
  feedback.value = "";
  loading.value = true;
  try {
    await auth.bootstrap();
    runtime.value = await getRuntimeInfoApi();
    feedback.value = "已同步当前会话、权限包与运行底座。";
    rememberActivity("刷新鉴权与运行态", "成功");
  } catch (err) {
    error.value = err instanceof Error ? err.message : "刷新失败";
    feedback.value = "刷新未完成，请检查登录态或服务可用性。";
    rememberActivity("刷新鉴权与运行态", "失败");
  } finally {
    loading.value = false;
  }
}

async function switchAuthority() {
  if (selectedAuthorityId.value === auth.userInfo?.authorityId) {
    return;
  }
  error.value = "";
  feedback.value = "";
  switching.value = true;
  const previous = auth.userInfo?.authority;
  try {
    await auth.switchAuthority(selectedAuthorityId.value);
    runtime.value = await getRuntimeInfoApi();
    rememberRole(previous, "作为最近一次切换前角色，保留用于首页快速恢复。");
    rememberActivity(`切换角色到 ${selectedAuthorityId.value}`, "成功");
    feedback.value = `已切换到 ${currentRoleLabel.value}，并刷新权限包。若结果不符，可立即恢复上一个角色。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "角色切换失败";
    feedback.value = "角色切换未完成，已保留当前上下文。";
    rememberActivity(`切换角色到 ${selectedAuthorityId.value}`, "失败");
  } finally {
    switching.value = false;
  }
}

async function restorePreviousAuthority() {
  if (!recoverableRole.value) {
    return;
  }
  const target = recoverableRole.value;
  error.value = "";
  feedback.value = "";
  switching.value = true;
  const current = auth.userInfo?.authority;
  try {
    await auth.switchAuthority(target.authorityId);
    runtime.value = await getRuntimeInfoApi();
    roleHistory.value = roleHistory.value.filter((item) => item.id !== target.id);
    if (current && current.authorityId !== target.authorityId) {
      rememberRole(current, "从首页执行角色恢复后，保留当前角色供再次切回。");
    } else {
      persistRoleHistory();
    }
    rememberActivity(`恢复角色到 ${target.authorityId}`, "成功");
    feedback.value = `已恢复到 ${target.authorityName}，权限包已重新拉取。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "恢复角色失败";
    feedback.value = "角色恢复未完成，请稍后重试。";
    rememberActivity(`恢复角色到 ${target.authorityId}`, "失败");
  } finally {
    switching.value = false;
  }
}

async function openDefaultRoute() {
  const defaultName = auth.defaultRouterName;
  if (defaultName && router.hasRoute(defaultName)) {
    await router.push({ name: defaultName });
    rememberActivity(`进入默认入口 ${defaultName}`, "成功");
    return;
  }
  error.value = `默认入口 ${defaultName} 不存在`;
  rememberActivity(`进入默认入口 ${defaultName}`, "失败");
}

async function jumpTo(name: string) {
  if (!router.hasRoute(name)) {
    error.value = `路由 ${name} 不存在`;
    rememberActivity(`跳转 ${name}`, "失败");
    return;
  }
  await router.push({ name });
  rememberActivity(`跳转 ${name}`, "成功");
}

onMounted(() => {
  void reload();
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
