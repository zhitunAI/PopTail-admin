<template>
  <div class="split-grid">
    <div class="card">
      <div class="section-head">
        <div>
          <h3 class="title">API Token 工作台</h3>
          <p class="subtitle">
            基于当前令牌台账接口完成签发、筛选、轮转建议与摘要复制，帮助运维与集成方统一管理服务访问凭证。
          </p>
        </div>
        <button class="btn ghost" :disabled="loading" @click="reloadTokens">
          {{ loading ? "刷新中..." : "刷新台账" }}
        </button>
      </div>

      <div class="summary-grid">
        <article class="metric-card">
          <span class="metric-label">令牌总数</span>
          <strong class="metric-value">{{ tokenMetrics.total }}</strong>
          <small class="metric-note">当前台账记录总量</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">活跃令牌</span>
          <strong class="metric-value">{{ tokenMetrics.active }}</strong>
          <small class="metric-note">状态含 active / issued / ready</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">高风险轮转</span>
          <strong class="metric-value">{{ tokenMetrics.rotationNeeded }}</strong>
          <small class="metric-note">长期有效或状态异常建议复核</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">当前筛选结果</span>
          <strong class="metric-value">{{ filteredTokens.length }}</strong>
          <small class="metric-note">{{ statusText }}</small>
        </article>
      </div>

      <div class="toolbar-grid">
        <div class="field">
          <label>令牌名称</label>
          <input v-model="draft.name" placeholder="例如 gateway-sync" />
        </div>
        <div class="field">
          <label>作用域</label>
          <input
            v-model="draft.scope"
            placeholder="例如 system.audit.read"
            @keydown.enter="issueToken"
          />
        </div>
        <div class="field">
          <label>有效期</label>
          <select v-model="draft.ttl">
            <option v-for="item in ttlOptions" :key="item" :value="item">{{ item }}</option>
          </select>
        </div>
      </div>

      <div class="preset-panel">
        <div class="preset-section">
          <div class="section-title-row">
            <strong>作用域预设</strong>
            <span class="hint">单击即可填充草稿</span>
          </div>
          <div class="tag-list">
            <button
              v-for="preset in scopePresets"
              :key="preset.key"
              class="tag action-tag"
              type="button"
              @click="applyScopePreset(preset)"
            >
              {{ preset.label }}
            </button>
          </div>
        </div>
        <div class="preset-section">
          <div class="section-title-row">
            <strong>签发建议</strong>
            <span class="hint">根据集成方式调整名称与有效期</span>
          </div>
          <div class="tag-list">
            <span v-for="item in capabilities" :key="item.name" class="tag">
              {{ item.name }}：{{ item.desc }}
            </span>
          </div>
        </div>
      </div>

      <div class="row wrap">
        <button class="btn primary" :disabled="submitting" @click="issueToken">
          {{ submitting ? "登记中..." : "登记令牌" }}
        </button>
        <button class="btn ghost" :disabled="submitting" @click="fillDraftFromRecommendation">
          填充轮转建议
        </button>
        <button class="btn ghost" :disabled="clearing || !tokens.length" @click="clearTokens">
          {{ clearing ? "清空中..." : "清空记录" }}
        </button>
      </div>

      <div class="status-banner">
        <strong>当前状态：</strong>{{ statusText }}
      </div>
    </div>

    <div class="card">
      <div class="section-head">
        <div>
          <h3 class="title">令牌台账</h3>
          <p class="subtitle">按名称、作用域、状态和有效期筛查令牌，并聚焦最近签发与待轮转记录。</p>
        </div>
      </div>

      <div class="toolbar-grid">
        <div class="field">
          <label>关键词</label>
          <input
            v-model="filters.keyword"
            placeholder="搜索名称或作用域"
            @keydown.enter="applyFilters"
          />
        </div>
        <div class="field">
          <label>状态</label>
          <select v-model="filters.status">
            <option value="all">全部状态</option>
            <option v-for="item in statusOptions" :key="item" :value="item">{{ item }}</option>
          </select>
        </div>
        <div class="field">
          <label>有效期</label>
          <select v-model="filters.ttl">
            <option value="all">全部有效期</option>
            <option v-for="item in ttlOptions" :key="item" :value="item">{{ item }}</option>
          </select>
        </div>
      </div>

      <div class="row wrap">
        <button class="btn ghost" @click="applyFilters">应用筛选</button>
        <button class="btn ghost" @click="resetFilters">重置筛选</button>
        <button
          class="btn ghost"
          :disabled="!selectedToken"
          @click="copyTokenDigest"
        >
          复制令牌摘要
        </button>
      </div>

      <div class="panel-grid">
        <section class="subcard">
          <div class="section-title-row">
            <strong>最近签发</strong>
            <span class="hint">按最新记录倒序展示</span>
          </div>
          <div v-if="recentTokens.length" class="stack-list">
            <button
              v-for="item in recentTokens"
              :key="item.ID"
              class="list-chip"
              type="button"
              @click="selectToken(item)"
            >
              <span>{{ item.name }}</span>
              <small>{{ item.scope }}</small>
            </button>
          </div>
          <p v-else class="empty-hint">暂无最近签发记录。</p>
        </section>

        <section class="subcard">
          <div class="section-title-row">
            <strong>轮转建议</strong>
            <span class="hint">优先复核长期令牌与异常状态</span>
          </div>
          <div v-if="rotationCandidates.length" class="stack-list">
            <button
              v-for="item in rotationCandidates"
              :key="item.ID"
              class="list-chip warning"
              type="button"
              @click="selectToken(item)"
            >
              <span>{{ item.name }}</span>
              <small>{{ getRotationReason(item) }}</small>
            </button>
          </div>
          <p v-else class="empty-hint">当前没有明显需要轮转的令牌。</p>
        </section>
      </div>

      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>名称</th>
              <th>作用域</th>
              <th>有效期</th>
              <th>状态</th>
              <th>建议</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in filteredTokens"
              :key="item.ID"
              :class="{ selected: selectedToken?.ID === item.ID }"
              @click="selectToken(item)"
            >
              <td>{{ item.name }}</td>
              <td>{{ item.scope }}</td>
              <td>{{ item.ttl }}</td>
              <td>
                <span class="status-pill" :class="statusTone(item.status)">
                  {{ item.status }}
                </span>
              </td>
              <td>{{ getRotationReason(item) }}</td>
            </tr>
            <tr v-if="!filteredTokens.length">
              <td colspan="5">暂无匹配的令牌记录，可调整筛选条件或先签发新令牌。</td>
            </tr>
          </tbody>
        </table>
      </div>

      <section class="subcard detail-card">
        <div class="section-title-row">
          <strong>详情预览</strong>
          <span class="hint">{{ selectedToken ? "已选中令牌" : "点击任意记录查看详情" }}</span>
        </div>
        <template v-if="selectedToken">
          <div class="detail-grid">
            <div>
              <span class="detail-label">令牌名称</span>
              <strong>{{ selectedToken.name }}</strong>
            </div>
            <div>
              <span class="detail-label">作用域</span>
              <strong>{{ selectedToken.scope }}</strong>
            </div>
            <div>
              <span class="detail-label">有效期</span>
              <strong>{{ selectedToken.ttl }}</strong>
            </div>
            <div>
              <span class="detail-label">状态</span>
              <strong>{{ selectedToken.status }}</strong>
            </div>
            <div>
              <span class="detail-label">轮转判断</span>
              <strong>{{ getRotationReason(selectedToken) }}</strong>
            </div>
            <div>
              <span class="detail-label">摘要</span>
              <strong>{{ buildDigest(selectedToken) }}</strong>
            </div>
          </div>
        </template>
        <p v-else class="empty-hint">请选择一条令牌记录查看详情、复制摘要或作为下一次签发参考。</p>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { clearApiTokenApi, getApiTokenListApi, issueApiTokenApi } from "../../api/admin";
