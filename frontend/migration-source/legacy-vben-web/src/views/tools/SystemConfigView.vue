<template>
  <div class="workspace">
    <section class="hero">
      <div>
        <p class="eyebrow">配置工作台</p>
        <h3>系统配置</h3>
        <p class="subtitle">
          基于当前系统配置接口维护运行约束，支持差异审阅、本地草稿、风险提示与保存回填。
        </p>
      </div>
      <div class="hero-actions">
        <button class="btn primary" :disabled="loading || saving || !canSave" @click="save">
          {{ saving ? "保存中..." : "保存配置" }}
        </button>
        <button class="btn ghost" :disabled="loading || saving" @click="load(true)">
          {{ loading ? "刷新中..." : "重新加载" }}
        </button>
        <button class="btn ghost" :disabled="saving || !hasDraft" @click="applyDraft">
          回填本地草稿
        </button>
      </div>
    </section>

    <section class="summary-grid">
      <article class="summary-card">
        <span class="summary-label">当前变更项</span>
        <strong>{{ changedEntries.length }}</strong>
        <small>{{ changedEntries.length ? "已识别配置差异" : "当前与已加载配置一致" }}</small>
      </article>
      <article class="summary-card">
        <span class="summary-label">风险提示数</span>
        <strong>{{ riskItems.length }}</strong>
        <small>{{ riskHeadline }}</small>
      </article>
      <article class="summary-card">
        <span class="summary-label">本地草稿</span>
        <strong>{{ hasDraft ? "可回填" : "未暂存" }}</strong>
        <small>{{ draftLabel }}</small>
      </article>
      <article class="summary-card">
        <span class="summary-label">保存反馈</span>
        <strong>{{ saveStateLabel }}</strong>
        <small>{{ feedbackMessage }}</small>
      </article>
    </section>

    <section class="content-grid">
      <div class="panel">
        <div class="panel-header">
          <div>
            <h4>配置维护</h4>
            <p>维护运行地址、数据库、Redis 与兼容行为。</p>
          </div>
          <span class="badge" :class="dirty ? 'warn' : 'ok'">
            {{ dirty ? "存在未保存改动" : "已与当前配置同步" }}
          </span>
        </div>

        <div class="form-grid">
          <label class="field">
            <span>Bind Address</span>
            <input v-model.trim="form.bindAddress" placeholder="如 0.0.0.0:8888" />
          </label>
          <label class="field">
            <span>Database URL</span>
            <input v-model.trim="form.databaseUrl" placeholder="如 postgres://user:pass@db:5432/app" />
          </label>
          <label class="field">
            <span>Redis URL</span>
            <input v-model.trim="form.redisUrl" placeholder="如 redis://redis:6379/0" />
          </label>
          <label class="field">
            <span>Multipoint Enabled</span>
            <select v-model="form.multipointEnabled">
              <option :value="true">true</option>
              <option :value="false">false</option>
            </select>
          </label>
          <label class="field">
            <span>Compatibility Headers</span>
            <select v-model="form.compatibilityRefreshHeaders">
              <option :value="true">true</option>
              <option :value="false">false</option>
            </select>
          </label>
        </div>

        <div class="action-row">
          <button class="btn ghost" :disabled="saving || !dirty" @click="restoreLoadedConfig">
            恢复已加载配置
          </button>
          <button class="btn ghost" :disabled="saving" @click="saveDraft">
            暂存本地草稿
          </button>
          <button class="btn ghost" :disabled="saving || !hasDraft" @click="clearDraft">
            清空草稿
          </button>
        </div>

        <p v-if="validationMessage" class="hint warning">{{ validationMessage }}</p>
        <p v-if="statusMessage" class="hint">{{ statusMessage }}</p>
      </div>

      <div class="panel">
        <div class="panel-header">
          <div>
            <h4>配置摘要</h4>
            <p>根据当前表单实时生成运行摘要与建议动作。</p>
          </div>
        </div>

        <div class="summary-list">
          <div class="summary-item">
            <span>运行地址</span>
            <strong>{{ runtimeSummary.bindMode }}</strong>
          </div>
          <div class="summary-item">
            <span>数据库</span>
            <strong>{{ runtimeSummary.databaseMode }}</strong>
          </div>
          <div class="summary-item">
            <span>Redis</span>
            <strong>{{ runtimeSummary.redisMode }}</strong>
          </div>
          <div class="summary-item">
            <span>会话策略</span>
            <strong>{{ runtimeSummary.sessionMode }}</strong>
          </div>
          <div class="summary-item">
            <span>兼容头策略</span>
            <strong>{{ runtimeSummary.compatMode }}</strong>
          </div>
        </div>

        <div class="callout">
          <strong>建议动作</strong>
          <ul>
            <li v-for="item in recommendedActions" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </section>

    <section class="content-grid">
      <div class="panel">
        <div class="panel-header">
          <div>
            <h4>变更前后对比</h4>
            <p>保存前先确认关键配置差异，避免误改运行依赖。</p>
          </div>
          <span class="badge neutral">{{ changedEntries.length }} 项差异</span>
        </div>

        <div v-if="changedEntries.length" class="diff-list">
          <article v-for="entry in changedEntries" :key="entry.key" class="diff-item">
            <header>
              <strong>{{ entry.label }}</strong>
              <span class="badge neutral">{{ entry.level }}</span>
            </header>
            <p><span class="muted">当前配置：</span>{{ entry.before }}</p>
            <p><span class="muted">待保存值：</span>{{ entry.after }}</p>
          </article>
        </div>
        <p v-else class="empty">当前没有待保存差异。</p>
      </div>

      <div class="panel">
        <div class="panel-header">
          <div>
            <h4>风险提示</h4>
            <p>本地根据配置组合给出保存前检查点。</p>
          </div>
          <span class="badge" :class="riskItems.length ? 'warn' : 'ok'">
            {{ riskItems.length ? "建议先复核" : "未发现明显风险" }}
          </span>
        </div>

        <div v-if="riskItems.length" class="risk-list">
          <article v-for="item in riskItems" :key="item.title" class="risk-item">
            <strong>{{ item.title }}</strong>
            <p>{{ item.detail }}</p>
          </article>
        </div>
        <div v-else class="callout success">
          <strong>配置组合平稳</strong>
          <p>当前表单字段完整，且未检测到明显冲突，可继续保存或暂存。</p>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getSystemConfigApi, updateSystemConfigApi } from "../../api/admin";
