<script setup lang="ts">
import type { EmailRecord } from "#/types/gin-ai-admin";

import { computed, onMounted, reactive, ref } from "vue";

import { useAccess } from "@vben/access";
import { Page } from "@vben/common-ui";
import { IconifyIcon } from "@vben/icons";
import { useAccessStore } from "@vben/stores";

import { Alert, Button, TabPane, Tabs } from "ant-design-vue";

import { emailTestApi, getEmailListApi, sendEmailApi } from "#/api/gin-ai-admin/admin";

type EmailTemplate = {
  body: string;
  description: string;
  id: string;
  label: string;
  recommendedTo: string;
  subject: string;
};

type LocalEmailPreset = {
  body: string;
  id: string;
  name: string;
  savedAt: number;
  sourceTemplateId: string;
  subject: string;
  to: string;
};
type EmailConfigDraft = {
  from: string;
  host: string;
  isLoginAuth: boolean;
  isSSL: boolean;
  nickname: string;
  port: number;
  secret: string;
  to: string;
};

type PersistedEmailConfigDraft = Omit<EmailConfigDraft, "secret">;
type EmailPluginButtonAction =
  | "applyTemplate"
  | "copy"
  | "export"
  | "preset"
  | "refresh"
  | "send"
  | "test";

const EMAIL_PLUGIN_BUTTON_ACCESS_CODES: Record<EmailPluginButtonAction, string[]> = {
  applyTemplate: [
    "套用模板",
    "恢复模板正文",
    "收件预设",
    "btn:套用模板",
    "btn:恢复模板正文",
    "btn:收件预设",
    "btn:emailPlugin:applyTemplate",
    "btn:pluginEmail:applyTemplate",
    "btn:/system/tools/plugin-email:applyTemplate",
    "btn:/plugin-email:applyTemplate",
  ],
  copy: [
    "复制",
    "复制最近记录",
    "复制摘要",
    "btn:复制",
    "btn:复制最近记录",
    "btn:复制摘要",
    "btn:emailPlugin:copy",
    "btn:pluginEmail:copy",
    "btn:/system/tools/plugin-email:copy",
    "btn:/plugin-email:copy",
  ],
  export: [
    "导出",
    "导出记录",
    "btn:导出",
    "btn:导出记录",
    "btn:emailPlugin:export",
    "btn:pluginEmail:export",
    "btn:/system/tools/plugin-email:export",
    "btn:/plugin-email:export",
  ],
  preset: [
    "保存本地预设",
    "载入预设",
    "删除预设",
    "btn:保存本地预设",
    "btn:载入预设",
    "btn:删除预设",
    "btn:emailPlugin:preset",
    "btn:pluginEmail:preset",
    "btn:/system/tools/plugin-email:preset",
    "btn:/plugin-email:preset",
  ],
  refresh: [
    "刷新",
    "查询",
    "btn:刷新",
    "btn:查询",
    "btn:emailPlugin:refresh",
    "btn:pluginEmail:refresh",
    "btn:/system/tools/plugin-email:refresh",
    "btn:/plugin-email:refresh",
  ],
  send: [
    "发送邮件",
    "发送",
    "btn:发送邮件",
    "btn:发送",
    "btn:emailPlugin:send",
    "btn:pluginEmail:send",
    "btn:/system/tools/plugin-email:send",
    "btn:/plugin-email:send",
  ],
  test: [
    "发送测试邮件",
    "测试邮件",
    "btn:发送测试邮件",
    "btn:测试邮件",
    "btn:emailPlugin:test",
    "btn:pluginEmail:test",
    "btn:/system/tools/plugin-email:test",
    "btn:/plugin-email:test",
  ],
};

const EMAIL_PLUGIN_BUTTON_ACCESS_LABELS: Record<EmailPluginButtonAction, string[]> = {
  applyTemplate: ["套用模板", "恢复模板正文", "收件预设", "applyTemplate"],
  copy: ["复制", "复制最近记录", "复制摘要", "copy"],
  export: ["导出", "导出记录", "export"],
  preset: ["保存本地预设", "载入预设", "删除预设", "preset"],
  refresh: ["刷新", "查询", "refresh"],
  send: ["发送邮件", "发送", "send"],
  test: ["发送测试邮件", "测试邮件", "test"],
};

