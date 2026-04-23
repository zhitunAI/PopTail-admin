<template>
  <div class="stack">
    <div class="card">
      <div class="header-row">
        <div>
          <h3 class="title">错误处置工作台</h3>
          <p class="subtitle">
            基于现有错误日志进行分级筛查、待办追踪与详情回溯，帮助快速完成排障闭环。
          </p>
        </div>
        <div class="row wrap gap-sm">
          <button class="btn ghost" :disabled="loading" @click="loadLogs(false)">
            {{ loading ? "刷新中..." : "刷新日志" }}
          </button>
          <button class="btn primary" :disabled="!activeRow" @click="copyActiveSummary">
            复制当前错误
          </button>
        </div>
      </div>

      <div class="overview-grid">
        <div class="metric-card">
          <span class="metric-label">待处理</span>
          <strong>{{ boardSummary.pending }}</strong>
          <small>建议优先建立处置人</small>
        </div>
        <div class="metric-card">
          <span class="metric-label">处理中</span>
          <strong>{{ boardSummary.processing }}</strong>
          <small>需要持续跟踪验证</small>
        </div>
        <div class="metric-card">
          <span class="metric-label">已修复</span>
          <strong>{{ boardSummary.resolved }}</strong>
          <small>确认复测与回归范围</small>
        </div>
        <div class="metric-card danger">
          <span class="metric-label">高优先级</span>
          <strong>{{ boardSummary.critical }}</strong>
          <small>核心链路或服务异常</small>
        </div>
      </div>

      <div class="toolbar-grid advanced">
        <div class="field">
          <label>路径关键词</label>
          <input v-model.trim="filters.path" placeholder="如 /api、/admin、upload" />
        </div>
        <div class="field">
          <label>错误关键词</label>
          <input v-model.trim="filters.keyword" placeholder="如 timeout、panic、permission" />
        </div>
        <div class="field">
          <label>处理状态</label>
          <select v-model="filters.status">
            <option value="all">全部状态</option>
            <option value="pending">待处理</option>
            <option value="processing">处理中</option>
            <option value="resolved">已修复</option>
          </select>
        </div>
        <div class="field">
          <label>优先级</label>
          <select v-model="filters.severity">
            <option value="all">全部级别</option>
            <option value="critical">高</option>
            <option value="major">中</option>
            <option value="minor">低</option>
          </select>
        </div>
      </div>

      <div class="row wrap gap-sm">
        <button class="btn primary" @click="applyFilters">查询</button>
        <button class="btn ghost" @click="resetFilters">重置</button>
        <button class="btn ghost" @click="focusFirstPending">定位待处理首条</button>
      </div>

      <div class="chip-row">
        <button
          v-for="bucket in statusBuckets"
          :key="bucket.key"
          class="chip"
          :class="{ active: boardStatus === bucket.key }"
          @click="boardStatus = bucket.key"
        >
          {{ bucket.label }} · {{ bucket.count }}
        </button>
      </div>

      <div class="board-layout">
        <div class="board-list">
          <div class="list-header">
            <span>日志列表</span>
            <small>共 {{ filteredRows.length }} 条</small>
          </div>
          <div v-if="errorMessage" class="state-banner error">{{ errorMessage }}</div>
          <div v-else-if="!filteredRows.length" class="state-banner">当前筛选条件下暂无日志。</div>
          <div v-else class="log-list">
            <button
              v-for="item in filteredRows"
              :key="item.ID"
              class="log-item"
              :class="{
                active: activeRow?.ID === item.ID,
                critical: deriveSeverity(item) === 'critical',
              }"
              @click="inspect(item)"
            >
              <div class="log-item-top">
                <span class="log-id">#{{ item.ID }}</span>
                <span class="badge" :class="statusClass(normalizeStatus(item.status))">
                  {{ statusLabel(normalizeStatus(item.status)) }}
                </span>
                <span class="badge outline" :class="deriveSeverity(item)">
                  {{ severityLabel(deriveSeverity(item)) }}
                </span>
              </div>
              <strong class="log-path">{{ item.path || '未记录路径' }}</strong>
              <p class="log-error">{{ summarizeError(item.error) }}</p>
            </button>
          </div>
        </div>

        <div class="board-detail card detail-card">
          <template v-if="activeEntry">
            <div class="detail-header">
              <div>
                <h4>错误详情 #{{ activeEntry.ID }}</h4>
                <p>{{ activeEntry.path || '未记录路径' }}</p>
              </div>
              <div class="row wrap gap-sm">
                <span class="badge" :class="statusClass(activeEntry.reviewStatus)">
                  {{ statusLabel(activeEntry.reviewStatus) }}
                </span>
                <span class="badge outline" :class="activeEntry.severity">
                  {{ severityLabel(activeEntry.severity) }}
                </span>
              </div>
            </div>

            <div class="detail-grid">
              <div class="detail-section">
                <h5>错误原文</h5>
                <pre class="code-block">{{ activeEntry.error }}</pre>
                <div class="row wrap gap-sm">
                  <button class="btn ghost" @click="copyError(activeEntry)">复制原文</button>
                  <button class="btn ghost" @click="copyTraceback(activeEntry)">复制回溯摘要</button>
                </div>
              </div>

              <div class="detail-section">
                <h5>处理建议</h5>
                <ul class="bullet-list">
                  <li v-for="tip in buildSuggestions(activeEntry)" :key="tip">{{ tip }}</li>
                </ul>
              </div>

              <div class="detail-section">
                <h5>回溯视图</h5>
                <div class="timeline">
                  <div v-for="node in buildTimeline(activeEntry)" :key="node.label" class="timeline-node">
                    <strong>{{ node.label }}</strong>
                    <span>{{ node.value }}</span>
                  </div>
                </div>
              </div>

              <div class="detail-section">
                <h5>审阅结论</h5>
                <div class="review-grid">
                  <div>
                    <label>处理分类</label>
                    <select v-model="reviewDraft.status">
                      <option value="pending">待处理</option>
                      <option value="processing">处理中</option>
                      <option value="resolved">已修复</option>
                    </select>
                  </div>
                  <div>
                    <label>处理说明</label>
                    <textarea
                      v-model.trim="reviewDraft.note"
                      rows="5"
                      placeholder="记录复现条件、影响范围、责任归属或修复方案。"
                    />
                  </div>
                </div>
                <div class="row wrap gap-sm">
                  <button class="btn primary" @click="commitReview">更新审阅状态</button>
                  <button class="btn ghost" @click="loadReviewDraft(activeEntry)">恢复默认建议</button>
                </div>
                <p v-if="reviewMessage" class="hint success">{{ reviewMessage }}</p>
              </div>
            </div>
          </template>
          <div v-else class="empty-state">请选择左侧日志查看错误上下文、建议与审阅动作。</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getErrorLogsApi } from "../../api/admin";
