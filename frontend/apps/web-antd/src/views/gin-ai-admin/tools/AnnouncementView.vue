<script setup lang="ts">
import type { AnnouncementDataSource, AnnouncementRecord } from "#/types/gin-ai-admin";

import { computed, onMounted, reactive, ref } from "vue";

import { useAccess } from "@vben/access";
import { Page } from "@vben/common-ui";
import { IconifyIcon } from "@vben/icons";
import { useAccessStore } from "@vben/stores";

import { Alert, Button } from "ant-design-vue";

import {
  createAnnouncementApi,
  deleteAnnouncementApi,
  deleteAnnouncementByIdsApi,
  getAnnouncementDataSourceApi,
  getAnnouncementListApi,
  updateAnnouncementApi,
} from "#/api/gin-ai-admin/admin";

type AttachmentMode = "batch" | "manual" | "staging";
type AttachmentDraftRecord = { name: string; url: string };
type AttachmentStagingRecord = {
  key: string;
  name: string;
  sizeLabel: string;
  suggestedPath: string;
};
type AnnouncementButtonAction =
  | "attachment"
  | "batchDelete"
  | "copy"
  | "create"
  | "delete"
  | "edit"
  | "export"
  | "refresh";

const ANNOUNCEMENT_BUTTON_ACCESS_CODES: Record<AnnouncementButtonAction, string[]> = {
  attachment: [
    "添加附件",
    "导入附件",
    "编辑附件",
    "移除附件",
    "复制附件清单",
    "复制链接",
    "btn:添加附件",
    "btn:导入附件",
    "btn:编辑附件",
    "btn:移除附件",
    "btn:复制附件清单",
    "btn:复制链接",
    "btn:announcement:attachment",
    "btn:anInfo:attachment",
    "btn:/system/tools/announcement:attachment",
    "btn:/anInfo:attachment",
  ],
  batchDelete: [
    "批量删除",
    "btn:批量删除",
    "btn:announcement:batchDelete",
    "btn:anInfo:batchDelete",
    "btn:/system/tools/announcement:batchDelete",
    "btn:/anInfo:batchDelete",
  ],
  copy: [
    "复制",
    "复制摘要",
    "btn:复制",
    "btn:复制摘要",
    "btn:announcement:copy",
    "btn:anInfo:copy",
    "btn:/system/tools/announcement:copy",
    "btn:/anInfo:copy",
  ],
  create: [
    "新增公告",
    "新增",
    "btn:新增公告",
    "btn:新增",
    "btn:announcement:create",
    "btn:anInfo:create",
    "btn:/system/tools/announcement:create",
    "btn:/anInfo:create",
  ],
  delete: [
    "删除",
    "删除公告",
    "btn:删除",
    "btn:删除公告",
    "btn:announcement:delete",
    "btn:anInfo:delete",
    "btn:/system/tools/announcement:delete",
    "btn:/anInfo:delete",
  ],
  edit: [
    "保存公告",
    "编辑公告",
    "更新公告",
    "btn:保存公告",
    "btn:编辑公告",
    "btn:更新公告",
    "btn:announcement:edit",
    "btn:anInfo:edit",
    "btn:/system/tools/announcement:edit",
    "btn:/anInfo:edit",
    "btn:announcement:update",
    "btn:anInfo:update",
  ],
  export: [
    "导出",
    "导出选中",
    "btn:导出",
    "btn:导出选中",
    "btn:announcement:export",
    "btn:anInfo:export",
    "btn:/system/tools/announcement:export",
    "btn:/anInfo:export",
  ],
  refresh: [
    "刷新",
    "查询",
    "btn:刷新",
    "btn:查询",
    "btn:announcement:refresh",
    "btn:anInfo:refresh",
    "btn:/system/tools/announcement:refresh",
    "btn:/anInfo:refresh",
  ],
};

const ANNOUNCEMENT_BUTTON_ACCESS_LABELS: Record<AnnouncementButtonAction, string[]> = {
  attachment: ["添加附件", "导入附件", "编辑附件", "移除附件", "复制附件清单", "复制链接", "attachment"],
  batchDelete: ["批量删除", "batchDelete"],
  copy: ["复制", "复制摘要", "copy"],
  create: ["新增公告", "新增", "create"],
  delete: ["删除", "删除公告", "delete"],
  edit: ["保存公告", "编辑公告", "更新公告", "edit", "update"],
  export: ["导出", "导出选中", "export"],
  refresh: ["刷新", "查询", "refresh"],
};