import type { SystemConfigInfo } from "../../types";

type ConfigFieldKey = keyof SystemConfigInfo;

interface DiffEntry {
  after: string;
  before: string;
  key: ConfigFieldKey;
  label: string;
  level: "低影响" | "中影响" | "高影响";
}

interface RiskItem {
  detail: string;
  title: string;
}

const draftStorageKey = "gva-system-config-draft";

const fieldLabels: Record<ConfigFieldKey, string> = {
  bindAddress: "Bind Address",
  compatibilityRefreshHeaders: "Compatibility Headers",
  databaseUrl: "Database URL",
  multipointEnabled: "Multipoint Enabled",
  redisUrl: "Redis URL",
};

const form = reactive<SystemConfigInfo>(createEmptyConfig());
const baseline = ref<SystemConfigInfo>(createEmptyConfig());
const statusMessage = ref("正在加载当前系统配置...");
const loading = ref(false);
const saving = ref(false);
const hasDraft = ref(false);
const draftUpdatedAt = ref("");
const lastLoadedAt = ref("");
const lastSavedAt = ref("");

function createEmptyConfig(): SystemConfigInfo {
  return {
    bindAddress: "",
    compatibilityRefreshHeaders: true,
    databaseUrl: "",
    multipointEnabled: false,
    redisUrl: "",
  };
}

function cloneConfig(config: SystemConfigInfo): SystemConfigInfo {
  return {
    bindAddress: config.bindAddress,
    compatibilityRefreshHeaders: config.compatibilityRefreshHeaders,
    databaseUrl: config.databaseUrl,
    multipointEnabled: config.multipointEnabled,
    redisUrl: config.redisUrl,
  };
}