const EMAIL_PLUGIN_BUTTON_ACCESS_CODE_PREFIXES = [
  "btn:emailPlugin:",
  "btn:pluginEmail:",
  "btn:/system/tools/plugin-email:",
  "btn:/plugin-email:",
];

const PRESET_STORAGE_KEY = "gaa-email-plugin-presets-v1";
const EMAIL_CONFIG_STORAGE_KEY = "gaa-email-plugin-config-v1";
const templateLibrary: EmailTemplate[] = [
  {
    id: "ops-healthcheck",
    label: "运维巡检模板",
    description: "适合验证插件链路是否畅通，并提醒值班同学检查附件/链接。",
    recommendedTo: "ops-team@gaa.local",
    subject: "GAA 运维巡检提醒",
    body: [
      "各位同学好，",
      "",
      "请在今日巡检窗口内完成以下核对：",
      "1. 检查公告附件链接是否可访问；",
      "2. 核对核心插件页面 smoke 是否通过；",
      "3. 回填异常日志与处理结论。",
      "",
      "如链路异常，请在 30 分钟内同步到值班群。",
    ].join("\n"),
  },
  {
    id: "release-window",
    label: "发布窗口模板",
    description: "用于发布前确认邮件，默认指向发布值班地址。",
    recommendedTo: "release-review@gaa.local",
    subject: "GAA 发布窗口确认",
    body: [
      "发布值班同学好，",
      "",
      "本次发布窗口请重点确认：",
      "- 版本包、插件包是否齐备；",
      "- 回滚联系人和时间窗是否明确；",
      "- 邮件插件与公告附件手动链路是否已复核。",
      "",
      "确认完成后请回复 OK。",
    ].join("\n"),
  },
  {
    id: "content-sync",
    label: "公告联动模板",
    description: "适合内容/运营协作时同步公告、邮件和手动链接复核结论。",
    recommendedTo: "content-ops@gaa.local",
    subject: "GAA 公告联动复核",
    body: [
      "内容运营同学好，",
      "",
      "请同步核对以下事项：",
      "- 公告标题与正文是否一致；",
      "- 附件手动链接是否可访问；",
      "- 对外通知邮件正文是否已同步最新结论。",
      "",
      "如需改稿，请直接在预设基础上编辑后回传。",
    ].join("\n"),
  },
];
const recipientPresets = [
  { label: "运维组", value: "ops-team@gaa.local" },
  { label: "发布值班", value: "release-review@gaa.local" },
  { label: "内容运营", value: "content-ops@gaa.local" },
];

const records = ref<EmailRecord[]>([]);
const message = ref("");
const error = ref("");
const loading = ref(false);
const sending = ref(false);
const activeTab = ref("compose");
const accessStore = useAccessStore();
const { hasAccessByCodes } = useAccess();
const selectedTemplateId = ref(templateLibrary[0]?.id ?? "");
const localPresets = ref<LocalEmailPreset[]>([]);
const emailConfig = reactive<EmailConfigDraft>({
  from: 'no-reply@example.com',
  host: 'smtp.example.com',
  isLoginAuth: true,
  isSSL: true,
  nickname: 'Vue Rust Admin',
  port: 465,
  secret: '',
  to: 'ops-team@gaa.local',
});
const form = reactive({
  to: "ops-team@gaa.local",
  subject: "GAA 邮件测试",
  body: "当前邮件由 Rust / Vben 重构版邮件插件工作台登记。",
});