const ANNOUNCEMENT_BUTTON_ACCESS_CODE_PREFIXES = [
  "btn:announcement:",
  "btn:anInfo:",
  "btn:/system/tools/announcement:",
  "btn:/anInfo:",
];

const announcements = ref<AnnouncementRecord[]>([]);
const selectedAnnouncement = ref<AnnouncementRecord | null>(null);
const dataSource = ref<AnnouncementDataSource>({ userID: [] });
const selectedIds = ref<number[]>([]);
const loading = ref(false);
const saving = ref(false);
const message = ref("");
const error = ref("");
const accessStore = useAccessStore();
const { hasAccessByCodes } = useAccess();
const filters = reactive({ title: "" });
const form = reactive({
  ID: undefined as number | undefined,
  title: "",
  content: "",
  userID: 1,
  attachments: [] as AttachmentDraftRecord[],
});
const attachmentMode = ref<AttachmentMode>("manual");
const attachmentDraft = reactive({ name: "", url: "" });
const attachmentImportText = ref("");
const attachmentStaging = ref<AttachmentStagingRecord[]>([]);

const attachmentCount = computed(() => announcements.value.reduce((sum, item) => sum + item.attachments.length, 0));
const allSelected = computed(
  () => announcements.value.length > 0 && announcements.value.every((item) => selectedIds.value.includes(item.ID)),
);
const selectedRecords = computed(() =>
  announcements.value.filter((item) => selectedIds.value.includes(item.ID)),
);
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.title.trim()) issues.push("公告标题不能为空。");
  if (!form.content.trim()) issues.push("公告内容不能为空。");
  if (!form.userID) issues.push("请先选择作者。");
  return issues;
});
const attachmentDraftIssue = computed(() => {
  const hasName = !!attachmentDraft.name.trim();
  const normalizedUrl = normalizeAttachmentUrl(attachmentDraft.url);
  if (!hasName && !normalizedUrl) return "";
  if (!normalizedUrl) return "请补充附件链接。";
  if (!isSupportedAttachmentUrl(normalizedUrl)) {
    return "附件链接仅支持 https://、/uploads/...、oss://...、s3://... 等可核对地址。";
  }
  return "";
});
const attachmentBatchPreview = computed(() => parseAttachmentImportText(attachmentImportText.value).items.slice(0, 3));
const selectedAdvice = computed(() =>
  selectedAnnouncement.value?.attachments.length
    ? "该公告带附件，建议联动核对链接是否仍然有效。"
    : "该公告无附件，可继续核对正文和作者信息。",
);
const currentAnnouncementSaveAction = computed<"create" | "edit">(() => (form.ID ? "edit" : "create"));
const announcementButtonAccessCandidates = new Set<string>(
  Object.values(ANNOUNCEMENT_BUTTON_ACCESS_CODES).flat(),
);
const announcementButtonAccessLabels = Object.values(ANNOUNCEMENT_BUTTON_ACCESS_LABELS).flat();
const hasAnnouncementButtonAccessEnvelope = computed(() =>
  accessStore.accessCodes.some(
    (code) =>
      announcementButtonAccessCandidates.has(code) ||
      ANNOUNCEMENT_BUTTON_ACCESS_CODE_PREFIXES.some((prefix) => code.startsWith(prefix)) ||
      announcementButtonAccessLabels.some(
        (label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`),
      ),
  ),
);
const canSaveAnnouncement = computed(() => canUseAnnouncementAction(currentAnnouncementSaveAction.value));

function hasFlexibleAnnouncementButtonAccess(action: AnnouncementButtonAction) {
  if (hasAccessByCodes(ANNOUNCEMENT_BUTTON_ACCESS_CODES[action])) {
    return true;
  }
  const labels = ANNOUNCEMENT_BUTTON_ACCESS_LABELS[action];
  return accessStore.accessCodes.some((code) =>
    labels.some((label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`)),
  );
}

function canUseAnnouncementAction(action: AnnouncementButtonAction) {
  return !hasAnnouncementButtonAccessEnvelope.value || hasFlexibleAnnouncementButtonAccess(action);
}

