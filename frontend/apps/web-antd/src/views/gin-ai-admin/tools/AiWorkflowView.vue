<script setup lang="ts">
import type {
  AiWorkflowMessageRecord,
  AiWorkflowSessionListItem,
  AiWorkflowSessionUpsertInput,
} from "#/types/gin-ai-admin";

import { computed, onMounted, reactive, ref, watch } from "vue";

import { useAccess } from "@vben/access";
import { Page } from "@vben/common-ui";
import { useAccessStore } from "@vben/stores";

import {
  Alert,
  Button,
  Card,
  Collapse,
  CollapsePanel,
  Form,
  FormItem,
  Input,
  InputNumber,
  message,
  Select,
  SelectOption,
  Space,  TabPane,
  Tabs,
  Tag,
} from "ant-design-vue";

import {
  deleteAIWorkflowSessionApi,
  dumpAIWorkflowMarkdownApi,
  getAIWorkflowSessionDetailApi,
  getAIWorkflowSessionListApi,
  rollbackAIWorkflowNodeApi,
  resumeAIWorkflowChatApi,
  saveAIWorkflowSessionApi,
  submitAiModerationDecision,
  updateAIWorkflowStageApi,
} from "#/api/gin-ai-admin/admin";
import { useAuthStore } from "#/store/gin-ai-admin/auth";

type Mode = "delegated" | "system";
type WorkflowAction = "approve" | "ban" | "escalate" | "reject";
type AiWorkflowButtonAction =
  | "applyPreset"
  | "copy"
  | "export"
  | "recordAudit"
  | "reset"
  | "reuse"
  | "submit";

const AI_WORKFLOW_BUTTON_ACCESS_CODES: Record<AiWorkflowButtonAction, string[]> = {
  applyPreset: [
    "快捷模板动作",
    "模板动作",
    "应用模板",
    "btn:快捷模板动作",
    "btn:模板动作",
    "btn:应用模板",
    "btn:aiWorkflow:applyPreset",
    "btn:/system/tools/ai-workflow:applyPreset",
    "btn:/systemTools/aiWorkflow:applyPreset",
  ],
  copy: [
    "复制",
    "复制请求预览",
    "btn:复制",
    "btn:复制请求预览",
    "btn:aiWorkflow:copy",
    "btn:/system/tools/ai-workflow:copy",
    "btn:/systemTools/aiWorkflow:copy",
  ],
  export: [
    "导出",
    "导出审计时间线",
    "落盘分析",
    "落盘 Prompt",
    "btn:导出",
    "btn:导出审计时间线",
    "btn:落盘分析",
    "btn:落盘 Prompt",
    "btn:aiWorkflow:export",
    "btn:/system/tools/ai-workflow:export",
    "btn:/systemTools/aiWorkflow:export",
    "btn:aiWorkflow:dump",
    "btn:/system/tools/ai-workflow:dump",
  ],
  recordAudit: [
    "写入本地审计",
    "人工记录",
    "btn:写入本地审计",
    "btn:人工记录",
    "btn:aiWorkflow:recordAudit",
    "btn:/system/tools/ai-workflow:recordAudit",
    "btn:/systemTools/aiWorkflow:recordAudit",
  ],
  reset: [
    "恢复默认",
    "新会话",
    "btn:恢复默认",
    "btn:新会话",
    "btn:aiWorkflow:reset",
    "btn:/system/tools/ai-workflow:reset",
    "btn:/systemTools/aiWorkflow:reset",
  ],
  reuse: [
    "回填",
    "回填最近提交",
    "切换到这里",
    "btn:回填",
    "btn:回填最近提交",
    "btn:切换到这里",
    "btn:aiWorkflow:reuse",
    "btn:/system/tools/ai-workflow:reuse",
    "btn:/systemTools/aiWorkflow:reuse",
  ],
  submit: [
    "提交",
    "提交审核请求",
    "生成工作流",
    "开始分析",
    "btn:提交",
    "btn:提交审核请求",
    "btn:生成工作流",
    "btn:开始分析",
    "btn:aiWorkflow:submit",
    "btn:/system/tools/ai-workflow:submit",
    "btn:/systemTools/aiWorkflow:submit",
    "btn:aiWorkflow:workflowPromptChat",
    "btn:/system/tools/ai-workflow:workflowPromptChat",
  ],
};