const activeTemplate = computed(
  () => templateLibrary.find((item) => item.id === selectedTemplateId.value) ?? null,
);
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.to.trim()) issues.push("目标邮箱不能为空。");
  if (form.to && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.to.trim())) issues.push("目标邮箱格式不正确。");
  if (!form.subject.trim()) issues.push("邮件标题不能为空。");
  if (!form.body.trim()) issues.push("邮件内容不能为空。");
  return issues;
});
const emailPluginButtonAccessCandidates = new Set<string>(
  Object.values(EMAIL_PLUGIN_BUTTON_ACCESS_CODES).flat(),
);
const emailPluginButtonAccessLabels = Object.values(EMAIL_PLUGIN_BUTTON_ACCESS_LABELS).flat();
const hasEmailPluginButtonAccessEnvelope = computed(() =>
  accessStore.accessCodes.some(
    (code) =>
      emailPluginButtonAccessCandidates.has(code) ||
      EMAIL_PLUGIN_BUTTON_ACCESS_CODE_PREFIXES.some((prefix) => code.startsWith(prefix)) ||
      emailPluginButtonAccessLabels.some(
        (label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`),
      ),
  ),
);

function hasFlexibleEmailPluginButtonAccess(action: EmailPluginButtonAction) {
  if (hasAccessByCodes(EMAIL_PLUGIN_BUTTON_ACCESS_CODES[action])) {
    return true;
  }
  const labels = EMAIL_PLUGIN_BUTTON_ACCESS_LABELS[action];
  return accessStore.accessCodes.some((code) =>
    labels.some((label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`)),
  );
}

function canUseEmailPluginAction(action: EmailPluginButtonAction) {
  return !hasEmailPluginButtonAccessEnvelope.value || hasFlexibleEmailPluginButtonAccess(action);
}

function denyEmailPluginAction(
  action: EmailPluginButtonAction,
  label = EMAIL_PLUGIN_BUTTON_ACCESS_LABELS[action][0],
) {
  if (canUseEmailPluginAction(action)) {
    return false;
  }
  error.value = `无按钮权限，当前账号不能执行「${label}」操作。`;
  return true;
}

async function loadAll(showMessage = false) {
  if (showMessage && denyEmailPluginAction("refresh", "刷新")) {
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const result = await getEmailListApi();
    records.value = result.List.toSorted((a, b) => b.createdAt - a.createdAt);
    message.value = "已刷新邮件记录。";
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "获取邮件记录失败";
  } finally {
    loading.value = false;
  }
}