import type { ApiTokenRecord } from "../../types";

type ScopePreset = {
  key: string;
  label: string;
  name: string;
  scope: string;
  ttl: string;
};

const capabilities = [
  { name: "短期访问", desc: "适合批量任务或临时集成，优先使用 24h / 7d 生命周期" },
  { name: "按域授权", desc: "建议按审计、用户、发布等单一职责拆分 scope" },
  { name: "轮转落账", desc: "签发后保留台账，异常状态或超长 TTL 需优先复核" },
];

const ttlOptions = ["24h", "7d", "30d"];
const scopePresets: ScopePreset[] = [
  {
    key: "audit-read",
    label: "审计只读",
    name: "audit-reader",
    scope: "system.audit.read",
    ttl: "7d",
  },
  {
    key: "publish-bot",
    label: "发布机器人",
    name: "release-bot",
    scope: "release.deploy",
    ttl: "24h",
  },
  {
    key: "plugin-sync",
    label: "插件同步",
    name: "plugin-sync",
    scope: "plugin.install.manage",
    ttl: "30d",
  },
  {
    key: "example-upload",
    label: "示例上传",
    name: "upload-agent",
    scope: "example.upload.write",
    ttl: "24h",
  },
];

const tokens = ref<ApiTokenRecord[]>([]);
const selectedToken = ref<ApiTokenRecord | null>(null);
const loading = ref(false);
const submitting = ref(false);
const clearing = ref(false);
const statusText = ref("等待加载令牌台账。");
const filters = reactive({
  keyword: "",
  status: "all",
  ttl: "all",
});
const draft = reactive({
  name: "gateway-sync",
  scope: "system.audit.read",
  ttl: "24h",
});