const AI_WORKFLOW_BUTTON_ACCESS_LABELS: Record<AiWorkflowButtonAction, string[]> = {
  applyPreset: ["快捷模板动作", "模板动作", "应用模板", "applyPreset"],
  copy: ["复制", "复制请求预览", "copy"],
  export: ["导出", "导出审计时间线", "落盘分析", "落盘 Prompt", "export", "dump"],
  recordAudit: ["写入本地审计", "人工记录", "recordAudit"],
  reset: ["恢复默认", "新会话", "reset"],
  reuse: ["回填", "回填最近提交", "切换到这里", "reuse"],
  submit: ["提交", "提交审核请求", "生成工作流", "开始分析", "submit", "workflowPromptChat"],
};

const AI_WORKFLOW_BUTTON_ACCESS_CODE_PREFIXES = [
  "btn:aiWorkflow:",
  "btn:/system/tools/ai-workflow:",
  "btn:/systemTools/aiWorkflow:",
];

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

type WorkflowTreeNode = {
  canRollback?: boolean;
  content: string;
  id: string;
  level: number;
  parentId?: null | string;
  role: string;
  stage?: string;
  timeLabel: string;
};

const RECENT_KEY = "gva.ai.workflow.recent";
const TIMELINE_KEY = "gva.ai.workflow.timeline";

const auth = useAuthStore();
const accessStore = useAccessStore();
const { hasAccessByCodes } = useAccess();
const submitting = ref(false);
const error = ref("");
const recentRecords = ref<RecentRecord[]>([]);
const timeline = ref<TimelineEntry[]>([]);
const success = ref<null | { auditId: string; timeLabel: string }>(null);
const sessionLoading = ref(false);
const sessionSaving = ref(false);
const sessionDeleting = ref(false);
const sessionMutating = ref(false);
const cloudSessions = ref<AiWorkflowSessionListItem[]>([]);
const activeSessionId = ref<null | number>(null);
const activeSessionMessages = ref<AiWorkflowMessageRecord[]>([]);