import type { ErrorLogInfo } from "../../types";

type ReviewStatus = "pending" | "processing" | "resolved";
type SeverityLevel = "critical" | "major" | "minor";
type EnrichedErrorLog = ErrorLogInfo & {
  reviewStatus: ReviewStatus;
  severity: SeverityLevel;
};

type TimelineNode = {
  label: string;
  value: string;
};

const loading = ref(false);
const errorMessage = ref("");
const reviewMessage = ref("");
const sourceRows = ref<EnrichedErrorLog[]>([]);
const rows = ref<EnrichedErrorLog[]>([]);
const activeRow = ref<EnrichedErrorLog | null>(null);
const boardStatus = ref<"all" | ReviewStatus>("all");
const filters = reactive({
  path: "",
  keyword: "",
  status: "all" as "all" | ReviewStatus,
  severity: "all" as "all" | SeverityLevel,
});
const reviewDraft = reactive({
  status: "pending" as ReviewStatus,
  note: "",
});

const boardSummary = computed(() => ({
  pending: sourceRows.value.filter((item) => item.reviewStatus === "pending").length,
  processing: sourceRows.value.filter((item) => item.reviewStatus === "processing").length,
  resolved: sourceRows.value.filter((item) => item.reviewStatus === "resolved").length,
  critical: sourceRows.value.filter((item) => item.severity === "critical").length,
}));

const statusBuckets = computed<Array<{ count: number; key: "all" | ReviewStatus; label: string }>>(() => {
  const all = sourceRows.value.length;
  const pending = sourceRows.value.filter((item) => item.reviewStatus === "pending").length;
  const processing = sourceRows.value.filter((item) => item.reviewStatus === "processing").length;
  const resolved = sourceRows.value.filter((item) => item.reviewStatus === "resolved").length;
  return [
    { key: "all", label: "全部", count: all },
    { key: "pending", label: "待处理", count: pending },
    { key: "processing", label: "处理中", count: processing },
    { key: "resolved", label: "已修复", count: resolved },
  ];
});