const statusOptions = computed(() =>
  Array.from(new Set(tokens.value.map((item) => item.status).filter(Boolean))),
);

const filteredTokens = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();
  return tokens.value.filter((item) => {
    if (
      keyword &&
      !`${item.name} ${item.scope}`.toLowerCase().includes(keyword)
    ) {
      return false;
    }
    if (filters.status !== "all" && item.status !== filters.status) {
      return false;
    }
    if (filters.ttl !== "all" && item.ttl !== filters.ttl) {
      return false;
    }
    return true;
  });
});

const recentTokens = computed(() =>
  [...tokens.value].sort((left, right) => right.ID - left.ID).slice(0, 4),
);

const rotationCandidates = computed(() =>
  [...tokens.value]
    .filter((item) => needsRotation(item))
    .sort((left, right) => right.ID - left.ID)
    .slice(0, 4),
);

const tokenMetrics = computed(() => ({
  total: tokens.value.length,
  active: tokens.value.filter((item) => isHealthyStatus(item.status)).length,
  rotationNeeded: tokens.value.filter((item) => needsRotation(item)).length,
}));

function isHealthyStatus(status: string) {
  return ["active", "issued", "ready"].includes(status.toLowerCase());
}

function needsRotation(item: ApiTokenRecord) {
  return item.ttl === "30d" || !isHealthyStatus(item.status);
}

function getRotationReason(item: ApiTokenRecord) {
  if (!isHealthyStatus(item.status)) {
    return "状态异常，建议尽快复核或重发";
  }
  if (item.ttl === "30d") {
    return "长期令牌，建议纳入轮转计划";
  }
  if (item.ttl === "7d") {
    return "中周期令牌，建议在本周内确认续用";
  }
  return "短周期令牌，可维持当前策略";
}

function statusTone(status: string) {
  if (isHealthyStatus(status)) {
    return "ok";
  }
  if (status.toLowerCase().includes("pending")) {
    return "pending";
  }
  return "risk";
}