function denyAnnouncementAction(
  action: AnnouncementButtonAction,
  label = ANNOUNCEMENT_BUTTON_ACCESS_LABELS[action][0],
) {
  if (canUseAnnouncementAction(action)) {
    return false;
  }
  error.value = `无按钮权限，当前账号不能执行「${label}」操作。`;
  return true;
}

async function loadAll(showMessage = false) {
  if (showMessage && denyAnnouncementAction("refresh", "刷新")) {
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const [list, source] = await Promise.all([getAnnouncementListApi(filters), getAnnouncementDataSourceApi()]);
    announcements.value = list.List.toSorted((a, b) => b.CreatedAt - a.CreatedAt);
    selectedIds.value = selectedIds.value.filter((id) => announcements.value.some((item) => item.ID === id));
    dataSource.value = source;
    if (!selectedAnnouncement.value && announcements.value[0]) {
      selectAnnouncement(announcements.value[0]);
    }
    message.value = "已刷新公告列表与作者数据源。";
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "获取公告失败";
  } finally {
    loading.value = false;
  }
}

function authorLabel(userID: number) {
  return dataSource.value.userID.find((item) => item.value === userID)?.label || `用户 ${userID}`;
}

function selectAnnouncement(item: AnnouncementRecord) {
  selectedAnnouncement.value = item;
  form.ID = item.ID;
  form.title = item.title;
  form.content = item.content;
  form.userID = item.userID;
  form.attachments = item.attachments.map((entry) => ({ ...entry }));
  attachmentImportText.value = "";
  attachmentStaging.value = [];
  resetAttachmentDraft();
}

function openCreate() {
  selectedAnnouncement.value = null;
  form.ID = undefined;
  form.title = "";
  form.content = "";
  form.userID = dataSource.value.userID[0]?.value ?? 1;
  form.attachments = [];
  attachmentImportText.value = "";
  attachmentStaging.value = [];
  resetAttachmentDraft();
  message.value = "已打开新公告草稿。";
}

function resetAttachmentDraft() {
  attachmentMode.value = "manual";
  attachmentDraft.name = "";
  attachmentDraft.url = "";
}

function toggleAnnouncementSelection(id: number, checked: boolean) {
  if (checked) {
    selectedIds.value = [...new Set([...selectedIds.value, id])];
    return;
  }
  selectedIds.value = selectedIds.value.filter((item) => item !== id);
}

function toggleSelectAll(checked: boolean) {
  selectedIds.value = checked ? announcements.value.map((item) => item.ID) : [];
}

function normalizeAttachmentUrl(value: string) {
  const normalized = value.trim();
  if (!normalized) return "";
  if (normalized.startsWith("www.")) return `https://${normalized}`;
  return normalized;
}

function isSupportedAttachmentUrl(value: string) {
  return /^(https?:\/\/|\/|oss:\/\/|s3:\/\/)/i.test(value);
}