function applyConfig(config: SystemConfigInfo) {
  form.bindAddress = config.bindAddress;
  form.databaseUrl = config.databaseUrl;
  form.redisUrl = config.redisUrl;
  form.multipointEnabled = config.multipointEnabled;
  form.compatibilityRefreshHeaders = config.compatibilityRefreshHeaders;
}

function stringifyValue(value: boolean | string) {
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return value || "未填写";
}

function updateDraftState() {
  if (typeof window === "undefined") {
    hasDraft.value = false;
    draftUpdatedAt.value = "";
    return;
  }
  const raw = window.localStorage.getItem(draftStorageKey);
  if (!raw) {
    hasDraft.value = false;
    draftUpdatedAt.value = "";
    return;
  }
  try {
    const parsed = JSON.parse(raw) as { config?: SystemConfigInfo; updatedAt?: string };
    hasDraft.value = Boolean(parsed.config);
    draftUpdatedAt.value = parsed.updatedAt || "";
  } catch {
    hasDraft.value = false;
    draftUpdatedAt.value = "";
  }
}

const dirty = computed(() =>
  (Object.keys(fieldLabels) as ConfigFieldKey[]).some((key) => form[key] !== baseline.value[key]),
);

const changedEntries = computed<DiffEntry[]>(() =>
  (Object.keys(fieldLabels) as ConfigFieldKey[])
    .filter((key) => form[key] !== baseline.value[key])
    .map((key) => ({
      after: stringifyValue(form[key]),
      before: stringifyValue(baseline.value[key]),
      key,
      label: fieldLabels[key],
      level:
        key === "databaseUrl" || key === "redisUrl"
          ? "高影响"
          : key === "bindAddress" || key === "multipointEnabled"
            ? "中影响"
            : "低影响",
    })),
);

const validationMessage = computed(() => {
  if (!form.bindAddress.trim()) {
    return "Bind Address 不能为空。";
  }
  if (!form.databaseUrl.trim()) {
    return "Database URL 不能为空。";
  }
  if (!form.redisUrl.trim()) {
    return "Redis URL 不能为空。";
  }
  if (!form.bindAddress.includes(":")) {
    return "Bind Address 建议包含端口，例如 0.0.0.0:8888。";
  }
  if (!/^postgres(ql)?:\/\//.test(form.databaseUrl.trim())) {
    return "Database URL 建议使用 PostgreSQL 连接串。";
  }
  if (!/^redis:\/\//.test(form.redisUrl.trim())) {
    return "Redis URL 建议使用 redis:// 开头。";
  }
  return "";
});

const runtimeSummary = computed(() => ({
  bindMode:
    form.bindAddress.startsWith("0.0.0.0") || form.bindAddress.startsWith(":")
      ? "对外监听"
      : "本地/定向监听",
  compatMode: form.compatibilityRefreshHeaders ? "兼容旧刷新头" : "仅保留新头行为",
  databaseMode: /localhost|127\.0\.0\.1/.test(form.databaseUrl) ? "本地数据库" : "容器/远端数据库",
  redisMode: /localhost|127\.0\.0\.1/.test(form.redisUrl) ? "本地 Redis" : "容器/远端 Redis",
  sessionMode: form.multipointEnabled ? "多点登录已开启" : "单会话优先",
}));

