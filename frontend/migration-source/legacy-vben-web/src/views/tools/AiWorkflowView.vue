<template>
  <div class="stack">
    <div class="card">
      <div class="header-row">
        <div>
          <h3 class="title">AI 审核工作台</h3>
          <p class="subtitle">
            基于现有 AI 审核提交接口，补足前置校验、请求预览、模板动作、最近提交与本地审计时间线，
            便于值班人员快速完成审核处置。
          </p>
        </div>
        <div class="row wrap gap-sm">
          <button class="btn ghost" @click="loadLatestRecord">回填最近提交</button>
          <button class="btn ghost" @click="resetForm">恢复默认</button>
        </div>
      </div>

      <div class="overview-grid">
        <div class="metric-card">
          <span class="metric-label">当前模式</span>
          <strong>{{ modeMeta.label }}</strong>
          <small>{{ modeMeta.summary }}</small>
        </div>
        <div class="metric-card">
          <span class="metric-label">执行人</span>
          <strong>{{ operatorLabel }}</strong>
          <small>{{ auth.userInfo?.authority?.authorityName || "未识别角色" }}</small>
        </div>
        <div class="metric-card" :class="{ danger: validationIssues.length > 0 }">
          <span class="metric-label">前置校验</span>
          <strong>{{ validationIssues.length ? `${validationIssues.length} 项待处理` : "已通过" }}</strong>
          <small>{{ validationIssues.length ? validationIssues[0] : "可以直接发起审核请求" }}</small>
        </div>
        <div class="metric-card">
          <span class="metric-label">最近提交</span>
          <strong>{{ recentRecords.length }}</strong>
          <small>本地保留最近 6 条操作审计</small>
        </div>
      </div>

      <div class="preset-grid">
        <button
          v-for="preset in presets"
          :key="preset.label"
          class="preset-card"
          @click="applyPreset(preset)"
        >
          <strong>{{ preset.label }}</strong>
          <span>{{ preset.action }} · {{ preset.mode }}</span>
          <small>{{ preset.reason }}</small>
        </button>
      </div>

      <div class="editor-layout">
        <div class="editor-column">
          <div class="toolbar-grid">
            <div class="field">
              <label>执行模式</label>
              <select v-model="form.mode">
                <option value="system">system</option>
                <option value="delegated">delegated</option>
              </select>
            </div>
            <div class="field">
              <label>目标标识</label>
              <input
                v-model.trim="form.articleId"
                placeholder="article-202 / user-42 / comment-18"
                @keydown.enter.prevent="submit"
              />
            </div>
            <div class="field">
              <label>处置动作</label>
              <select v-model="form.action">
                <option value="approve">approve</option>
                <option value="reject">reject</option>
                <option value="ban">ban</option>
                <option value="escalate">escalate</option>
              </select>
            </div>
            <div class="field">
              <label>处置原因</label>
              <input
                v-model.trim="form.reason"
                placeholder="例如 policy-safe / spam-hit / manual-escalation"
                @keydown.enter.prevent="submit"
              />
            </div>
          </div>

          <div class="field">
            <label>审核备注</label>
            <textarea
              v-model.trim="form.note"
              rows="4"
              placeholder="补充命中策略、复核结论、升级原因或交接说明。"
            />
          </div>

          <div v-if="form.mode === 'system'" class="toolbar-grid">
            <div class="field">
              <label>Service Token</label>
              <input
                v-model.trim="form.serviceToken"
                placeholder="svc-moderation-token"
                @keydown.enter.prevent="submit"
              />
            </div>
            <div class="field">
              <label>令牌摘要</label>
              <input :value="serviceTokenSummary" disabled />
            </div>
          </div>

          <div v-else class="toolbar-grid">
            <div class="field">
              <label>Operator User ID</label>
              <input v-model.number="form.operatorUserId" min="1" type="number" />
            </div>
            <div class="field">
              <label>授权 Scope</label>
              <input
                v-model.trim="form.scope"
                placeholder="例如 user.violation.handle"
                @keydown.enter.prevent="submit"
              />
            </div>
          </div>

          <div class="mode-panel" :class="form.mode">
            <h4>{{ modeMeta.label }} 模式说明</h4>
            <ul class="bullet-list">
              <li v-for="item in modeMeta.tips" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div v-if="validationIssues.length" class="state-banner error">
            <strong>提交前请先处理以下问题：</strong>
            <ul class="issue-list">
              <li v-for="issue in validationIssues" :key="issue">{{ issue }}</li>
            </ul>
          </div>
          <div v-else class="state-banner success">
            请求体完整，可直接提交审核工作流。
          </div>

          <div class="row wrap gap-sm">
            <button class="btn primary" :disabled="submitting || validationIssues.length > 0" @click="submit">
              {{ submitting ? "提交中..." : "提交审核请求" }}
            </button>
            <button class="btn ghost" @click="copyPreview">复制请求预览</button>
            <button class="btn ghost" @click="appendTimeline('manual', '人工记录一次本地审计检查。')">
              写入本地审计
            </button>
          </div>

          <p v-if="success" class="hint success">
            最近审计 ID：{{ success.auditId }} · {{ success.timeLabel }}
          </p>
          <p v-if="error" class="error">{{ error }}</p>
        </div>

        <div class="preview-column">
          <div class="card detail-card">
            <div class="list-header">
              <span>请求预览</span>
              <small>{{ previewSize }}</small>
            </div>
            <pre class="code-block">{{ requestPreview }}</pre>
          </div>

          <div class="card detail-card">
            <div class="list-header">
              <span>最近提交记录</span>
              <small>点击可回填</small>
            </div>
            <div v-if="!recentRecords.length" class="empty-state">
              暂无本地提交记录，提交后会自动生成审计快照。
            </div>
            <div v-else class="history-list">
              <button
                v-for="entry in recentRecords"
                :key="entry.auditId"
                class="history-item"
                :class="{ active: success?.auditId === entry.auditId }"
                @click="reuseRecord(entry)"
              >
                <div class="history-top">
                  <strong>{{ entry.action }}</strong>
                  <span class="badge" :class="entry.mode">{{ entry.mode }}</span>
                </div>
                <p>{{ entry.articleId }} · {{ entry.reason }}</p>
                <small>{{ entry.timeLabel }} · {{ entry.auditId }}</small>
              </button>
            </div>
          </div>

          <div class="card detail-card">
            <div class="list-header">
              <span>本地审计时间线</span>
              <small>{{ timeline.length }} 条</small>
            </div>
            <div v-if="!timeline.length" class="empty-state">暂无时间线。</div>
            <div v-else class="timeline">
              <div v-for="item in timeline" :key="item.id" class="timeline-node">
                <div class="timeline-head">
                  <strong>{{ item.title }}</strong>
                  <span class="badge outline">{{ item.kind }}</span>
                </div>
                <p>{{ item.detail }}</p>
                <small>{{ item.timeLabel }}</small>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { submitAiModerationDecision } from "../../api/admin";