function loadLocalPresets() {
  if (typeof window === "undefined") return;
  try {
    const raw = window.localStorage.getItem(PRESET_STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as LocalEmailPreset[];
    localPresets.value = Array.isArray(parsed)
      ? parsed
          .filter((item) => item && typeof item === "object")
          .toSorted((a, b) => b.savedAt - a.savedAt)
      : [];
  } catch {
    localPresets.value = [];
  }
}

function loadEmailConfig() {
  if (typeof window === 'undefined') return;
  const raw = window.localStorage.getItem(EMAIL_CONFIG_STORAGE_KEY);
  if (!raw) return;
  try {
    const parsed = JSON.parse(raw) as PersistedEmailConfigDraft;
    Object.assign(emailConfig, parsed);
    emailConfig.secret = "";
  } catch {
    // ignore bad local config
  }
}

function saveEmailConfig() {
  if (typeof window === 'undefined') return;
  const persisted: PersistedEmailConfigDraft = {
    from: emailConfig.from,
    host: emailConfig.host,
    isLoginAuth: emailConfig.isLoginAuth,
    isSSL: emailConfig.isSSL,
    nickname: emailConfig.nickname,
    port: emailConfig.port,
    to: emailConfig.to,
  };
  window.localStorage.setItem(EMAIL_CONFIG_STORAGE_KEY, JSON.stringify(persisted));
  message.value = '邮件配置已保存到本地。';
}

function persistLocalPresets() {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(PRESET_STORAGE_KEY, JSON.stringify(localPresets.value));
}

function applyTemplate(templateId: string, options: { preserveRecipient?: boolean } = {}) {
  if (denyEmailPluginAction("applyTemplate", "套用模板")) {
    return;
  }
  const template = templateLibrary.find((item) => item.id === templateId);
  if (!template) return;
  selectedTemplateId.value = template.id;
  if (!options.preserveRecipient) {
    form.to = template.recommendedTo;
  }
  form.subject = template.subject;
  form.body = template.body;
  message.value = `已套用模板：${template.label}`;
}

function applySelectedTemplate() {
  if (!selectedTemplateId.value) return;
  applyTemplate(selectedTemplateId.value);
}

function restoreTemplateDraft() {
  if (!activeTemplate.value) {
    message.value = "当前未选择模板。";
    return;
  }
  applyTemplate(activeTemplate.value.id, { preserveRecipient: true });
  message.value = `已按 ${activeTemplate.value.label} 恢复标题和正文，保留当前收件人。`;
}

function saveCurrentAsPreset() {
  if (denyEmailPluginAction("preset", "保存本地预设")) {
    return;
  }
  if (typeof window === "undefined") {
    message.value = "当前环境不支持保存本地预设。";
    return;
  }
  const defaultName = activeTemplate.value?.label ? `${activeTemplate.value.label}（本地）` : form.subject.trim() || "邮件预设";
  const name = window.prompt("请输入本地预设名称：", defaultName)?.trim();
  if (!name) return;
  const preset: LocalEmailPreset = {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    name,
    savedAt: Date.now(),
    sourceTemplateId: selectedTemplateId.value,
    to: form.to.trim(),
    subject: form.subject.trim(),
    body: form.body.trim(),
  };
  const existingIndex = localPresets.value.findIndex((item) => item.name === name);
  if (existingIndex === -1) {
    localPresets.value.unshift(preset);
  } else {
    localPresets.value.splice(existingIndex, 1, preset);
  }
  localPresets.value = localPresets.value.toSorted((a, b) => b.savedAt - a.savedAt);
  persistLocalPresets();
  message.value = `已保存本地预设：${name}`;
}

function applyLocalPreset(item: LocalEmailPreset) {
  if (denyEmailPluginAction("preset", "载入预设")) {
    return;
  }
  selectedTemplateId.value = item.sourceTemplateId;
  form.to = item.to;
  form.subject = item.subject;
  form.body = item.body;
  message.value = `已载入本地预设：${item.name}`;
}

function removeLocalPreset(item: LocalEmailPreset) {
  if (denyEmailPluginAction("preset", "删除预设")) {
    return;
  }
  if (typeof window !== "undefined" && !window.confirm(`确认删除本地预设“${item.name}”吗？`)) {
    return;
  }
  localPresets.value = localPresets.value.filter((preset) => preset.id !== item.id);
  persistLocalPresets();
  message.value = `已删除本地预设：${item.name}`;
}

function useRecipientPreset(value: string) {
  if (denyEmailPluginAction("applyTemplate", "收件预设")) {
    return;
  }
  form.to = value;
  message.value = `已切换收件预设：${value}`;
}

async function sendTest() {
  await dispatch("test");
}

async function sendEmail() {
  await dispatch("send");
}

async function dispatch(mode: "send" | "test") {
  if (denyEmailPluginAction(mode === "test" ? "test" : "send")) {
    return;
  }
  if (validationIssues.value.length > 0) {
    error.value = validationIssues.value[0] ?? "邮件表单不完整";
    return;
  }
  sending.value = true;
  error.value = "";
  try {
    const payload = { to: form.to.trim(), subject: form.subject.trim(), body: form.body.trim() };
    const created = mode === "test" ? await emailTestApi(payload) : await sendEmailApi(payload);
    message.value = `${mode === "test" ? "测试邮件" : "邮件"}已发送：${created.to}`;
    await loadAll();
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "发送邮件失败";
  } finally {
    sending.value = false;
  }
}

async function copyLatest() {
  if (denyEmailPluginAction("copy", "复制最近记录")) {
    return;
  }
  if (records.value[0]) {
    await copyRecord(records.value[0]);
  }
}

async function copyRecord(item: EmailRecord) {
  if (denyEmailPluginAction("copy", "复制摘要")) {
    return;
  }
  const text = [
    `收件人：${item.to}`,
    `标题：${item.subject}`,
    `模式：${item.mode}`,
    `状态：${item.status}`,
    `时间：${formatTime(item.createdAt)}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = "已复制邮件摘要。";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制邮件摘要。";
  }
}

function exportRecords() {
  if (denyEmailPluginAction("export", "导出记录")) {
    return;
  }
  if (typeof window === "undefined") {
    message.value = "当前环境不支持导出邮件记录。";
    return;
  }
  const content = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      currentDraft: { ...form },
      selectedTemplateId: selectedTemplateId.value,
      templateLibrary,
      localPresets: localPresets.value,
      records: records.value,
    },
    null,
    2,
  );
  const blob = new Blob([content], { type: "application/json;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `email-plugin-records-${Date.now()}.json`;
  link.click();
  window.URL.revokeObjectURL(url);
  message.value = `已导出 ${records.value.length} 条邮件记录。`;
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  loadLocalPresets();
  loadEmailConfig();
  void loadAll();
});
</script>

<template>
  <Page>
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon class="text-lg" icon="lucide:mail" />
        <span>邮件插件</span>
      </div>
    </template>
    <template #extra>
      <div class="flex flex-wrap items-center gap-2">
        <Button :loading="loading" :disabled="!canUseEmailPluginAction('refresh')" @click="loadAll(true)">刷新</Button>
        <Button :disabled="records.length === 0 || !canUseEmailPluginAction('copy')" @click="copyLatest">复制最近记录</Button>
        <Button :disabled="!canUseEmailPluginAction('preset')" @click="saveCurrentAsPreset">保存本地预设</Button>
        <Button :disabled="records.length === 0 || !canUseEmailPluginAction('export')" @click="exportRecords">导出记录</Button>
      </div>
    </template>

    <Alert v-if="message" class="mb-4" show-icon type="info" :message="message" />
    <Alert v-if="error" class="mb-4" show-icon type="error" :message="error" />

    <Tabs v-model:activeKey="activeTab">
      <TabPane key="compose" tab="邮件发送">
        <div class="split-grid">
          <div class="card">
            <h3 class="title">发送表单</h3>
            <div class="field">
              <label>模板 / 预设</label>
              <div class="toolbar-grid">
                <select v-model="selectedTemplateId">
                  <option v-for="item in templateLibrary" :key="item.id" :value="item.id">
                    {{ item.label }}
                  </option>
                </select>
                <div class="row wrap">
                  <button class="btn ghost" :disabled="!canUseEmailPluginAction('applyTemplate')" @click="applySelectedTemplate">套用模板</button>
                  <button class="btn ghost" :disabled="!canUseEmailPluginAction('applyTemplate')" @click="restoreTemplateDraft">恢复模板正文</button>
                </div>
              </div>
            </div>
            <div class="field">
              <label>收件预设</label>
              <div class="row wrap">
                <button
                  v-for="item in recipientPresets"
                  :key="item.value"
                  class="tag action-tag"
                  :disabled="!canUseEmailPluginAction('applyTemplate')"
                  type="button"
                  @click="useRecipientPreset(item.value)"
                >
                  {{ item.label }} · {{ item.value }}
                </button>
              </div>
            </div>
            <div class="toolbar-grid">
              <div class="field">
                <label>目标邮箱</label>
                <input v-model.trim="form.to" placeholder="ops-team@gaa.local" />
              </div>
              <div class="field">
                <label>邮件标题</label>
                <input v-model.trim="form.subject" placeholder="请输入邮件标题" />
              </div>
            </div>
            <div class="field">
              <label>邮件内容</label>
              <textarea v-model.trim="form.body" rows="10" placeholder="请输入邮件正文"></textarea>
            </div>
            <div v-if="validationIssues.length > 0" class="state-banner error">
              <strong>发送前请修正：</strong>
              <ul>
                <li v-for="item in validationIssues" :key="item">{{ item }}</li>
              </ul>
            </div>
            <div class="row wrap">
              <button class="btn ghost" :disabled="sending || validationIssues.length > 0 || !canUseEmailPluginAction('test')" @click="sendTest">
                {{ sending ? "发送中..." : "发送测试邮件" }}
              </button>
              <button class="btn primary" :disabled="sending || validationIssues.length > 0 || !canUseEmailPluginAction('send')" @click="sendEmail">
                {{ sending ? "发送中..." : "发送邮件" }}
              </button>
            </div>
          </div>
          <div class="card">
            <h3 class="title">模板 / 本地预设</h3>
            <div class="signal-list">
              <div v-for="item in templateLibrary" :key="item.id" class="signal-item">
                <div class="row between wrap">
                  <strong>{{ item.label }}</strong>
                  <button class="btn ghost small" :disabled="!canUseEmailPluginAction('applyTemplate')" @click="applyTemplate(item.id)">套用</button>
                </div>
                <p>{{ item.description }}</p>
                <p class="subtitle">推荐收件人：{{ item.recommendedTo }}</p>
              </div>
            </div>
            <div class="top-gap">
              <div class="row between wrap">
                <strong>本地预设（{{ localPresets.length }}）</strong>
                <span class="subtitle">仅保存在当前浏览器</span>
              </div>
              <div v-if="localPresets.length === 0" class="state-banner top-gap">
                <strong>暂无本地预设。</strong>
                <span>可先套用模板，再将当前草稿保存为浏览器本地预设。</span>
              </div>
              <div v-else class="signal-list top-gap">
                <div v-for="item in localPresets" :key="item.id" class="signal-item">
                  <div class="row between wrap">
                    <strong>{{ item.name }}</strong>
                    <div class="row wrap">
                      <button class="btn ghost small" :disabled="!canUseEmailPluginAction('preset')" @click="applyLocalPreset(item)">载入</button>
                      <button class="btn ghost small" :disabled="!canUseEmailPluginAction('preset')" @click="removeLocalPreset(item)">删除</button>
                    </div>
                  </div>
                  <p>{{ item.subject }}</p>
                  <p class="subtitle">{{ item.to }} · {{ formatTime(item.savedAt) }}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </TabPane>
      <TabPane key="records" tab="邮件收发">
        <div class="card">
          <h3 class="title">邮件记录</h3>
          <div class="data-table">
            <table>
              <thead>
                <tr>
                  <th>时间</th>
                  <th>收件人</th>
                  <th>标题</th>
                  <th>模式</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in records" :key="item.ID">
                  <td>{{ formatTime(item.createdAt) }}</td>
                  <td>{{ item.to }}</td>
                  <td>{{ item.subject }}</td>
                  <td>{{ item.mode }}</td>
                  <td>{{ item.status }}</td>
                  <td>
                    <button class="btn ghost" :disabled="!canUseEmailPluginAction('copy')" @click="copyRecord(item)">复制摘要</button>
                  </td>
                </tr>
                <tr v-if="records.length === 0">
                  <td colspan="6">暂无邮件记录</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </TabPane>
      <TabPane key="config" tab="邮件配置">
        <div class="card">
          <h3 class="title">邮件配置</h3>
          <div class="toolbar-grid">
            <div class="field">
              <label>接收者邮箱</label>
              <input v-model.trim="emailConfig.to" placeholder="多个邮箱可用逗号分隔" />
            </div>
            <div class="field">
              <label>发送者邮箱</label>
              <input v-model.trim="emailConfig.from" placeholder="请输入发送者邮箱" />
            </div>
            <div class="field">
              <label>Host</label>
              <input v-model.trim="emailConfig.host" placeholder="smtp.example.com" />
            </div>
            <div class="field">
              <label>端口</label>
              <input v-model.number="emailConfig.port" type="number" placeholder="465" />
            </div>
            <div class="field">
              <label>昵称</label>
              <input v-model.trim="emailConfig.nickname" placeholder="请输入昵称" />
            </div>
            <div class="field">
              <label>Secret</label>
              <input
                v-model.trim="emailConfig.secret"
                type="password"
                autocomplete="new-password"
                placeholder="请输入密钥"
              />
            </div>
            <div class="field">
              <label>SSL</label>
              <select v-model="emailConfig.isSSL">
                <option :value="true">true</option>
                <option :value="false">false</option>
              </select>
            </div>
            <div class="field">
              <label>LoginAuth</label>
              <select v-model="emailConfig.isLoginAuth">
                <option :value="true">true</option>
                <option :value="false">false</option>
              </select>
            </div>
          </div>
          <div class="mt-4 flex justify-end">
            <button class="btn primary" @click="saveEmailConfig">保存配置</button>
          </div>
        </div>
      </TabPane>
    </Tabs>
  </Page>
</template>