const riskItems = computed<RiskItem[]>(() => {
  const items: RiskItem[] = [];
  if (/localhost|127\.0\.0\.1/.test(form.databaseUrl) || /localhost|127\.0\.0\.1/.test(form.redisUrl)) {
    items.push({
      detail: "当前配置指向本地地址，若用于 Docker/桌面集成环境，需确认容器网络或宿主映射是否一致。",
      title: "数据库或 Redis 指向本机地址",
    });
  }
  if (!form.multipointEnabled && !form.compatibilityRefreshHeaders) {
    items.push({
      detail: "关闭多点登录且关闭兼容刷新头时，旧客户端刷新链路可能更容易暴露兼容问题。",
      title: "会话与刷新策略较严格",
    });
  }
  if (form.multipointEnabled && !form.redisUrl.trim()) {
    items.push({
      detail: "多点登录依赖 Redis 侧状态协同，请先确认 Redis 地址有效。",
      title: "多点登录缺少 Redis 支撑",
    });
  }
  if (changedEntries.value.some((entry) => entry.level === "高影响")) {
    items.push({
      detail: "数据库或 Redis 地址发生变化时，建议保存后立即复核连接状态与迁移环境。",
      title: "存在高影响基础设施变更",
    });
  }
  if (validationMessage.value) {
    items.push({
      detail: validationMessage.value,
      title: "当前表单仍有待修正项",
    });
  }
  return items;
});

const recommendedActions = computed(() => {
  const actions = ["保存前核对连接串中的主机、端口与库名是否与当前环境匹配。"];
  if (changedEntries.value.some((entry) => entry.key === "databaseUrl")) {
    actions.push("数据库地址已变更，建议保存后立即执行连接或迁移探针。");
  }
  if (changedEntries.value.some((entry) => entry.key === "redisUrl")) {
    actions.push("Redis 地址已变更，建议同步复核会话状态与运行态页面。");
  }
  if (changedEntries.value.some((entry) => entry.key === "multipointEnabled")) {
    actions.push("会话策略已调整，建议保存后重新登录验证多点登录行为。");
  }
  if (!changedEntries.value.length) {
    actions.push("当前没有待保存差异，可用本地草稿记录下一轮配置调整。");
  }
  return actions;
});

const saveStateLabel = computed(() => {
  if (saving.value) {
    return "保存中";
  }
  if (lastSavedAt.value) {
    return "已保存";
  }
  if (dirty.value) {
    return "待保存";
  }
  return "已同步";
});

const feedbackMessage = computed(() => {
  if (saving.value) {
    return "正在提交系统配置。";
  }
  if (lastSavedAt.value) {
    return `最近保存于 ${lastSavedAt.value}`;
  }
  if (lastLoadedAt.value) {
    return `最近加载于 ${lastLoadedAt.value}`;
  }
  return "尚未执行保存。";
});

const draftLabel = computed(() =>
  hasDraft.value ? `最近暂存于 ${draftUpdatedAt.value || "本地"} ` : "可将当前表单暂存到本地",
);

const riskHeadline = computed(() =>
  riskItems.value.length ? "保存前建议先处理高风险项" : "当前未检测到显著配置风险",
);

const canSave = computed(() => !validationMessage.value && dirty.value);

function formatNow() {
  return new Date().toLocaleString("zh-CN", { hour12: false });
}

async function load(showMessage = false) {
  loading.value = true;
  try {
    const config = await getSystemConfigApi();
    applyConfig(config);
    baseline.value = cloneConfig(config);
    lastLoadedAt.value = formatNow();
    statusMessage.value = showMessage ? "系统配置已重新加载。" : "已载入当前系统配置。";
  } catch (error) {
    statusMessage.value =
      error instanceof Error ? `加载系统配置失败：${error.message}` : "加载系统配置失败。";
  } finally {
    loading.value = false;
    updateDraftState();
  }
}

function restoreLoadedConfig() {
  applyConfig(baseline.value);
  statusMessage.value = "已恢复到最近一次加载的系统配置。";
}

function saveDraft() {
  if (typeof window === "undefined") {
    statusMessage.value = "当前环境不支持本地草稿。";
    return;
  }
  const updatedAt = formatNow();
  window.localStorage.setItem(
    draftStorageKey,
    JSON.stringify({
      config: cloneConfig(form),
      updatedAt,
    }),
  );
  updateDraftState();
  statusMessage.value = "当前配置已暂存到本地草稿。";
}