import { useAuthStore } from "../../stores/auth";

type Mode = "system" | "delegated";
type WorkflowAction = "approve" | "reject" | "ban" | "escalate";

type PresetAction = {
  action: WorkflowAction;
  label: string;
  mode: Mode;
  reason: string;
  scope?: string;
};

type RecentRecord = {
  accepted: boolean;
  action: string;
  articleId: string;
  auditId: string;
  mode: Mode;
  operatorUserId?: number;
  reason: string;
  scope?: string;
  serviceToken?: string;
  submittedAt: string;
  timeLabel: string;
};

type TimelineEntry = {
  detail: string;
  id: string;
  kind: "draft" | "manual" | "preset" | "submit";
  timeLabel: string;
  title: string;
};

const RECENT_KEY = "gva.ai.workflow.recent";
const TIMELINE_KEY = "gva.ai.workflow.timeline";

const auth = useAuthStore();
const submitting = ref(false);
const error = ref("");
const recentRecords = ref<RecentRecord[]>([]);
const timeline = ref<TimelineEntry[]>([]);
const success = ref<{ auditId: string; timeLabel: string } | null>(null);

const form = reactive({
  mode: "system" as Mode,
  articleId: "article-100",
  action: "approve" as WorkflowAction,
  reason: "policy-safe",
  note: "已完成基础策略复核。",
  serviceToken: "svc-moderation-token",
  operatorUserId: auth.userInfo?.ID ?? 1,
  scope: "user.violation.handle",
});