function inferAttachmentName(url: string) {
  const normalized = normalizeAttachmentUrl(url).split(/[?#]/)[0] ?? "";
  const segment = normalized
    .split("/")
    .map((item) => item.trim())
    .findLast(Boolean);
  if (!segment) {
    return `附件 ${form.attachments.length + 1}`;
  }
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

function upsertAttachment(record: AttachmentDraftRecord) {
  const existingIndex = form.attachments.findIndex((item) => item.url === record.url);
  if (existingIndex !== -1) {
    form.attachments.splice(existingIndex, 1, record);
    message.value = `已更新附件：${record.name}`;
    return;
  }
  form.attachments.push(record);
  message.value = `已添加附件：${record.name}`;
}

function appendAttachment() {
  if (denyAnnouncementAction("attachment", "添加附件")) {
    return;
  }
  const url = normalizeAttachmentUrl(attachmentDraft.url);
  const name = attachmentDraft.name.trim() || inferAttachmentName(url);
  if (!url) {
    error.value = "请补充附件链接。";
    return;
  }
  if (!isSupportedAttachmentUrl(url)) {
    error.value = "附件链接仅支持 https://、/uploads/...、oss://...、s3://... 等可核对地址。";
    return;
  }
  upsertAttachment({ name, url });
  error.value = "";
  resetAttachmentDraft();
}

function parseAttachmentImportText(text: string) {
  const items: AttachmentDraftRecord[] = [];
  const skipped: string[] = [];
  for (const [index, rawLine] of text.split(/\r?\n/).entries()) {
    const line = rawLine.trim();
    if (!line) continue;
    let rawName = "";
    let rawUrl: string;
    if (line.includes("|")) {
      const [namePart, ...urlParts] = line.split("|");
      rawName = namePart?.trim() ?? "";
      rawUrl = urlParts.join("|").trim();
    } else if (line.includes("\t")) {
      const [namePart, ...urlParts] = line.split("\t");
      rawName = namePart?.trim() ?? "";
      rawUrl = urlParts.join("\t").trim();
    } else {
      rawUrl = line;
    }
    const url = normalizeAttachmentUrl(rawUrl);
    if (!url || !isSupportedAttachmentUrl(url)) {
      skipped.push(`第 ${index + 1} 行缺少可核对链接`);
      continue;
    }
    items.push({
      name: rawName || inferAttachmentName(url),
      url,
    });
  }
  return { items, skipped };
}

function importAttachments() {
  if (denyAnnouncementAction("attachment", "导入附件")) {
    return;
  }
  const { items, skipped } = parseAttachmentImportText(attachmentImportText.value);
  if (items.length === 0) {
    error.value = skipped[0] ?? "没有可导入的附件链接。";
    return;
  }
  for (const item of items) {
    upsertAttachment(item);
  }
  attachmentImportText.value = "";
  error.value = skipped[0] ?? "";
  message.value = `已导入 ${items.length} 条附件链接${skipped.length > 0 ? `，另有 ${skipped.length} 条需手动修正` : ""}。`;
}

function formatFileSize(value: number) {
  if (value >= 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(1)} MB`;
  if (value >= 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${value} B`;
}

function createSuggestedAttachmentPath(name: string) {
  return `/uploads/announcements/${encodeURIComponent(name).replaceAll('%20', "+")}`;
}

function stageAttachmentFiles(event: Event) {
  if (denyAnnouncementAction("attachment", "添加附件")) {
    return;
  }
  const input = event.target as HTMLInputElement;
  const files = [...input.files ?? []];
  attachmentStaging.value = files.map((file, index) => ({
    key: `${file.name}-${file.size}-${index}`,
    name: file.name,
    sizeLabel: formatFileSize(file.size),
    suggestedPath: createSuggestedAttachmentPath(file.name),
  }));
  if (attachmentStaging.value.length > 0) {
    message.value = `已登记 ${attachmentStaging.value.length} 个本地文件，请补充真实上传路径后再加入公告。`;
  }
  input.value = "";
}

function useStagedAttachment(item: AttachmentStagingRecord) {
  if (denyAnnouncementAction("attachment", "编辑附件")) {
    return;
  }
  attachmentMode.value = "manual";
  attachmentDraft.name = item.name;
  attachmentDraft.url = item.suggestedPath;
  message.value = `已将 ${item.name} 带入手动链接草稿，请确认真实可访问地址后再保存。`;
}

function clearAttachmentStaging() {
  if (denyAnnouncementAction("attachment", "移除附件")) {
    return;
  }
  attachmentStaging.value = [];
  message.value = "已清空本地文件暂存。";
}

function editAttachment(index: number) {
  if (denyAnnouncementAction("attachment", "编辑附件")) {
    return;
  }
  const current = form.attachments[index];
  if (!current) return;
  attachmentMode.value = "manual";
  attachmentDraft.name = current.name;
  attachmentDraft.url = current.url;
  form.attachments.splice(index, 1);
  message.value = `已将 ${current.name} 带回草稿，可修改后重新添加。`;
}

function removeAttachment(index: number) {
  if (denyAnnouncementAction("attachment", "移除附件")) {
    return;
  }
  form.attachments.splice(index, 1);
}

async function copyAttachmentUrl(item: AttachmentDraftRecord) {
  if (denyAnnouncementAction("attachment", "复制链接")) {
    return;
  }
  try {
    await navigator.clipboard.writeText(item.url);
    message.value = `已复制附件链接：${item.name}`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", item.url);
    message.value = "已切换为手动复制附件链接。";
  }
}

async function copyAttachmentChecklist() {
  if (denyAnnouncementAction("attachment", "复制附件清单")) {
    return;
  }
  if (form.attachments.length === 0) return;
  const text = form.attachments.map((item, index) => `${index + 1}. ${item.name} | ${item.url}`).join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = "已复制附件清单。";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制附件清单。";
  }
}

function resetForm() {
  if (selectedAnnouncement.value) {
    selectAnnouncement(selectedAnnouncement.value);
    return;
  }
  openCreate();
}

async function saveAnnouncement() {
  if (
    denyAnnouncementAction(
      currentAnnouncementSaveAction.value,
      currentAnnouncementSaveAction.value === "create" ? "新增公告" : "保存公告",
    )
  ) {
    return;
  }
  if (validationIssues.value.length > 0) {
    error.value = validationIssues.value[0] ?? "公告信息不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    const payload = {
      ID: form.ID,
      title: form.title.trim(),
      content: form.content.trim(),
      userID: form.userID,
      attachments: form.attachments.map((item) => ({ ...item })),
    };
    const saved = form.ID ? await updateAnnouncementApi(payload) : await createAnnouncementApi(payload);
    message.value = `公告 ${saved.title} 已保存`;
    await loadAll();
    const current = announcements.value.find((item) => item.ID === saved.ID);
    if (current) selectAnnouncement(current);
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "保存公告失败";
  } finally {
    saving.value = false;
  }
}

async function removeAnnouncement(id: number) {
  if (denyAnnouncementAction("delete", "删除公告")) {
    return;
  }
  await deleteAnnouncementApi(id);
  selectedIds.value = selectedIds.value.filter((item) => item !== id);
  message.value = `公告 #${id} 已删除`;
  if (selectedAnnouncement.value?.ID === id) {
    selectedAnnouncement.value = null;
  }
  await loadAll();
}

async function removeSelectedAnnouncements() {
  if (denyAnnouncementAction("batchDelete", "批量删除")) {
    return;
  }
  if (selectedIds.value.length === 0) {
    return;
  }
  await deleteAnnouncementByIdsApi(selectedIds.value);
  if (selectedAnnouncement.value && selectedIds.value.includes(selectedAnnouncement.value.ID)) {
    selectedAnnouncement.value = null;
  }
  message.value = `已批量删除 ${selectedIds.value.length} 条公告。`;
  selectedIds.value = [];
  await loadAll();
}

async function copySummary() {
  if (denyAnnouncementAction("copy", "复制摘要")) {
    return;
  }
  if (!selectedAnnouncement.value) return;
  const text = [
    `标题：${selectedAnnouncement.value.title}`,
    `作者：${authorLabel(selectedAnnouncement.value.userID)}`,
    `附件：${selectedAnnouncement.value.attachments.length}`,
    `创建时间：${formatTime(selectedAnnouncement.value.CreatedAt)}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = "已复制公告摘要。";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制公告摘要。";
  }
}

function exportSelectedAnnouncements() {
  if (denyAnnouncementAction("export", "导出选中")) {
    return;
  }
  if (selectedRecords.value.length === 0 || typeof window === "undefined") {
    return;
  }
  const content = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      selectedIds: selectedIds.value,
      records: selectedRecords.value,
    },
    null,
    2,
  );
  const blob = new Blob([content], { type: "application/json;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `announcements-${Date.now()}.json`;
  link.click();
  window.URL.revokeObjectURL(url);
  message.value = `已导出 ${selectedRecords.value.length} 条公告。`;
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void loadAll();
});
</script>

<template>
  <Page>
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon class="text-lg" icon="lucide:megaphone" />
        <span>公告管理</span>
      </div>
    </template>
    <template #extra>
      <div class="flex flex-wrap items-center gap-2">
        <Button :loading="loading" :disabled="!canUseAnnouncementAction('refresh')" @click="loadAll(true)">刷新</Button>
        <Button :disabled="!selectedAnnouncement || !canUseAnnouncementAction('copy')" @click="copySummary">复制摘要</Button>
        <Button
          v-if="canUseAnnouncementAction('batchDelete')"
          danger
          :disabled="selectedIds.length === 0"
          @click="removeSelectedAnnouncements"
        >
          批量删除
        </Button>
        <Button :disabled="selectedIds.length === 0 || !canUseAnnouncementAction('export')" @click="exportSelectedAnnouncements">导出选中</Button>
        <Button v-if="canUseAnnouncementAction('create')" type="primary" @click="openCreate">新增公告</Button>
      </div>
    </template>

    <Alert v-if="message" class="mb-4" show-icon type="info" :message="message" />
    <Alert v-if="error" class="mb-4" show-icon type="error" :message="error" />

    <div class="muted-grid">
      <div class="stat-card">
        <h4>公告总数</h4>
        <p class="stat-value">{{ announcements.length }}</p>
        <p class="subtitle">当前公告台账总量</p>
      </div>
      <div class="stat-card">
        <h4>作者数</h4>
        <p class="stat-value">{{ dataSource.userID.length }}</p>
        <p class="subtitle">来自后端数据源</p>
      </div>
      <div class="stat-card">
        <h4>附件数</h4>
        <p class="stat-value">{{ attachmentCount }}</p>
        <p class="subtitle">当前列表内累计附件数</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ selectedAnnouncement?.title || "-" }}</p>
        <p class="subtitle">{{ selectedAnnouncement ? selectedAdvice : "请选择公告查看详情" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <div class="row between wrap">
          <h3 class="title">公告列表</h3>
          <input v-model.trim="filters.title" placeholder="按标题或内容筛选" @keydown.enter="() => loadAll(true)" />
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>
                  <input
                    :checked="allSelected"
                    :indeterminate="selectedIds.length > 0 && !allSelected"
                    type="checkbox"
                    @change="toggleSelectAll(($event.target as HTMLInputElement).checked)"
                  />
                </th>
                <th>时间</th>
                <th>标题</th>
                <th>作者</th>
                <th>附件数</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in announcements"
                :key="item.ID"
                :class="{ selected: selectedAnnouncement?.ID === item.ID }"
                @click="selectAnnouncement(item)"
              >
                <td>
                  <input
                    :checked="selectedIds.includes(item.ID)"
                    type="checkbox"
                    @click.stop
                    @change="toggleAnnouncementSelection(item.ID, ($event.target as HTMLInputElement).checked)"
                  />
                </td>
                <td>{{ formatTime(item.CreatedAt) }}</td>
                <td>{{ item.title }}</td>
                <td>{{ authorLabel(item.userID) }}</td>
                <td>{{ item.attachments.length }}</td>
                <td>
                  <div class="row wrap">
                    <button class="btn ghost" @click.stop="selectAnnouncement(item)">查看</button>
                    <button v-if="canUseAnnouncementAction('delete')" class="btn ghost" @click.stop="removeAnnouncement(item.ID)">删除</button>
                  </div>
                </td>
              </tr>
              <tr v-if="announcements.length === 0">
                <td colspan="6">暂无公告记录</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">{{ form.ID ? "编辑公告" : "新增公告" }}</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>标题</label>
            <input v-model.trim="form.title" />
          </div>
          <div class="field">
            <label>作者</label>
            <select v-model.number="form.userID">
              <option v-for="item in dataSource.userID" :key="item.value" :value="item.value">
                {{ item.label }}
              </option>
            </select>
          </div>
        </div>
        <div class="field">
          <label>内容</label>
          <textarea v-model.trim="form.content" rows="8"></textarea>
        </div>

        <div class="field">
          <label>附件输入方式</label>
          <select v-model="attachmentMode">
            <option value="manual">手动链接</option>
            <option value="batch">批量导入</option>
            <option value="staging">本地文件暂存</option>
          </select>
          <p class="subtitle">当前后端仍只保存 <code>name + url</code>；未接入上传接口前，请补充真实可访问链接。</p>
        </div>

        <div v-if="attachmentMode === 'manual'" class="field">
          <label>手动链接草稿</label>
          <div class="toolbar-grid">
            <input v-model.trim="attachmentDraft.name" :placeholder="attachmentDraft.url ? inferAttachmentName(attachmentDraft.url) : '附件名（可留空自动识别）'" />
            <input v-model.trim="attachmentDraft.url" placeholder="https://... /uploads/... / oss://..." />
          </div>
          <p class="subtitle">可直接粘贴链接；若附件名留空，将自动从链接末段推断。</p>
          <div v-if="attachmentDraftIssue" class="state-banner error top-gap">
            <strong>当前草稿待修正：</strong>
            <span>{{ attachmentDraftIssue }}</span>
          </div>
          <div class="row wrap top-gap">
            <button class="btn ghost" :disabled="!canUseAnnouncementAction('attachment')" @click="appendAttachment">添加附件</button>
            <button class="btn ghost" :disabled="!canUseAnnouncementAction('attachment')" @click="resetAttachmentDraft">清空草稿</button>
          </div>
        </div>

        <div v-else-if="attachmentMode === 'batch'" class="field">
          <label>批量导入附件</label>
          <textarea
            v-model.trim="attachmentImportText"
            rows="6"
            placeholder="每行一个链接，或使用“附件名 | 链接”格式。例如：&#10;发布说明 | https://cdn.example.com/release-note.pdf&#10;/uploads/announcements/checklist.xlsx"
          ></textarea>
          <p class="subtitle">支持纯链接、<code>名称 | 链接</code>、<code>名称 TAB 链接</code> 三种格式。</p>
          <div class="row wrap top-gap">
            <button class="btn ghost" :disabled="!canUseAnnouncementAction('attachment')" @click="importAttachments">导入到当前公告</button>
            <button class="btn ghost" @click="attachmentImportText = ''">清空文本</button>
          </div>
          <div v-if="attachmentBatchPreview.length > 0" class="signal-list top-gap">
            <div v-for="item in attachmentBatchPreview" :key="`${item.name}-${item.url}`" class="signal-item">
              <strong>{{ item.name }}</strong>
              <p class="subtitle break-all">{{ item.url }}</p>
            </div>
          </div>
        </div>

        <div v-else class="field">
          <label>本地文件暂存</label>
          <input multiple :disabled="!canUseAnnouncementAction('attachment')" type="file" @change="stageAttachmentFiles" />
          <p class="subtitle">这里只登记文件名、体积和建议路径，用于 smoke / parity 验证，不会真的上传到后端。</p>
          <div v-if="attachmentStaging.length === 0" class="state-banner top-gap">
            <strong>暂无本地文件暂存。</strong>
            <span>如需真正上传，仍依赖后端补齐附件传输接口。</span>
          </div>
          <div v-else class="signal-list top-gap">
            <div v-for="item in attachmentStaging" :key="item.key" class="signal-item">
              <div class="row between wrap">
                <strong>{{ item.name }}</strong>
                <button class="btn ghost small" :disabled="!canUseAnnouncementAction('attachment')" @click="useStagedAttachment(item)">带入草稿</button>
              </div>
              <p class="subtitle">{{ item.sizeLabel }} · 建议路径：{{ item.suggestedPath }}</p>
            </div>
          </div>
          <div class="row wrap top-gap">
            <button
              class="btn ghost"
              :disabled="attachmentStaging.length === 0 || !canUseAnnouncementAction('attachment')"
              @click="clearAttachmentStaging"
            >
              清空暂存
            </button>
          </div>
        </div>

        <div class="field">
          <div class="row between wrap">
            <label>当前附件（{{ form.attachments.length }}）</label>
            <button
              class="btn ghost small"
              :disabled="form.attachments.length === 0 || !canUseAnnouncementAction('attachment')"
              @click="copyAttachmentChecklist"
            >
              复制附件清单
            </button>
          </div>
          <div v-if="form.attachments.length === 0" class="state-banner">
            <strong>暂无附件。</strong>
            <span>可使用手动链接、批量导入或本地文件暂存来补齐附件信息。</span>
          </div>
          <div v-else class="signal-list top-gap">
            <div v-for="(item, index) in form.attachments" :key="`${item.name}-${item.url}-${index}`" class="signal-item">
              <div class="row between wrap">
                <strong>{{ item.name }}</strong>
                <div class="row wrap">
                  <a class="btn ghost small" :href="item.url" rel="noopener noreferrer" target="_blank">打开</a>
                  <button class="btn ghost small" :disabled="!canUseAnnouncementAction('attachment')" @click="copyAttachmentUrl(item)">复制链接</button>
                  <button class="btn ghost small" :disabled="!canUseAnnouncementAction('attachment')" @click="editAttachment(index)">编辑</button>
                  <button class="btn ghost small" :disabled="!canUseAnnouncementAction('attachment')" @click="removeAttachment(index)">移除</button>
                </div>
              </div>
              <p class="subtitle break-all">{{ item.url }}</p>
            </div>
          </div>
        </div>

        <div v-if="validationIssues.length > 0" class="state-banner error">
          <strong>保存前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="saving || validationIssues.length > 0 || !canSaveAnnouncement" @click="saveAnnouncement">
            {{ saving ? "保存中..." : "保存公告" }}
          </button>
          <button class="btn ghost" @click="resetForm">重置</button>
        </div>
      </div>
    </div>
  </Page>
</template>