const filteredRows = computed(() => {
  return rows.value.filter((item) => {
    if (boardStatus.value !== "all" && item.reviewStatus !== boardStatus.value) {
      return false;
    }
    return true;
  });
});

const activeEntry = computed(() => {
  if (!activeRow.value) {
    return null;
  }
  return rows.value.find((item) => item.ID === activeRow.value?.ID) ?? activeRow.value;
});

async function loadLogs(showMessage = false) {
  loading.value = true;
  errorMessage.value = "";
  reviewMessage.value = showMessage ? "日志已刷新。" : "";
  try {
    const result = await getErrorLogsApi();
    const nextRows = result.List.map(enrichRow);
    sourceRows.value = nextRows;
    rows.value = nextRows;
    if (activeRow.value) {
      activeRow.value = nextRows.find((item) => item.ID === activeRow.value?.ID) ?? nextRows[0] ?? null;
    } else {
      activeRow.value = nextRows[0] ?? null;
    }
    if (activeRow.value) {
      loadReviewDraft(activeRow.value);
    }
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : "获取错误日志失败";
  } finally {
    loading.value = false;
  }
}

function enrichRow(item: ErrorLogInfo): EnrichedErrorLog {
  const reviewStatus = normalizeStatus(item.status);
  return {
    ...item,
    reviewStatus,
    severity: deriveSeverity(item),
  };
}

function normalizeStatus(status: string): ReviewStatus {
  const value = status.trim().toLowerCase();
  if (["已修复", "resolved", "done", "success", "closed"].some((entry) => value.includes(entry))) {
    return "resolved";
  }
  if (["处理中", "processing", "repairing", "fixing", "review"].some((entry) => value.includes(entry))) {
    return "processing";
  }
  return "pending";
}

function deriveSeverity(item: Pick<ErrorLogInfo, "error" | "path" | "status">): SeverityLevel {
  const text = `${item.error} ${item.path} ${item.status}`.toLowerCase();
  if (["panic", "fatal", "timeout", "500", "refused", "denied", "crash"].some((entry) => text.includes(entry))) {
    return "critical";
  }
  if (["warning", "invalid", "missing", "forbidden", "401", "403", "404"].some((entry) => text.includes(entry))) {
    return "major";
  }
  return "minor";
}

function severityLabel(level: SeverityLevel) {
  return level === "critical" ? "高" : level === "major" ? "中" : "低";
}

function statusLabel(status: ReviewStatus) {
  return status === "pending" ? "待处理" : status === "processing" ? "处理中" : "已修复";
}

function statusClass(status: ReviewStatus) {
  return {
    pending: status === "pending",
    processing: status === "processing",
    resolved: status === "resolved",
  };
}

function summarizeError(errorText: string) {
  return errorText.length > 100 ? `${errorText.slice(0, 100)}...` : errorText;
}

function applyFilters() {
  reviewMessage.value = "";
  rows.value = sourceRows.value.filter((item) => {
    const matchesPath = !filters.path || item.path.toLowerCase().includes(filters.path.toLowerCase());
    const matchesKeyword = !filters.keyword || item.error.toLowerCase().includes(filters.keyword.toLowerCase());
    const matchesStatus = filters.status === "all" || item.reviewStatus === filters.status;
    const matchesSeverity = filters.severity === "all" || item.severity === filters.severity;
    return matchesPath && matchesKeyword && matchesStatus && matchesSeverity;
  });
  if (!rows.value.some((item) => item.ID === activeRow.value?.ID)) {
    activeRow.value = rows.value[0] ?? null;
  }
  if (activeRow.value) {
    loadReviewDraft(activeRow.value);
  }
}

function resetFilters() {
  filters.path = "";
  filters.keyword = "";
  filters.status = "all";
  filters.severity = "all";
  boardStatus.value = "all";
  rows.value = sourceRows.value;
  activeRow.value = rows.value[0] ?? null;
  if (activeRow.value) {
    loadReviewDraft(activeRow.value);
  }
  reviewMessage.value = "已恢复默认筛选。";
}

function inspect(item: EnrichedErrorLog) {
  activeRow.value = item;
  loadReviewDraft(item);
  reviewMessage.value = "";
}

function focusFirstPending() {
  boardStatus.value = "pending";
  const firstPending = filteredRows.value[0] ?? rows.value.find((item) => item.reviewStatus === "pending") ?? null;
  if (firstPending) {
    inspect(firstPending);
    reviewMessage.value = `已定位到待处理日志 #${firstPending.ID}。`;
    return;
  }
  reviewMessage.value = "当前没有待处理日志。";
}