const presets: PresetAction[] = [
  {
    label: "直接放行",
    mode: "system",
    action: "approve",
    reason: "policy-safe",
  },
  {
    label: "命中垃圾内容",
    mode: "delegated",
    action: "reject",
    reason: "spam-hit",
    scope: "content.moderation.reject",
  },
  {
    label: "违规封禁",
    mode: "delegated",
    action: "ban",
    reason: "severe-violation",
    scope: "user.violation.handle",
  },
  {
    label: "升级人工复核",
    mode: "system",
    action: "escalate",
    reason: "manual-escalation",
  },
];

const operatorLabel = computed(() => {
  const profile = auth.userInfo;
  if (!profile) {
    return "未登录用户";
  }
  return `${profile.nickName || profile.userName} (#${profile.ID})`;
});

const modeMeta = computed(() => {
  if (form.mode === "system") {
    return {
      label: "System",
      summary: "适合系统批量审核或服务侧自动裁决。",
      tips: [
        "需要填写 Service Token，适用于服务身份直接下发审核决定。",
        "system 模式适合稳定策略命中后的自动处置，不依赖人工授权范围。",
        "建议只在规则清晰、误判成本可控的场景下直接执行。",
      ],
    };
  }
  return {
    label: "Delegated",
    summary: "适合人工值班、复核升级与授权委托处置。",
    tips: [
      "需要明确 Operator User ID 与 Scope，便于记录委托来源。",
      "delegated 模式适合人工复核、升级封禁、例外授权等场景。",
      "Scope 应与职责边界一致，避免越权提交审核决定。",
    ],
  };
});

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.articleId.trim()) {
    issues.push("目标标识不能为空。");
  }
  if (!/^[\w-.:/]+$/.test(form.articleId.trim())) {
    issues.push("目标标识仅建议使用字母、数字、-、_、.、:、/。");
  }
  if (!form.reason.trim()) {
    issues.push("处置原因不能为空。");
  }
  if (form.reason.trim().length < 4) {
    issues.push("处置原因至少 4 个字符，便于后续审计检索。");
  }
  if (form.mode === "system") {
    if (!form.serviceToken.trim()) {
      issues.push("system 模式必须填写 Service Token。");
    }
    if (form.action === "ban") {
      issues.push("system 模式下直接 ban 风险较高，建议改用 delegated 执行。");
    }
  }
  if (form.mode === "delegated") {
    if (!Number.isFinite(form.operatorUserId) || form.operatorUserId <= 0) {
      issues.push("delegated 模式需要合法的 Operator User ID。");
    }
    if (!form.scope.trim()) {
      issues.push("delegated 模式必须填写 Scope。");
    }
  }
  if (form.action === "escalate" && !form.note.trim()) {
    issues.push("升级人工复核时请填写备注说明。");
  }
  return issues;
});

const payload = computed(() => ({
  mode: form.mode,
  articleId: form.articleId.trim(),
  action: form.action,
  reason: buildReasonText(),
  serviceToken: form.mode === "system" ? form.serviceToken.trim() : undefined,
  delegation:
    form.mode === "delegated"
      ? {
          operatorUserId: Math.trunc(form.operatorUserId || 0),
          scope: form.scope.trim(),
        }
      : undefined,
}));

const requestPreview = computed(() => JSON.stringify(payload.value, null, 2));
const previewSize = computed(() => `${requestPreview.value.length} chars`);
const serviceTokenSummary = computed(() => {
  const token = form.serviceToken.trim();
  if (!token) {
    return "未填写";
  }
  if (token.length <= 6) {
    return `${token} · 短令牌`;
  }
  return `${token.slice(0, 3)}***${token.slice(-3)} · ${token.length} 位`;
});

watch(
  () => [form.mode, form.articleId, form.action, form.reason, form.scope, form.operatorUserId, form.serviceToken, form.note],
  () => {
    persistTimeline();
  },
  { deep: false },
);

onMounted(() => {
  recentRecords.value = readLocal<RecentRecord[]>(RECENT_KEY, []);
  timeline.value = readLocal<TimelineEntry[]>(TIMELINE_KEY, []);
  appendTimeline("draft", "页面已加载，当前草稿已进入本地审计范围。");
});

function buildReasonText() {
  const note = form.note.trim();
  if (!note) {
    return form.reason.trim();
  }
  return `${form.reason.trim()} | note:${note}`;
}