function applyDraft() {
  if (typeof window === "undefined") {
    statusMessage.value = "当前环境不支持本地草稿。";
    return;
  }
  const raw = window.localStorage.getItem(draftStorageKey);
  if (!raw) {
    hasDraft.value = false;
    draftUpdatedAt.value = "";
    statusMessage.value = "未找到可回填的本地草稿。";
    return;
  }
  try {
    const parsed = JSON.parse(raw) as { config?: SystemConfigInfo };
    if (!parsed.config) {
      statusMessage.value = "本地草稿内容无效，已忽略。";
      return;
    }
    applyConfig(parsed.config);
    statusMessage.value = "已回填本地草稿，可继续审阅差异后保存。";
    updateDraftState();
  } catch {
    statusMessage.value = "本地草稿解析失败，请重新暂存。";
  }
}

function clearDraft() {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(draftStorageKey);
  }
  updateDraftState();
  statusMessage.value = "本地草稿已清空。";
}

async function save() {
  if (!canSave.value) {
    statusMessage.value = validationMessage.value || "当前没有可保存的配置变更。";
    return;
  }
  saving.value = true;
  try {
    const saved = await updateSystemConfigApi(cloneConfig(form));
    applyConfig(saved);
    baseline.value = cloneConfig(saved);
    lastSavedAt.value = formatNow();
    statusMessage.value = "系统配置已保存并回填最新结果。";
  } catch (error) {
    statusMessage.value =
      error instanceof Error ? `保存系统配置失败：${error.message}` : "保存系统配置失败。";
  } finally {
    saving.value = false;
    updateDraftState();
  }
}

onMounted(() => {
  updateDraftState();
  void load();
});
</script>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero,
.panel,
.summary-card {
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  background: #fff;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.04);
}

.hero {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 22px;
}

.eyebrow {
  margin: 0 0 6px;
  color: #2563eb;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h3,
h4,
p {
  margin: 0;
}

.subtitle {
  margin-top: 8px;
  max-width: 760px;
  color: #64748b;
  line-height: 1.6;
}

.hero-actions,
.action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.summary-grid,
.content-grid,
.form-grid,
.summary-list,
.diff-list,
.risk-list {
  display: grid;
  gap: 12px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.summary-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
}

.summary-card strong {
  font-size: 24px;
  color: #0f172a;
}

.summary-card small,
.panel-header p,
.hint,
.empty,
.risk-item p,
.callout p,
.callout li {
  color: #64748b;
  line-height: 1.6;
}

.summary-label,
.field span,
.muted {
  color: #475569;
  font-size: 13px;
}

.content-grid {
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.panel {
  padding: 18px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.form-grid {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field input,
.field select {
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 14px;
}

.field input:focus,
.field select:focus {
  outline: none;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 14px;
  cursor: pointer;
  transition: 0.2s ease;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.btn.primary {
  background: #2563eb;
  color: #fff;
}

.btn.ghost {
  background: #eff6ff;
  color: #1d4ed8;
}

.badge {
  height: fit-content;
  border-radius: 999px;
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 600;
}

.badge.ok {
  background: #dcfce7;
  color: #166534;
}

.badge.warn {
  background: #fef3c7;
  color: #92400e;
}

.badge.neutral {
  background: #e2e8f0;
  color: #334155;
}

.summary-item,
.diff-item,
.risk-item,
.callout {
  border-radius: 12px;
  background: #f8fafc;
  padding: 12px 14px;
}

.summary-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.summary-item strong {
  color: #0f172a;
}

.callout {
  margin-top: 12px;
}

.callout strong,
.diff-item strong,
.risk-item strong {
  color: #0f172a;
}

.callout ul {
  margin: 8px 0 0;
  padding-left: 18px;
}

.callout.success {
  background: #f0fdf4;
}

.diff-item header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.hint {
  margin-top: 12px;
}

.hint.warning {
  color: #b45309;
}

.empty {
  padding: 16px 0;
}

@media (max-width: 900px) {
  .hero,
  .panel-header,
  .summary-item,
  .diff-item header {
    flex-direction: column;
  }
}
</style>