function buildSuggestions(item: EnrichedErrorLog) {
  const suggestions = new Set<string>();
  suggestions.add(`先确认 ${item.path || "该入口"} 是否稳定复现，并记录调用参数与用户上下文。`);
  if (item.severity === "critical") {
    suggestions.add("该错误属于高优先级，建议先核查服务可用性、数据库/缓存连接及核心依赖状态。");
  }
  if (item.path.toLowerCase().includes("upload")) {
    suggestions.add("上传链路错误需补查文件大小、格式校验与断点状态同步是否一致。");
  }
  if (item.path.toLowerCase().includes("auth") || item.error.toLowerCase().includes("token")) {
    suggestions.add("鉴权相关错误请核查 token 生命周期、权限包刷新与多端会话策略。");
  }
  if (item.error.toLowerCase().includes("timeout")) {
    suggestions.add("超时类错误建议补充慢查询/外部依赖耗时观察，并确认是否需要重试或降级。");
  }
  suggestions.add("修复后需在相同入口进行回归，并补登记影响范围与验证结果。");
  return [...suggestions];
}

function buildTimeline(item: EnrichedErrorLog): TimelineNode[] {
  const fragments = item.error
    .split(/\n+/)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .slice(0, 3);

  return [
    { label: "入口路径", value: item.path || "未记录" },
    { label: "处置阶段", value: statusLabel(item.reviewStatus) },
    { label: "优先级", value: severityLabel(item.severity) },
    {
      label: "异常摘要",
      value: fragments[0] ?? summarizeError(item.error),
    },
    {
      label: "回溯线索",
      value: fragments[1] ?? "建议补充调用参数、请求体与关联用户信息。",
    },
    {
      label: "下一步",
      value: buildSuggestions(item)[0],
    },
  ];
}

function loadReviewDraft(item: EnrichedErrorLog) {
  reviewDraft.status = item.reviewStatus;
  reviewDraft.note = [
    `入口：${item.path || "未记录"}`,
    `级别：${severityLabel(item.severity)}`,
    `建议：${buildSuggestions(item)[0]}`,
  ].join("\n");
}

function commitReview() {
  if (!activeEntry.value) {
    return;
  }
  const nextStatus = reviewDraft.status;
  sourceRows.value = sourceRows.value.map((item) => {
    if (item.ID !== activeEntry.value?.ID) {
      return item;
    }
    return {
      ...item,
      status: statusLabel(nextStatus),
      reviewStatus: nextStatus,
    };
  });
  rows.value = rows.value.map((item) => {
    if (item.ID !== activeEntry.value?.ID) {
      return item;
    }
    return {
      ...item,
      status: statusLabel(nextStatus),
      reviewStatus: nextStatus,
    };
  });
  activeRow.value = rows.value.find((item) => item.ID === activeEntry.value?.ID) ?? activeRow.value;
  reviewMessage.value = reviewDraft.note
    ? `已将日志 #${activeEntry.value.ID} 标记为${statusLabel(nextStatus)}，并生成审阅说明。`
    : `已将日志 #${activeEntry.value.ID} 标记为${statusLabel(nextStatus)}。`;
}

async function copyText(content: string, successMessage: string) {
  if (!content) {
    reviewMessage.value = "没有可复制的内容。";
    return;
  }
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(content);
      reviewMessage.value = successMessage;
      return;
    }
  } catch {
    // ignore and fallback below
  }
  const textarea = document.createElement("textarea");
  textarea.value = content;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "absolute";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand("copy");
  document.body.removeChild(textarea);
  reviewMessage.value = successMessage;
}

async function copyError(item: EnrichedErrorLog) {
  await copyText(item.error, `已复制日志 #${item.ID} 的错误原文。`);
}

async function copyTraceback(item: EnrichedErrorLog) {
  const content = buildTimeline(item)
    .map((entry) => `${entry.label}: ${entry.value}`)
    .join("\n");
  await copyText(content, `已复制日志 #${item.ID} 的回溯摘要。`);
}

async function copyActiveSummary() {
  if (!activeEntry.value) {
    reviewMessage.value = "请先选择一条错误日志。";
    return;
  }
  const content = [
    `错误编号: #${activeEntry.value.ID}`,
    `路径: ${activeEntry.value.path || "未记录"}`,
    `状态: ${statusLabel(activeEntry.value.reviewStatus)}`,
    `优先级: ${severityLabel(activeEntry.value.severity)}`,
    `错误: ${activeEntry.value.error}`,
    `审阅建议: ${reviewDraft.note || buildSuggestions(activeEntry.value)[0]}`,
  ].join("\n");
  await copyText(content, `已复制日志 #${activeEntry.value.ID} 的处置摘要。`);
}

