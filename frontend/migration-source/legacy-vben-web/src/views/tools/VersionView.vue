<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">版本发布工作台</h3>
          <p class="subtitle">结合运行时、配置、MCP 状态、工具资产与发布记录，形成当前版本的交付健康度和发布建议。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="reload">刷新</button>
          <button class="btn ghost" :disabled="!releases.length" @click="copyLatestRelease">复制最近发布摘要</button>
          <button class="btn primary" :disabled="loading" @click="appendRelease">记录当前批次</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>运行版本</h4>
        <p class="stat-value">{{ runtime?.rustVersion || "-" }}</p>
        <p class="subtitle">{{ runtime?.os || "-" }}</p>
      </div>
      <div class="stat-card">
        <h4>发布健康度</h4>
        <p class="stat-value">{{ releaseHealthScore }}%</p>
        <p class="subtitle">{{ releaseHealthLabel }}</p>
      </div>
      <div class="stat-card">
        <h4>最近批次</h4>
        <p class="stat-value">{{ releases.length }}</p>
        <p class="subtitle">{{ latestReleaseLabel }}</p>
      </div>
      <div class="stat-card">
        <h4>待关注项</h4>
        <p class="stat-value">{{ releaseWarnings.length }}</p>
        <p class="subtitle">{{ releaseWarnings[0] || "当前未发现显著阻塞" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">环境快照</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>Rust Server</td><td>{{ runtime?.rustVersion ?? "-" }}</td></tr>
              <tr><td>操作系统</td><td>{{ runtime?.os ?? "-" }}</td></tr>
              <tr><td>CPU</td><td>{{ runtime?.cpuCores ?? "-" }}</td></tr>
              <tr><td>数据库</td><td>{{ runtime?.dbBackend ?? "-" }}</td></tr>
              <tr><td>Redis</td><td>{{ runtime?.redisEnabled ? "enabled" : "disabled" }}</td></tr>
              <tr><td>Bind</td><td>{{ config?.bindAddress ?? "-" }}</td></tr>
              <tr><td>Database URL</td><td>{{ config?.databaseUrl ?? "-" }}</td></tr>
              <tr><td>Redis URL</td><td>{{ config?.redisUrl || "-" }}</td></tr>
              <tr><td>多点登录</td><td>{{ config?.multipointEnabled ? "enabled" : "disabled" }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">能力摘要</h3>
        <div class="tag-list">
          <span class="tag">自动代码 {{ counts.autoCode }}</span>
          <span class="tag">技能 {{ counts.skills }}</span>
          <span class="tag">插件安装 {{ counts.plugins }}</span>
          <span class="tag">MCP 工具 {{ counts.mcpTools }}</span>
          <span class="tag">API Token {{ counts.tokens }}</span>
        </div>
        <div class="data-table top-gap">
          <table>
            <tbody>
              <tr><td>最近批次</td><td>{{ releases[0]?.note || "暂无发布记录" }}</td></tr>
              <tr><td>最近时间</td><td>{{ releases[0] ? formatTime(releases[0].createdAt) : "-" }}</td></tr>
              <tr><td>MCP 健康</td><td>{{ mcpStatus?.reachable ? "reachable" : "stopped" }}</td></tr>
              <tr><td>说明</td><td>{{ mcpStatus?.message || "-" }}</td></tr>
              <tr><td>发布建议</td><td>{{ releaseAdvice }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">发布风险与建议</h3>
        <div class="signal-list">
          <div v-for="item in releaseChecks" :key="item.title" class="signal-item">
            <div class="row space-between wrap">
              <strong>{{ item.title }}</strong>
              <span class="tag" :class="item.state">{{ item.label }}</span>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="title">最近版本对比</h3>
        <div v-if="releaseComparison" class="data-table">
          <table>
            <tbody>
              <tr><td>最近批次</td><td>{{ releaseComparison.latest.note }}</td></tr>
              <tr><td>上一批次</td><td>{{ releaseComparison.previous?.note || "暂无上一批次" }}</td></tr>
              <tr><td>时间间隔</td><td>{{ releaseComparison.diffLabel }}</td></tr>
              <tr><td>趋势判断</td><td>{{ releaseComparison.trend }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">暂无可对比的发布记录。</p>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap">
        <h3 class="title">发布记录</h3>
        <input v-model.trim="releaseQuery" placeholder="按说明筛选发布记录" />
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>时间</th>
              <th>说明</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in filteredReleases" :key="item.ID">
              <td>{{ formatTime(item.createdAt) }}</td>
              <td>{{ item.note }}</td>
              <td>
                <button class="btn ghost" @click="copyRelease(item)">复制摘要</button>
              </td>
            </tr>
            <tr v-if="!filteredReleases.length">
              <td colspan="3">暂无匹配的发布记录</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  getApiTokenListApi,
  getAutoCodeRegistryApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getRuntimeInfoApi,
  getSkillListApi,
  getSystemConfigApi,
  saveReleaseApi,
} from "../../api/admin";
import type { McpServiceStatus, ReleaseRecord, RuntimeInfo, SystemConfigInfo } from "../../types";

const runtime = ref<RuntimeInfo | null>(null);
const config = ref<SystemConfigInfo | null>(null);
const mcpStatus = ref<McpServiceStatus | null>(null);
const releases = ref<ReleaseRecord[]>([]);
const loading = ref(false);
const message = ref("");
const error = ref("");
const releaseQuery = ref("");
const counts = reactive({
  autoCode: 0,
  skills: 0,
  plugins: 0,
  mcpTools: 0,
  tokens: 0,
});

const filteredReleases = computed(() => {
  const keyword = releaseQuery.value.trim().toLowerCase();
  return releases.value.filter((item) => !keyword || item.note.toLowerCase().includes(keyword));
});

const releaseHealthScore = computed(() => {
  let score = 100;
  if (!runtime.value) score -= 25;
  if (runtime.value && !runtime.value.redisEnabled) score -= 15;
  if (!mcpStatus.value?.reachable) score -= 20;
  if (!counts.mcpTools) score -= 10;
  if (!counts.skills) score -= 10;
  if (!releases.value.length) score -= 10;
  return Math.max(0, score);
});

const releaseHealthLabel = computed(() => {
  if (releaseHealthScore.value >= 85) return "可发布";
  if (releaseHealthScore.value >= 60) return "需复核";
  return "建议先修正环境或工具链";
});

const latestReleaseLabel = computed(() => (releases.value[0] ? formatTime(releases.value[0].createdAt) : "暂无发布"));

const releaseWarnings = computed(() => {
  const warnings: string[] = [];
  if (!runtime.value?.redisEnabled) warnings.push("Redis 未启用，会影响状态化能力验收。");
  if (!mcpStatus.value?.reachable) warnings.push("MCP 服务不可达，建议先恢复工具联调链路。");
  if (!counts.mcpTools) warnings.push("当前没有 MCP 工具定义。");
  if (!counts.skills) warnings.push("当前没有技能资产。");
  if (!releases.value.length) warnings.push("当前还没有发布记录。");
  return warnings;
});

const releaseAdvice = computed(() => {
  if (releaseWarnings.value.length) return releaseWarnings.value[0];
  return "当前环境与工具链基本就绪，可继续记录并验收当前批次。";
});

const releaseChecks = computed(() => [
  {
    title: "运行底座",
    label: runtime.value ? "已接入" : "待加载",
    state: runtime.value ? "good" : "warn",
    detail: runtime.value ? `当前运行于 ${runtime.value.os} / ${runtime.value.dbBackend}` : "尚未拿到运行态信息。",
  },
  {
    title: "Redis 会话能力",
    label: runtime.value?.redisEnabled ? "正常" : "待修正",
    state: runtime.value?.redisEnabled ? "good" : "warn",
    detail: runtime.value?.redisEnabled ? "Redis 已启用，可继续验证会话/状态化流程。" : "Redis 未启用，建议优先排查。", 
  },
  {
    title: "MCP 联调链路",
    label: mcpStatus.value?.reachable ? "可达" : "不可达",
    state: mcpStatus.value?.reachable ? "good" : "warn",
    detail: mcpStatus.value?.message || "尚未获取 MCP 状态。",
  },
  {
    title: "发布记录",
    label: releases.value.length ? "已登记" : "待登记",
    state: releases.value.length ? "good" : "warn",
    detail: releases.value.length ? `最近一条：${releases.value[0].note}` : "建议至少记录当前恢复批次。", 
  },
]);

const releaseComparison = computed(() => {
  const latest = releases.value[0];
  if (!latest) return null;
  const previous = releases.value[1];
  if (!previous) {
    return {
      latest,
      previous: null,
      diffLabel: "暂无上一批次",
      trend: "当前为首条发布记录",
    };
  }
  const diffMs = Math.max(0, latest.createdAt - previous.createdAt);
  const diffHours = Math.round(diffMs / (1000 * 60 * 60));
  return {
    latest,
    previous,
    diffLabel: diffHours > 24 ? `${Math.round(diffHours / 24)} 天` : `${diffHours} 小时`,
    trend: latest.note === previous.note ? "最近两次说明相同，建议补充批次差异。" : "最近两次发布说明存在差异，可继续细化验收记录。",
  };
});

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [
      runtimeInfo,
      configInfo,
      mcpInfo,
      releaseInfo,
      autoCodeInfo,
      skillInfo,
      pluginInfo,
      mcpTools,
      tokens,
    ] = await Promise.all([
      getRuntimeInfoApi(),
      getSystemConfigApi(),
      getMcpStatusApi(),
      getReleaseListApi(),
      getAutoCodeRegistryApi(),
      getSkillListApi(),
      getPluginInstallListApi(),
      getMcpToolListApi(),
      getApiTokenListApi(),
    ]);
    runtime.value = runtimeInfo;
    config.value = configInfo;
    mcpStatus.value = mcpInfo;
    releases.value = releaseInfo.List.sort((a, b) => b.createdAt - a.createdAt);
    counts.autoCode = autoCodeInfo.Total;
    counts.skills = skillInfo.length;
    counts.plugins = pluginInfo.Total;
    counts.mcpTools = mcpTools.length;
    counts.tokens = tokens.Total;
    message.value = "已刷新版本工作台。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取版本信息失败";
  } finally {
    loading.value = false;
  }
}

async function appendRelease() {
  await saveReleaseApi({
    note: `版本页记录批次：${new Date().toLocaleString("zh-CN")}`,
  });
  message.value = "已记录当前发布批次";
  await reload();
}

async function copyLatestRelease() {
  if (!releases.value.length) return;
  await copyRelease(releases.value[0]);
}

async function copyRelease(item: ReleaseRecord) {
  const text = [`时间：${formatTime(item.createdAt)}`, `说明：${item.note}`].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = "已复制发布摘要。";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制发布摘要。";
  }
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void reload();
});
</script>