function formatTime(iso = new Date().toISOString()) {
  return new Intl.DateTimeFormat("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date(iso));
}

function buildTimelineEntry(
  kind: TimelineEntry["kind"],
  title: string,
  detail: string,
): TimelineEntry {
  const now = new Date().toISOString();
  return {
    id: `${kind}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    kind,
    title,
    detail,
    timeLabel: formatTime(now),
  };
}

function appendTimeline(kind: TimelineEntry["kind"], detail: string) {
  timeline.value = [buildTimelineEntry(kind, timelineTitle(kind), detail), ...timeline.value].slice(0, 12);
  writeLocal(TIMELINE_KEY, timeline.value);
}

function timelineTitle(kind: TimelineEntry["kind"]) {
  switch (kind) {
    case "preset":
      return "模板动作";
    case "submit":
      return "提交结果";
    case "manual":
      return "人工记录";
    default:
      return "草稿状态";
  }
}

function applyPreset(preset: PresetAction) {
  form.mode = preset.mode;
  form.action = preset.action;
  form.reason = preset.reason;
  form.note =
    preset.action === "escalate"
      ? "需要人工复核，请二线值班确认策略边界。"
      : `已套用模板动作：${preset.label}。`;
  if (preset.scope) {
    form.scope = preset.scope;
  }
  if (preset.mode === "delegated") {
    form.operatorUserId = auth.userInfo?.ID ?? form.operatorUserId;
  }
  appendTimeline("preset", `已套用模板动作「${preset.label}」，模式 ${preset.mode}，动作 ${preset.action}。`);
}

async function copyPreview() {
  const text = requestPreview.value;
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      appendTimeline("manual", "已复制请求预览到剪贴板。");
      return;
    }
  } catch {
    // ignore and fallback
  }
  window.prompt("当前环境不支持直接写入剪贴板，请手动复制：", text);
  appendTimeline("manual", "剪贴板不可用，已使用手动复制提示。");
}

function reuseRecord(entry: RecentRecord) {
  form.mode = entry.mode;
  form.articleId = entry.articleId;
  form.action = (entry.action as WorkflowAction) || "approve";
  form.reason = entry.reason;
  form.note = `已回填历史审计 ${entry.auditId}。`;
  form.serviceToken = entry.serviceToken || form.serviceToken;
  form.operatorUserId = entry.operatorUserId || auth.userInfo?.ID || form.operatorUserId;
  form.scope = entry.scope || form.scope;
  success.value = { auditId: entry.auditId, timeLabel: entry.timeLabel };
  appendTimeline("manual", `已回填历史记录 ${entry.auditId}，目标 ${entry.articleId}。`);
}

function loadLatestRecord() {
  const latest = recentRecords.value[0];
  if (!latest) {
    appendTimeline("manual", "尝试回填最近提交，但当前没有本地记录。");
    return;
  }
  reuseRecord(latest);
}

function resetForm() {
  form.mode = "system";
  form.articleId = "article-100";
  form.action = "approve";
  form.reason = "policy-safe";
  form.note = "已完成基础策略复核。";
  form.serviceToken = "svc-moderation-token";
  form.operatorUserId = auth.userInfo?.ID ?? 1;
  form.scope = "user.violation.handle";
  error.value = "";
  success.value = null;
  appendTimeline("draft", "已恢复默认草稿。");
}

async function submit() {
  if (validationIssues.value.length > 0) {
    error.value = validationIssues.value[0] || "提交信息不完整";
    appendTimeline("manual", `提交被前置校验拦截：${validationIssues.value.join("；")}`);
    return;
  }

  submitting.value = true;
  error.value = "";
  success.value = null;
  appendTimeline("submit", `开始提交 ${form.articleId} 的审核请求，动作 ${form.action}。`);

  try {
    const result = await submitAiModerationDecision(payload.value);
    const submittedAt = new Date().toISOString();
    const record: RecentRecord = {
      accepted: result.accepted,
      action: form.action,
      articleId: form.articleId.trim(),
      auditId: result.auditId,
      mode: form.mode,
      operatorUserId: form.mode === "delegated" ? Math.trunc(form.operatorUserId || 0) : undefined,
      reason: form.reason.trim(),
      scope: form.mode === "delegated" ? form.scope.trim() : undefined,
      serviceToken: form.mode === "system" ? form.serviceToken.trim() : undefined,
      submittedAt,
      timeLabel: formatTime(submittedAt),
    };
    success.value = {
      auditId: result.auditId,
      timeLabel: record.timeLabel,
    };
    recentRecords.value = [record, ...recentRecords.value.filter((item) => item.auditId !== record.auditId)].slice(0, 6);
    writeLocal(RECENT_KEY, recentRecords.value);
    appendTimeline(
      "submit",
      `审核请求已提交成功，审计 ID ${result.auditId}，模式 ${form.mode}，accepted=${String(result.accepted)}。`,
    );
  } catch (err) {
    error.value = err instanceof Error ? err.message : "提交失败";
    appendTimeline("submit", `审核请求提交失败：${error.value}`);
  } finally {
    submitting.value = false;
  }
}

function persistTimeline() {
  writeLocal(TIMELINE_KEY, timeline.value);
}

function readLocal<T>(key: string, fallback: T): T {
  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function writeLocal<T>(key: string, value: T) {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // ignore storage failures
  }
}
</script>

<style scoped>
.stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.header-row,
.row,
.history-top,
.timeline-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
}

.wrap {
  flex-wrap: wrap;
}

.gap-sm {
  gap: 8px;
}

.title {
  margin: 0;
}

.subtitle {
  color: #64748b;
  margin: 6px 0 0;
}

.overview-grid,
.preset-grid,
.toolbar-grid,
.editor-layout {
  display: grid;
  gap: 16px;
}

.overview-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin: 16px 0;
}

.preset-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 16px;
}

.toolbar-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 16px;
}

.editor-layout {
  align-items: start;
  grid-template-columns: minmax(0, 1.45fr) minmax(320px, 1fr);
}

.editor-column,
.preview-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-card,
.preset-card,
.detail-card,
.mode-panel,
.history-item {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
}

.metric-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
}

.metric-card strong {
  font-size: 20px;
}

.metric-card.danger {
  background: #fff1f2;
  border-color: #fecdd3;
}

.metric-label {
  color: #64748b;
  font-size: 12px;
}

.preset-card,
.history-item {
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  text-align: left;
}

.preset-card:hover,
.history-item:hover,
.history-item.active {
  border-color: #94a3b8;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field input,
.field select,
.field textarea {
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  padding: 10px 12px;
}

.field input:disabled {
  background: #f1f5f9;
  color: #64748b;
}

.mode-panel {
  padding: 16px;
}

.mode-panel.system {
  background: #eff6ff;
  border-color: #bfdbfe;
}

.mode-panel.delegated {
  background: #f8fafc;
  border-color: #cbd5e1;
}

.bullet-list,
.issue-list {
  margin: 8px 0 0;
  padding-left: 18px;
}

.btn {
  border: none;
  border-radius: 10px;
  cursor: pointer;
  padding: 10px 14px;
}

.btn.primary {
  background: #2563eb;
  color: #fff;
}

.btn.ghost {
  background: #e2e8f0;
  color: #0f172a;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.state-banner,
.empty-state {
  background: #f8fafc;
  border: 1px dashed #cbd5e1;
  border-radius: 12px;
  padding: 14px;
}

.state-banner.success {
  background: #ecfdf5;
  border-color: #86efac;
}

.state-banner.error,
.error {
  color: #b91c1c;
}

.state-banner.error {
  background: #fef2f2;
  border-color: #fca5a5;
}

.hint.success {
  color: #166534;
}

.list-header {
  align-items: center;
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
}

.code-block {
  background: #0f172a;
  border-radius: 10px;
  color: #e2e8f0;
  font-size: 12px;
  margin: 0;
  overflow: auto;
  padding: 14px;
}

.history-list,
.timeline {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.badge {
  background: #dbeafe;
  border-radius: 999px;
  color: #1d4ed8;
  font-size: 12px;
  padding: 2px 8px;
}

.badge.system {
  background: #dbeafe;
  color: #1d4ed8;
}

.badge.delegated {
  background: #ede9fe;
  color: #6d28d9;
}

.badge.outline {
  background: transparent;
  border: 1px solid #cbd5e1;
  color: #475569;
}

.timeline-node {
  border-left: 3px solid #cbd5e1;
  padding-left: 12px;
}

@media (max-width: 960px) {
  .editor-layout {
    grid-template-columns: 1fr;
  }
}
</style>