onMounted(() => {
  void loadLogs();
});
</script>

<style scoped>
.stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: #fff;
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 10px 30px rgba(15, 23, 42, 0.08);
}

.header-row,
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.wrap {
  flex-wrap: wrap;
}

.gap-sm {
  gap: 8px;
}

.title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
}

.subtitle {
  margin: 8px 0 0;
  color: #64748b;
}

.overview-grid,
.toolbar-grid,
.detail-grid,
.review-grid {
  display: grid;
  gap: 12px;
}

.overview-grid {
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  margin: 16px 0;
}

.metric-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  background: linear-gradient(180deg, #f8fafc, #fff);
}

.metric-card strong {
  font-size: 28px;
}

.metric-card small,
.metric-label {
  color: #64748b;
}

.metric-card.danger {
  border-color: #fecaca;
  background: linear-gradient(180deg, #fff1f2, #fff);
}

.toolbar-grid.advanced {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label,
.detail-section h5,
.review-grid label {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

.field input,
.field select,
.review-grid select,
.review-grid textarea {
  width: 100%;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  padding: 10px 12px;
  font: inherit;
  box-sizing: border-box;
}

.review-grid {
  grid-template-columns: minmax(160px, 220px) minmax(0, 1fr);
  align-items: start;
}

.review-grid textarea {
  resize: vertical;
  min-height: 120px;
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 10px 14px;
  font: inherit;
  cursor: pointer;
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
  opacity: 0.6;
  cursor: not-allowed;
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 14px 0 12px;
}

.chip {
  border: 1px solid #cbd5e1;
  background: #fff;
  color: #334155;
  border-radius: 999px;
  padding: 8px 12px;
  cursor: pointer;
}

.chip.active {
  background: #0f172a;
  border-color: #0f172a;
  color: #fff;
}

.board-layout {
  display: grid;
  grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
  gap: 16px;
  margin-top: 10px;
}

.board-list {
  border: 1px solid #e2e8f0;
  border-radius: 16px;
  padding: 12px;
  background: #f8fafc;
}

.list-header,
.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.list-header small,
.detail-header p,
.hint {
  color: #64748b;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 720px;
  overflow: auto;
}

.log-item {
  width: 100%;
  text-align: left;
  border: 1px solid #dbeafe;
  background: #fff;
  border-radius: 14px;
  padding: 14px;
  cursor: pointer;
}

.log-item.active {
  border-color: #2563eb;
  box-shadow: inset 0 0 0 1px #2563eb;
}

.log-item.critical {
  background: linear-gradient(180deg, #fff7ed, #fff);
}

.log-item-top {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.log-id,
.log-error {
  color: #475569;
}

.log-path {
  display: block;
  margin: 10px 0 6px;
  color: #0f172a;
}

.badge {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 600;
}

.badge.pending {
  background: #fef3c7;
  color: #92400e;
}

.badge.processing {
  background: #dbeafe;
  color: #1d4ed8;
}

.badge.resolved {
  background: #dcfce7;
  color: #166534;
}

.badge.outline {
  background: transparent;
  border: 1px solid currentColor;
}

.badge.critical {
  color: #c2410c;
}

.badge.major {
  color: #7c3aed;
}

.badge.minor {
  color: #475569;
}

.detail-card {
  padding: 18px;
}

.detail-grid {
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
}

.detail-section {
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 14px;
  background: #fff;
}

.code-block {
  margin: 0 0 12px;
  padding: 12px;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 12px;
  white-space: pre-wrap;
  word-break: break-word;
}

.bullet-list,
.timeline {
  margin: 0;
  padding-left: 18px;
  color: #334155;
}

.timeline {
  padding-left: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.timeline-node {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-left: 14px;
  border-left: 3px solid #cbd5e1;
}

.state-banner,
.empty-state {
  padding: 16px;
  border-radius: 12px;
  background: #f8fafc;
  color: #475569;
}

.state-banner.error {
  background: #fef2f2;
  color: #b91c1c;
}

.hint.success {
  margin: 10px 0 0;
  color: #166534;
}

@media (max-width: 960px) {
  .board-layout,
  .review-grid {
    grid-template-columns: 1fr;
  }
}
</style>
