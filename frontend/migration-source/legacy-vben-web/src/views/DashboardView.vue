<template>
  <div class="stack dashboard-page">
    <div class="card hero-card">
      <div class="row space-between wrap gap-12">
        <div>
          <h1 class="title">控制台总控工作台</h1>
          <p class="subtitle">
            汇总当前登录态、角色权限、运行底座与最近动作，作为首页巡检、切换和分发的统一入口。
          </p>
        </div>
        <div class="row wrap gap-8">
          <button class="btn ghost" :disabled="loading" @click="reload">
            {{ loading ? "刷新中..." : "刷新鉴权与运行态" }}
          </button>
          <button class="btn ghost" @click="openDefaultRoute">进入当前默认入口</button>
          <button
            class="btn ghost"
            :disabled="switching || !recoverableRole"
            @click="restorePreviousAuthority"
          >
            恢复上一个角色
          </button>
        </div>
      </div>

      <div class="tag-list">
        <span class="tag">登录态：{{ auth.isLoggedIn ? "已接入" : "未登录" }}</span>
        <span class="tag">主角色：{{ currentRoleLabel }}</span>
        <span class="tag">备选角色：{{ alternateAuthorities.length }}</span>
        <span class="tag">权限覆盖：{{ coverageLabel }}</span>
        <span class="tag">运行态：{{ runtimeBadge }}</span>
        <span class="tag">最近刷新：{{ lastRefresh }}</span>
      </div>

      <div class="muted-grid hero-stats">
        <div class="stat-card">
          <h4>当前用户</h4>
          <p class="stat-value">{{ auth.userInfo?.userName ?? "-" }}</p>
          <p class="subtitle">{{ auth.userInfo?.nickName || "未设置昵称" }}</p>
        </div>
        <div class="stat-card">
          <h4>权限路径</h4>
          <p class="stat-value">{{ auth.policyPaths.length }}</p>
          <p class="subtitle">{{ methodSummary }}</p>
        </div>
        <div class="stat-card">
          <h4>健康摘要</h4>
          <p class="stat-value">{{ healthScore }}%</p>
          <p class="subtitle">{{ healthSummary }}</p>
        </div>
        <div class="stat-card">
          <h4>今日关注</h4>
          <p class="stat-value">{{ todayFocus.length }}</p>
          <p class="subtitle">{{ focusLead }}</p>
        </div>
      </div>

      <p v-if="feedback" class="subtitle status-line">{{ feedback }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="split-grid dashboard-layout">
      <div class="card">
        <div class="row space-between wrap gap-12">
          <div>
            <h3 class="title">可恢复角色切换</h3>
            <p class="subtitle">支持在首页完成角色试切换，并保留最近一次可恢复角色。</p>
          </div>
          <span class="tag" :class="recoverableRole ? 'warn' : 'good'">
            {{ recoverableRole ? "存在可恢复角色" : "当前无待恢复角色" }}
          </span>
        </div>

        <div class="field">
          <label>切换到角色（AuthorityId）</label>
          <select v-model.number="selectedAuthorityId" :disabled="switching || !auth.userInfo?.authorities?.length">
            <option
              v-for="authority in auth.userInfo?.authorities ?? []"
              :key="authority.authorityId"
              :value="authority.authorityId"
            >
              {{ authority.authorityName }} ({{ authority.authorityId }})
            </option>
          </select>
        </div>

        <div class="row wrap gap-8">
          <button
            class="btn primary"
            :disabled="switching || selectedAuthorityId === auth.userInfo?.authorityId"
            @click="switchAuthority"
          >
            {{ switching ? "切换中..." : "切换角色并刷新权限包" }}
          </button>
          <button class="btn ghost" :disabled="switching || !recoverableRole" @click="restorePreviousAuthority">
            回滚到上一个角色
          </button>
        </div>

        <div class="data-table compact-table top-gap">
          <table>
            <tbody>
              <tr>
                <td>当前主角色</td>
                <td>{{ currentRoleLabel }}</td>
              </tr>
              <tr>
                <td>默认入口</td>
                <td>{{ auth.userInfo?.authority?.defaultRouter ?? "dashboard" }}</td>
              </tr>
              <tr>
                <td>可恢复角色</td>
                <td>{{ recoverableRole ? `${recoverableRole.authorityName} (${recoverableRole.authorityId})` : "暂无" }}</td>
              </tr>
              <tr>
                <td>最近切换记录</td>
                <td>{{ roleHistory.length ? `${roleHistory.length} 条` : "暂无" }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-if="roleHistory.length" class="signal-list top-gap">
          <div v-for="item in roleHistory.slice(0, 3)" :key="item.id" class="signal-item compact-item">
            <div class="row space-between wrap gap-12">
              <strong>{{ item.authorityName }} ({{ item.authorityId }})</strong>
              <span class="subtitle">{{ item.changedAt }}</span>
            </div>
            <p>保留原因：{{ item.note }}</p>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="row space-between wrap gap-12">
          <div>
            <h3 class="title">健康摘要 / 风险提醒</h3>
            <p class="subtitle">基于现有鉴权、运行态和最近动作的首页研判，不依赖额外接口。</p>
          </div>
          <span class="tag" :class="healthScore >= 85 ? 'good' : healthScore >= 60 ? 'warn' : 'bad'">
            {{ healthSummary }}
          </span>
        </div>

        <div class="data-table compact-table">
          <table>
            <tbody>
              <tr><td>服务环境</td><td>{{ runtime?.os ?? "未加载" }}</td></tr>
              <tr><td>数据库底座</td><td>{{ runtime?.dbBackend ?? "未加载" }}</td></tr>
              <tr><td>Redis 状态</td><td>{{ runtime?.redisEnabled ? "已接入" : "未启用" }}</td></tr>
              <tr><td>CPU 核数</td><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><td>服务版本</td><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><td>权限分组</td><td>{{ permissionHotspots.length }} 组</td></tr>
            </tbody>
          </table>
        </div>

        <div class="signal-list top-gap">
          <div v-for="item in attentionItems" :key="item.title" class="signal-item compact-item">
            <div class="row space-between wrap gap-12">
              <strong>{{ item.title }}</strong>
              <span class="tag" :class="item.level">{{ item.levelLabel }}</span>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap gap-12">
        <div>
          <h3 class="title">快捷入口分组</h3>
          <p class="subtitle">结合当前权限热点与常用管理动作，把入口分组放到首页快速触达。</p>
        </div>
        <span class="tag">推荐入口 {{ quickEntryGroups.flatMap((group) => group.items).length }} 个</span>
      </div>

      <div class="quick-group-grid">
        <div v-for="group in quickEntryGroups" :key="group.title" class="entry-card group-card">
          <div class="row space-between wrap gap-12">
            <div>
              <h4>{{ group.title }}</h4>
              <p class="entry-summary">{{ group.summary }}</p>
            </div>
            <span class="tag">{{ group.items.length }} 项</span>
          </div>
          <div class="action-grid top-gap">
            <button
              v-for="item in group.items"
              :key="item.name"
              class="btn ghost"
              :disabled="!router.hasRoute(item.name)"
              @click="jumpTo(item.name)"
            >
              {{ item.label }}
            </button>
          </div>
          <p class="subtitle top-gap">{{ group.detail }}</p>
        </div>
      </div>
    </div>

    <div class="split-grid dashboard-layout">
      <div class="card">
        <h3 class="title">权限热点</h3>
        <p class="subtitle">对当前主角色权限路径做本地聚类，帮助判断管理范围与敏感面。</p>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>分组</th>
                <th>接口数</th>
                <th>方法</th>
                <th>热点路径</th>
                <th>关注级别</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in permissionHotspots" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.count }}</td>
                <td>{{ item.methods }}</td>
                <td>{{ item.sample }}</td>
                <td>{{ item.levelLabel }}</td>
              </tr>
              <tr v-if="!permissionHotspots.length">
                <td colspan="5">当前暂无权限热点数据</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">今日关注面板</h3>
        <p class="subtitle">把今日最值得处理的事项集中显示，减少首页切页搜索成本。</p>
        <div class="signal-list">
          <div v-for="item in todayFocus" :key="item.title" class="signal-item">
            <div class="row space-between wrap gap-12">
              <strong>{{ item.title }}</strong>
              <button v-if="item.routeName" class="btn ghost" @click="jumpTo(item.routeName)">打开</button>
            </div>
            <p>{{ item.detail }}</p>
          </div>
          <div v-if="!todayFocus.length" class="signal-item compact-item">
            <strong>当前无高优先事项</strong>
            <p>登录态、角色与运行底座都比较稳定，可以按常规巡检节奏处理。</p>
          </div>
        </div>
      </div>
    </div>

    <div class="split-grid dashboard-layout">
      <div class="card">
        <h3 class="title">运行摘要</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>操作系统</td><td>{{ runtime?.os ?? "-" }}</td></tr>
              <tr><td>CPU 核数</td><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><td>服务版本</td><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><td>数据库后端</td><td>{{ runtime?.dbBackend ?? "-" }}</td></tr>
              <tr><td>Redis 启用</td><td>{{ runtime?.redisEnabled ? "是" : "否" }}</td></tr>
              <tr><td>默认路由</td><td>{{ auth.userInfo?.authority?.defaultRouter ?? "dashboard" }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">权限路径预览</h3>
        <p class="subtitle">当前主角色可访问的核心接口路径，用于快速核对权限包是否刷新到位。</p>
        <pre class="code-block">{{ policiesText || "暂无权限路径" }}</pre>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap gap-12">
        <div>
          <h3 class="title">最近动作</h3>
          <p class="subtitle">记录首页发起的刷新、切换、恢复与跳转动作，便于复盘当天操作。</p>
        </div>
        <button class="btn ghost" :disabled="!recentActivities.length" @click="clearActivities">清空记录</button>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>时间</th>
              <th>动作</th>
              <th>结果</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in recentActivities" :key="item.id">
              <td>{{ item.time }}</td>
              <td>{{ item.label }}</td>
              <td>{{ item.result }}</td>
            </tr>
            <tr v-if="!recentActivities.length">
              <td colspan="3">暂无最近动作</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";
import { getRuntimeInfoApi } from "../api/admin";
import type { AuthorityInfo, RuntimeInfo } from "../types";

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