const form = reactive({
  mode: "system" as Mode,
  articleId: "article-100",
  action: "approve" as WorkflowAction,
  reason: "policy-safe",
  note: "已完成基础策略复核。",
  serviceToken: "",
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

const workflowTree = computed<WorkflowTreeNode[]>(() => {
  const source = activeSessionMessages.value;
  if (source.length === 0) {
    return [];
  }

  const childrenMap = new Map<null | string, AiWorkflowMessageRecord[]>();
  for (const item of source) {
    const key = item.parentId ?? null;
    const bucket = childrenMap.get(key) ?? [];
    bucket.push(item);
    childrenMap.set(key, bucket);
  }
  for (const bucket of childrenMap.values()) {
    bucket.sort((left, right) => new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime());
  }

  const rootCandidates = source
    .filter((item) => !item.parentId || !source.some((candidate) => candidate.id === item.parentId))
    .sort((left, right) => new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime());

  const flattened: WorkflowTreeNode[] = [];
  const visit = (node: AiWorkflowMessageRecord, level: number) => {
    flattened.push({
      canRollback: node.canRollback,
      content: node.content,
      id: node.id,
      level,
      parentId: node.parentId,
      role: node.role,
      stage: node.stage,
      timeLabel: formatTime(node.createdAt),
    });
    for (const child of childrenMap.get(node.id) ?? []) {
      visit(child, level + 1);
    }
  };

  for (const root of rootCandidates) {
    visit(root, 0);
  }
  return flattened;
});

const aiWorkflowButtonAccessCandidates = new Set<string>(
  Object.values(AI_WORKFLOW_BUTTON_ACCESS_CODES).flat(),
);
const aiWorkflowButtonAccessLabels = Object.values(AI_WORKFLOW_BUTTON_ACCESS_LABELS).flat();

const hasAiWorkflowButtonAccessEnvelope = computed(() =>
  accessStore.accessCodes.some(
    (code) =>
      aiWorkflowButtonAccessCandidates.has(code) ||
      AI_WORKFLOW_BUTTON_ACCESS_CODE_PREFIXES.some((prefix) => code.startsWith(prefix)) ||
      aiWorkflowButtonAccessLabels.some(
        (label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`),
      ),
  ),
);

function hasFlexibleAiWorkflowButtonAccess(action: AiWorkflowButtonAction) {
  if (hasAccessByCodes(AI_WORKFLOW_BUTTON_ACCESS_CODES[action])) {
    return true;
  }
  const labels = AI_WORKFLOW_BUTTON_ACCESS_LABELS[action];
  return accessStore.accessCodes.some((code) =>
    labels.some((label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`)),
  );
}

function canUseAiWorkflowAction(action: AiWorkflowButtonAction) {
  return !hasAiWorkflowButtonAccessEnvelope.value || hasFlexibleAiWorkflowButtonAccess(action);
}

function denyAiWorkflowAction(action: AiWorkflowButtonAction) {
  if (canUseAiWorkflowAction(action)) {
    return false;
  }
  error.value = "无按钮权限，当前账号不能执行该 AI 工作流操作。";
  appendTimeline("manual", `按钮权限拦截：${AI_WORKFLOW_BUTTON_ACCESS_LABELS[action][0]}。`);
  return true;
}

watch(
  () => [form.mode, form.articleId, form.action, form.reason, form.scope, form.operatorUserId, form.serviceToken, form.note],
  () => {
    persistTimeline();
  },
  { deep: false },
);

onMounted(() => {
  recentRecords.value = readLocal<RecentRecord[]>(RECENT_KEY, []).map((item) =>
    sanitizeRecentRecord(item),
  );
  writeLocal(RECENT_KEY, recentRecords.value);
  timeline.value = readLocal<TimelineEntry[]>(TIMELINE_KEY, []);
  appendTimeline("draft", "页面已加载，当前草稿已进入本地审计范围。");
  void loadCloudSessions();
});

function buildReasonText() {
  const note = form.note.trim();
  if (!note) {
    return form.reason.trim();
  }
  return `${form.reason.trim()} | ${note}`;
}

function formatTime(isoTime: string) {
  return new Date(isoTime).toLocaleString("zh-CN");
}

function getTimelineTitle(kind: TimelineEntry["kind"]) {
  switch (kind) {
    case "manual": {
      return "人工记录";
    }
    case "preset": {
      return "模板动作";
    }
    case "submit": {
      return "提交动作";
    }
    default: {
      return "草稿变更";
    }
  }
}

function appendTimeline(kind: TimelineEntry["kind"], detail: string) {
  const now = new Date().toISOString();
  const entry: TimelineEntry = {
    id: `${kind}-${now}`,
    kind,
    detail,
    title: getTimelineTitle(kind),
    timeLabel: formatTime(now),
  };
  timeline.value = [entry, ...timeline.value].slice(0, 12);
  persistTimeline();
}

function sanitizeRecentRecord(record: RecentRecord): RecentRecord {
  return {
    accepted: Boolean(record.accepted),
    action: String(record.action || ""),
    articleId: String(record.articleId || ""),
    auditId: String(record.auditId || ""),
    mode: record.mode,
    operatorUserId:
      typeof record.operatorUserId === "number" ? record.operatorUserId : undefined,
    reason: String(record.reason || ""),
    scope: record.scope ? String(record.scope) : undefined,
    submittedAt: String(record.submittedAt || ""),
    timeLabel: String(record.timeLabel || ""),
  };
}

function buildWorkflowMessages(): AiWorkflowMessageRecord[] {
  return timeline.value.map((item, index) => ({
    id: item.id,
    role: item.kind === "submit" ? "assistant" : "user",
    content: `${item.title}：${item.detail}`,
    snapshot: {
      articleId: form.articleId,
      action: form.action,
      mode: form.mode,
      reason: form.reason,
    },
    conversationId: form.articleId.trim() || "ai-workflow",
    messageId: item.id,
    createdAt: new Date().toISOString(),
    parentId: index > 0 ? timeline.value[index - 1]?.id ?? null : null,
    stage: form.action,
    canRollback: item.kind !== "submit",
  }));
}

function buildSessionPayload(): AiWorkflowSessionUpsertInput {
  const redactedFormData = {
    ...form,
    serviceToken: "",
  };
  return {
    id: activeSessionId.value ?? undefined,
    tab: "moderation",
    title: `${form.articleId.trim() || "draft"} · ${form.action}`,
    summary: buildReasonText(),
    conversationId: form.articleId.trim() || `workflow-${Date.now()}`,
    messageId: success.value?.auditId || timeline.value[0]?.id || `draft-${Date.now()}`,
    currentNodeId: success.value?.auditId || timeline.value[0]?.id || `node-${Date.now()}`,
    currentStage: form.action,
    settings: {
      mode: form.mode,
      operatorUserId: form.operatorUserId,
      scope: form.scope,
    },
    formData: redactedFormData,
    resultData: {
      recentRecords: recentRecords.value.map((item) => sanitizeRecentRecord(item)),
      success: success.value,
    },
    messages: buildWorkflowMessages(),
  };
}

function syncActiveMessages(messages: AiWorkflowMessageRecord[]) {
  activeSessionMessages.value = messages;
}

function hydrateFromCloudSession(session: {
  ID: number;
  formData?: Record<string, unknown>;
  messages?: AiWorkflowMessageRecord[];
  resultData?: Record<string, unknown>;
}) {
  activeSessionId.value = session.ID;
  syncActiveMessages(session.messages || []);
  const data = (session.formData || {}) as Record<string, unknown>;
  form.mode = (data.mode as Mode) || "system";
  form.articleId = String(data.articleId || form.articleId);
  form.action = (data.action as WorkflowAction) || "approve";
  form.reason = String(data.reason || form.reason);
  form.note = String(data.note || "");
  form.serviceToken = "";
  form.operatorUserId = Number(data.operatorUserId || form.operatorUserId || 1);
  form.scope = String(data.scope || form.scope);
  const resultData = (session.resultData || {}) as Record<string, unknown>;
  recentRecords.value = Array.isArray(resultData.recentRecords)
    ? (resultData.recentRecords as RecentRecord[]).map((item) =>
        sanitizeRecentRecord(item),
      )
    : recentRecords.value;
  writeLocal(RECENT_KEY, recentRecords.value);
  timeline.value = (session.messages || []).slice(0, 12).map((item) => ({
    id: item.id,
    kind: item.role === "assistant" ? "submit" : "manual",
    detail: item.content,
    title: item.stage || getTimelineTitle(item.role === "assistant" ? "submit" : "manual"),
    timeLabel: formatTime(item.createdAt),
  }));
  persistTimeline();
}

async function loadCloudSessions() {
  sessionLoading.value = true;
  try {
    const result = await getAIWorkflowSessionListApi({ tab: "moderation" });
    cloudSessions.value = result.List ?? [];
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "获取云端会话失败";
  } finally {
    sessionLoading.value = false;
  }
}

async function saveCloudSession() {
  if (denyAiWorkflowAction("recordAudit")) {
    return;
  }
  sessionSaving.value = true;
  try {
    const session = await saveAIWorkflowSessionApi(buildSessionPayload());
    activeSessionId.value = session.ID;
    syncActiveMessages(session.messages || []);
    appendTimeline("manual", `已保存云端会话 #${session.ID}。`);
    await loadCloudSessions();
    message.success("已保存云端会话");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "保存云端会话失败";
  } finally {
    sessionSaving.value = false;
  }
}

async function reuseCloudSession(entry: AiWorkflowSessionListItem) {
  if (denyAiWorkflowAction("reuse")) {
    return;
  }
  sessionLoading.value = true;
  try {
    const session = await getAIWorkflowSessionDetailApi(entry.ID);
    hydrateFromCloudSession(session);
    appendTimeline("manual", `已回填云端会话 #${entry.ID}。`);
    message.success("已回填云端会话");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "回填云端会话失败";
  } finally {
    sessionLoading.value = false;
  }
}

async function resumeCloudSession(entry: AiWorkflowSessionListItem) {
  if (denyAiWorkflowAction("submit")) {
    return;
  }
  const prompt =
    typeof window === "undefined"
      ? "继续分析并给出下一步建议"
      : (window.prompt("请输入续聊提示词：", form.note.trim() || "继续分析并给出下一步建议") || "").trim();
  if (!prompt) {
    error.value = "续聊提示词不能为空。";
    return;
  }
  sessionMutating.value = true;
  try {
    const session = await resumeAIWorkflowChatApi({
      sessionId: entry.ID,
      nodeId: entry.currentNodeId || undefined,
      messageId: entry.currentNodeId || undefined,
      prompt,
      tab: "moderation",
    });
    hydrateFromCloudSession(session);
    appendTimeline("manual", `已续聊云端会话 #${entry.ID}。`);
    await loadCloudSessions();
    message.success("已续聊云端会话");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "续聊云端会话失败";
  } finally {
    sessionMutating.value = false;
  }
}

async function deleteCloudSession(entry: AiWorkflowSessionListItem) {
  if (denyAiWorkflowAction("reset")) {
    return;
  }
  sessionDeleting.value = true;
  try {
    await deleteAIWorkflowSessionApi({ ID: entry.ID });
    if (activeSessionId.value === entry.ID) {
      activeSessionId.value = null;
    }
    appendTimeline("manual", `已删除云端会话 #${entry.ID}。`);
    await loadCloudSessions();
    message.success("已删除云端会话");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "删除云端会话失败";
  } finally {
    sessionDeleting.value = false;
  }
}

async function dumpCloudMarkdown() {
  if (denyAiWorkflowAction("export")) {
    return;
  }
  try {
    const result = await dumpAIWorkflowMarkdownApi(buildSessionPayload());
    const text = `${result.filePath} (${result.relativePath})`;
    await navigator.clipboard.writeText(text);
    appendTimeline("manual", `已导出云端 Markdown：${result.relativePath}。`);
    message.success("已导出 Markdown，路径已复制");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "导出 Markdown 失败";
  }
}

async function syncCloudStage(entry: AiWorkflowSessionListItem) {
  if (denyAiWorkflowAction("recordAudit")) {
    return;
  }
  sessionMutating.value = true;
  try {
    await updateAIWorkflowStageApi({
      sessionId: entry.ID,
      currentStage: form.action,
      nodeId: entry.currentNodeId || undefined,
    });
    appendTimeline("manual", `已将云端会话 #${entry.ID} 阶段更新为 ${form.action}。`);
    await loadCloudSessions();
    message.success("已同步云端阶段");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "同步云端阶段失败";
  } finally {
    sessionMutating.value = false;
  }
}

async function rollbackCloudSession(entry: AiWorkflowSessionListItem) {
  if (denyAiWorkflowAction("reset")) {
    return;
  }
  sessionMutating.value = true;
  try {
    await rollbackAIWorkflowNodeApi({
      sessionId: entry.ID,
      nodeId: entry.currentNodeId || undefined,
      messageId: entry.currentNodeId || undefined,
      id: entry.ID,
    });
    appendTimeline("manual", `已触发云端会话 #${entry.ID} 的回滚请求。`);
    await loadCloudSessions();
    message.success("已触发云端回滚");
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "回滚云端会话失败";
  } finally {
    sessionMutating.value = false;
  }
}

function applyPreset(preset: PresetAction) {
  if (denyAiWorkflowAction("applyPreset")) {
    return;
  }
  form.mode = preset.mode;
  form.action = preset.action;
  form.reason = preset.reason;
  if (preset.scope) {
    form.scope = preset.scope;
  }
  appendTimeline("preset", `已应用模板动作：${preset.label}。`);
}

async function copyPreview() {
  if (denyAiWorkflowAction("copy")) {
    return;
  }
  try {
    await navigator.clipboard.writeText(requestPreview.value);
    appendTimeline("manual", "已复制请求预览。");
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", requestPreview.value);
    appendTimeline("manual", "当前环境不支持自动复制，已提供手动复制。");
  }
}

function exportAuditTrail() {
  if (denyAiWorkflowAction("export")) {
    return;
  }
  if (typeof window === "undefined") {
    appendTimeline("manual", "当前环境不支持导出审计时间线。");
    return;
  }
  const exportPayload = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      draft: { ...form },
      preview: payload.value,
      recentRecords: recentRecords.value,
      timeline: timeline.value,
    },
    null,
    2,
  );
  const blob = new Blob([exportPayload], { type: "application/json;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `ai-workflow-audit-${Date.now()}.json`;
  link.click();
  window.URL.revokeObjectURL(url);
  appendTimeline("manual", "已导出本地审计时间线。");
}

function reuseRecord(entry: RecentRecord) {
  if (denyAiWorkflowAction("reuse")) {
    return;
  }
  form.mode = entry.mode;
  form.articleId = entry.articleId;
  form.action = entry.action as WorkflowAction;
  form.reason = entry.reason;
  form.note = "";
  form.operatorUserId = entry.operatorUserId || form.operatorUserId;
  form.scope = entry.scope || form.scope;
  success.value = {
    auditId: entry.auditId,
    timeLabel: entry.timeLabel,
  };
  appendTimeline("manual", `已回填审计记录 ${entry.auditId}。`);
}

function loadLatestRecord() {
  const latest = recentRecords.value[0];
  if (!latest) {
    error.value = "暂无最近提交记录可回填";
    return;
  }
  error.value = "";
  reuseRecord(latest);
}

function resetForm() {
  if (denyAiWorkflowAction("reset")) {
    return;
  }
  form.mode = "system";
  form.articleId = "article-100";
  form.action = "approve";
  form.reason = "policy-safe";
  form.note = "已完成基础策略复核。";
  form.serviceToken = "";
  form.operatorUserId = auth.userInfo?.ID ?? 1;
  form.scope = "user.violation.handle";
  error.value = "";
  success.value = null;
  appendTimeline("draft", "已恢复默认草稿。");
}

async function submit() {
  if (denyAiWorkflowAction("submit")) {
    return;
  }
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
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "提交失败";
    appendTimeline("submit", `审核请求提交失败：${error.value}`);
  } finally {
    submitting.value = false;
  }
}

function recordManualAuditCheck() {
  if (denyAiWorkflowAction("recordAudit")) {
    return;
  }
  appendTimeline("manual", "人工记录一次本地审计检查。");
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

<template>
  <Page
    title="AI 审核工作台"
  >
    <div class="workbench">
      <div class="content-grid">
        <Card class="panel-card" :bordered="false">
          <template #title>
            <div class="card-title-row">
              <span>审核草稿</span>
              <Space wrap>
                <Button :disabled="!canUseAiWorkflowAction('reuse')" @click="loadLatestRecord">回填最近提交</Button>
                <Button :disabled="!canUseAiWorkflowAction('reset')" @click="resetForm">恢复默认</Button>
              </Space>
            </div>
          </template>

          <Alert
            :message="validationIssues.length > 0 ? validationIssues[0] : '请求体完整，可直接提交审核工作流。'"
            :type="validationIssues.length > 0 ? 'warning' : 'success'"
            show-icon
          />
          <Alert v-if="error" class="mt-4" :message="error" show-icon type="error" />
          <Alert
            v-if="success"
            class="mt-4"
            :message="`最近审计 ID：${success.auditId} · ${success.timeLabel}`"
            show-icon
            type="success"
          />

          <Collapse class="mt-4" ghost>
            <CollapsePanel key="presets" header="快捷模板动作">
              <div class="tag-wrap">
                <Tag
                  v-for="preset in presets"
                  :key="preset.label"
                  class="cursor-tag"
                  :class="{ 'cursor-tag-disabled': !canUseAiWorkflowAction('applyPreset') }"
                  :tabindex="canUseAiWorkflowAction('applyPreset') ? 0 : -1"
                  color="blue"
                  @click="applyPreset(preset)"
                >
                  {{ preset.label }} · {{ preset.action }} · {{ preset.mode }}
                </Tag>
              </div>
            </CollapsePanel>
            <CollapsePanel key="mode" header="模式说明">
              <ul class="bullet-list">
                <li v-for="item in modeMeta.tips" :key="item">{{ item }}</li>
              </ul>
            </CollapsePanel>
          </Collapse>

          <Form class="mt-4" layout="vertical">
            <div class="form-grid">
              <FormItem label="执行模式">
                <Select v-model:value="form.mode">
                  <SelectOption value="system">system</SelectOption>
                  <SelectOption value="delegated">delegated</SelectOption>
                </Select>
              </FormItem>
              <FormItem label="目标标识">
                <Input v-model:value="form.articleId" placeholder="article-202 / user-42 / comment-18" @keydown.enter.prevent="submit" />
              </FormItem>
              <FormItem label="处置动作">
                <Select v-model:value="form.action">
                  <SelectOption value="approve">approve</SelectOption>
                  <SelectOption value="reject">reject</SelectOption>
                  <SelectOption value="ban">ban</SelectOption>
                  <SelectOption value="escalate">escalate</SelectOption>
                </Select>
              </FormItem>
              <FormItem label="处置原因">
                <Input v-model:value="form.reason" placeholder="例如 policy-safe / spam-hit / manual-escalation" @keydown.enter.prevent="submit" />
              </FormItem>
            </div>
            <FormItem label="审核备注">
              <Input.TextArea v-model:value="form.note" :rows="4" placeholder="补充命中策略、复核结论、升级原因或交接说明。" />
            </FormItem>

            <div v-if="form.mode === 'system'" class="form-grid compact-grid">
              <FormItem label="Service Token">
                <Input
                  v-model:value="form.serviceToken"
                  type="password"
                  autocomplete="off"
                  placeholder="请输入服务令牌"
                  @keydown.enter.prevent="submit"
                />
              </FormItem>
              <FormItem label="令牌摘要">
                <Input :value="serviceTokenSummary" disabled />
              </FormItem>
            </div>

            <div v-else class="form-grid compact-grid">
              <FormItem label="Operator User ID">
                <InputNumber v-model:value="form.operatorUserId" :min="1" class="full-width" />
              </FormItem>
              <FormItem label="授权 Scope">
                <Input v-model:value="form.scope" placeholder="例如 user.violation.handle" @keydown.enter.prevent="submit" />
              </FormItem>
            </div>
          </Form>

          <Space wrap>
            <Button
              v-if="canUseAiWorkflowAction('submit')"
              type="primary"
              :disabled="submitting || validationIssues.length > 0"
              :loading="submitting"
              @click="submit"
            >
              提交审核请求
            </Button>
            <Button :disabled="sessionSaving || !canUseAiWorkflowAction('recordAudit')" @click="saveCloudSession">保存云端会话</Button>
            <Button :disabled="!canUseAiWorkflowAction('copy')" @click="copyPreview">复制请求预览</Button>
            <Button :disabled="!canUseAiWorkflowAction('export')" @click="exportAuditTrail">导出审计时间线</Button>
            <Button :disabled="!canUseAiWorkflowAction('export')" @click="dumpCloudMarkdown">导出 Markdown</Button>
            <Button :disabled="!canUseAiWorkflowAction('recordAudit')" @click="recordManualAuditCheck">写入本地审计</Button>
          </Space>
        </Card>

        <Card class="panel-card" :bordered="false">
          <template #title>
            <div class="card-title-row">
              <span>请求预览与历史</span>
              <Tag color="processing">{{ previewSize }}</Tag>
            </div>
          </template>

          <Tabs>
            <TabPane key="preview" tab="请求预览">
              <pre class="pre-block">{{ requestPreview }}</pre>
            </TabPane>
            <TabPane key="recent" tab="最近提交记录">
              <div v-if="recentRecords.length > 0" class="history-stack">
                <Card v-for="entry in recentRecords" :key="entry.auditId" class="history-card" size="small">
                  <div class="card-title-row">
                    <strong>{{ entry.action }}</strong>
                    <Tag :color="entry.mode === 'system' ? 'blue' : 'purple'">{{ entry.mode }}</Tag>
                  </div>
                  <p class="history-copy">{{ entry.articleId }} · {{ entry.reason }}</p>
                  <p class="history-copy">{{ entry.timeLabel }} · {{ entry.auditId }}</p>
                  <Button size="small" :disabled="!canUseAiWorkflowAction('reuse')" @click="reuseRecord(entry)">回填</Button>
                </Card>
              </div>
              <Alert v-else message="暂无本地提交记录，提交后会自动生成审计快照。" show-icon type="info" />
            </TabPane>
            <TabPane key="timeline" tab="本地审计时间线">
              <div v-if="timeline.length > 0" class="history-stack">
                <Card v-for="item in timeline" :key="item.id" size="small">
                  <div class="card-title-row">
                    <strong>{{ item.title }}</strong>
                    <Tag>{{ item.kind }}</Tag>
                  </div>
                  <p class="history-copy">{{ item.detail }}</p>
                  <p class="history-copy">{{ item.timeLabel }}</p>
                </Card>
              </div>
              <Alert v-else message="暂无时间线。" show-icon type="info" />
            </TabPane>
            <TabPane key="tree" tab="流程树快照">
              <div v-if="workflowTree.length > 0" class="history-stack">
                <Card
                  v-for="node in workflowTree"
                  :key="node.id"
                  size="small"
                  :style="{ marginLeft: `${node.level * 16}px` }"
                >
                  <div class="card-title-row">
                    <strong>{{ node.stage || node.role }}</strong>
                    <Space wrap>
                      <Tag>{{ node.role }}</Tag>
                      <Tag v-if="node.canRollback" color="orange">可回滚</Tag>
                    </Space>
                  </div>
                  <p class="history-copy">{{ node.content }}</p>
                  <p class="history-copy">{{ node.timeLabel }} · parent={{ node.parentId || 'root' }}</p>
                </Card>
              </div>
              <Alert v-else message="暂无流程树快照，请先保存或回填云端会话。" show-icon type="info" />
            </TabPane>
            <TabPane key="cloud" tab="云端会话">
              <div v-if="cloudSessions.length > 0" class="history-stack">
                <Card v-for="entry in cloudSessions" :key="entry.ID" class="history-card" size="small">
                  <div class="card-title-row">
                    <strong>{{ entry.title }}</strong>
                    <Tag :color="activeSessionId === entry.ID ? 'processing' : 'default'">#{{ entry.ID }}</Tag>
                  </div>
                  <p class="history-copy">{{ entry.summary || '暂无摘要' }}</p>
                  <p class="history-copy">{{ entry.currentStage || '-' }} · {{ entry.UpdatedAt }}</p>
                  <Space wrap>
                    <Button size="small" :disabled="sessionLoading || !canUseAiWorkflowAction('reuse')" @click="reuseCloudSession(entry)">回填</Button>
                    <Button size="small" :disabled="sessionDeleting || !canUseAiWorkflowAction('copy')" @click="activeSessionId = entry.ID">设为当前</Button>
                    <Button size="small" :disabled="sessionMutating || !canUseAiWorkflowAction('submit')" @click="resumeCloudSession(entry)">续聊</Button>
                    <Button size="small" :disabled="sessionMutating || !canUseAiWorkflowAction('recordAudit')" @click="syncCloudStage(entry)">同步阶段</Button>
                    <Button size="small" :disabled="sessionMutating || !canUseAiWorkflowAction('reset')" @click="rollbackCloudSession(entry)">回滚</Button>
                    <Button danger size="small" :disabled="sessionDeleting || !canUseAiWorkflowAction('reset')" @click="deleteCloudSession(entry)">删除</Button>
                  </Space>
                </Card>
              </div>
              <Alert v-else :message="sessionLoading ? '正在加载云端会话…' : '暂无云端会话。'" show-icon type="info" />
            </TabPane>
          </Tabs>
        </Card>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.workbench {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-grid,
.content-grid,
.form-grid {
  display: grid;
  gap: 16px;
}

.metric-grid {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.content-grid {
  align-items: start;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
}

.form-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.compact-grid {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.metric-head,
.card-title-row {
  align-items: center;
  display: flex;
  gap: 8px;
  justify-content: space-between;
}

.metric-head {
  color: var(--ant-color-text-secondary);
  justify-content: flex-start;
  margin-bottom: 12px;
}

.metric-icon {
  font-size: 18px;
}

.metric-note,
.history-copy {
  color: var(--ant-color-text-secondary);
}

.metric-highlight {
  font-size: 24px;
  font-weight: 600;
  line-height: 1.2;
}

.panel-card {
  border-radius: 16px;
}

.tag-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.cursor-tag {
  cursor: pointer;
}

.cursor-tag-disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.bullet-list {
  margin: 0;
  padding-left: 18px;
}

.pre-block {
  margin: 0;
  max-height: 520px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.history-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.history-card {
  border-radius: 12px;
}

.full-width {
  width: 100%;
}

@media (max-width: 960px) {
  .content-grid {
    grid-template-columns: 1fr;
  }

  .card-title-row {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