function buildDigest(item: ApiTokenRecord) {
  return `${item.name} | ${item.scope} | ${item.ttl} | ${item.status}`;
}

function applyScopePreset(preset: ScopePreset) {
  draft.name = preset.name;
  draft.scope = preset.scope;
  draft.ttl = preset.ttl;
  statusText.value = `已应用作用域预设：${preset.label}。`;
}

function fillDraftFromRecommendation() {
  const candidate = rotationCandidates.value[0] ?? recentTokens.value[0];
  if (!candidate) {
    statusText.value = "当前没有可参考的令牌记录，请先新增一条令牌。";
    return;
  }
  draft.name = `${candidate.name}-next`;
  draft.scope = candidate.scope;
  draft.ttl = candidate.ttl === "30d" ? "7d" : "24h";
  statusText.value = `已根据 ${candidate.name} 生成轮转草稿，建议复核后再登记。`;
}

function selectToken(item: ApiTokenRecord) {
  selectedToken.value = item;
  statusText.value = `已定位令牌 ${item.name}，可复制摘要或按此补发新令牌。`;
}

function applyFilters() {
  statusText.value = `已应用筛选，命中 ${filteredTokens.value.length} 条令牌记录。`;
  if (
    selectedToken.value &&
    !filteredTokens.value.some((item) => item.ID === selectedToken.value?.ID)
  ) {
    selectedToken.value = filteredTokens.value[0] ?? null;
  }
}

function resetFilters() {
  filters.keyword = "";
  filters.status = "all";
  filters.ttl = "all";
  statusText.value = `已重置筛选，当前展示 ${tokens.value.length} 条令牌记录。`;
  selectedToken.value = tokens.value[0] ?? null;
}

async function copyTokenDigest() {
  if (!selectedToken.value) {
    statusText.value = "请先选择一条令牌记录。";
    return;
  }
  const text = buildDigest(selectedToken.value);
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      statusText.value = `已复制令牌摘要：${selectedToken.value.name}。`;
      return;
    }
  } catch {
    // fall through
  }
  if (typeof window !== "undefined") {
    window.prompt("当前环境不支持自动写入剪贴板，请手动复制：", text);
    statusText.value = `已提供手动复制窗口：${selectedToken.value.name}。`;
  }
}

async function load(showMessage = false) {
  loading.value = true;
  try {
    const result = await getApiTokenListApi();
    tokens.value = result.List;
    if (!selectedToken.value || !tokens.value.some((item) => item.ID === selectedToken.value?.ID)) {
      selectedToken.value = recentTokens.value[0] ?? tokens.value[0] ?? null;
    }
    if (showMessage) {
      statusText.value = `已刷新令牌台账，共 ${tokens.value.length} 条记录。`;
    } else if (tokens.value.length) {
      statusText.value = `令牌台账已加载，最近签发 ${recentTokens.value[0]?.name ?? "无"}。`;
    } else {
      statusText.value = "当前台账为空，可先签发一条服务令牌。";
    }
  } finally {
    loading.value = false;
  }
}

async function reloadTokens() {
  await load(true);
}

async function issueToken() {
  const name = draft.name.trim();
  const scope = draft.scope.trim();
  if (!name || !scope) {
    statusText.value = "令牌名称和作用域不能为空。";
    return;
  }
  submitting.value = true;
  try {
    const created = await issueApiTokenApi({
      name,
      scope,
      ttl: draft.ttl,
    });
    draft.name = `${name}-next`;
    selectedToken.value = created;
    statusText.value = `已登记令牌 ${created.name}，建议复制摘要并同步给调用方。`;
    await load(false);
  } finally {
    submitting.value = false;
  }
}

async function clearTokens() {
  clearing.value = true;
  try {
    await clearApiTokenApi();
    tokens.value = [];
    selectedToken.value = null;
    statusText.value = "已清空令牌台账记录。";
  } finally {
    clearing.value = false;
  }
}

onMounted(async () => {
  await load(false);
});
</script>
