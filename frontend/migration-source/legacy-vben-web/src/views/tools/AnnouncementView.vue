<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">公告管理工作台</h3>
          <p class="subtitle">对齐原系统公告管理示例，支持公告列表、创建/编辑、附件清单和作者数据源联动。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="loadAll">刷新</button>
          <button class="btn ghost" :disabled="!selectedAnnouncement" @click="copySummary">复制摘要</button>
          <button class="btn primary" @click="openCreate">新增公告</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

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
          <input v-model.trim="filters.title" placeholder="按标题或内容筛选" @keydown.enter="loadAll" />
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr>
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
                <td>{{ formatTime(item.CreatedAt) }}</td>
                <td>{{ item.title }}</td>
                <td>{{ authorLabel(item.userID) }}</td>
                <td>{{ item.attachments.length }}</td>
                <td>
                  <div class="row wrap">
                    <button class="btn ghost" @click.stop="selectAnnouncement(item)">查看</button>
                    <button class="btn ghost" @click.stop="removeAnnouncement(item.ID)">删除</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!announcements.length">
                <td colspan="5">暂无公告记录</td>
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
          <textarea v-model.trim="form.content" rows="8" />
        </div>

        <div class="field">
          <label>附件</label>
          <div class="row wrap">
            <input v-model.trim="attachmentDraft.name" placeholder="附件名" />
            <input v-model.trim="attachmentDraft.url" placeholder="附件链接" />
            <button class="btn ghost" @click="appendAttachment">添加附件</button>
          </div>
          <div class="tag-list top-gap">
            <span v-for="(item, index) in form.attachments" :key="`${item.name}-${index}`" class="tag">
              {{ item.name }}
              <button class="btn ghost small" @click="removeAttachment(index)">移除</button>
            </span>
            <span v-if="!form.attachments.length" class="tag">暂无附件</span>
          </div>
        </div>

        <div v-if="validationIssues.length" class="state-banner error">
          <strong>保存前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="saveAnnouncement">
            {{ saving ? "保存中..." : "保存公告" }}
          </button>
          <button class="btn ghost" @click="resetForm">重置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  createAnnouncementApi,
  deleteAnnouncementApi,
  getAnnouncementDataSourceApi,
  getAnnouncementListApi,
  updateAnnouncementApi,
} from "../../api/admin";
import type { AnnouncementDataSource, AnnouncementRecord } from "../../types";

const announcements = ref<AnnouncementRecord[]>([]);
const selectedAnnouncement = ref<AnnouncementRecord | null>(null);
const dataSource = ref<AnnouncementDataSource>({ userID: [] });
const loading = ref(false);
const saving = ref(false);
const message = ref("");
const error = ref("");
const filters = reactive({ title: "" });
const form = reactive({
  ID: undefined as number | undefined,
  title: "",
  content: "",
  userID: 1,
  attachments: [] as Array<{ name: string; url: string }>,
});
const attachmentDraft = reactive({ name: "", url: "" });

const attachmentCount = computed(() => announcements.value.reduce((sum, item) => sum + item.attachments.length, 0));
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.title.trim()) issues.push("公告标题不能为空。");
  if (!form.content.trim()) issues.push("公告内容不能为空。");
  if (!form.userID) issues.push("请先选择作者。");
  return issues;
});
const selectedAdvice = computed(() =>
  selectedAnnouncement.value?.attachments.length
    ? "该公告带附件，建议联动核对链接是否仍然有效。"
    : "该公告无附件，可继续核对正文和作者信息。",
);

async function loadAll() {
  loading.value = true;
  error.value = "";
  try {
    const [list, source] = await Promise.all([
      getAnnouncementListApi(filters),
      getAnnouncementDataSourceApi(),
    ]);
    announcements.value = list.List.sort((a, b) => b.CreatedAt - a.CreatedAt);
    dataSource.value = source;
    if (!selectedAnnouncement.value && announcements.value[0]) {
      selectAnnouncement(announcements.value[0]);
    }
    message.value = "已刷新公告列表与作者数据源。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取公告失败";
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
}

function openCreate() {
  selectedAnnouncement.value = null;
  form.ID = undefined;
  form.title = "";
  form.content = "";
  form.userID = dataSource.value.userID[0]?.value ?? 1;
  form.attachments = [];
  attachmentDraft.name = "";
  attachmentDraft.url = "";
  message.value = "已打开新公告草稿。";
}

function appendAttachment() {
  if (!attachmentDraft.name.trim() || !attachmentDraft.url.trim()) return;
  form.attachments.push({ name: attachmentDraft.name.trim(), url: attachmentDraft.url.trim() });
  attachmentDraft.name = "";
  attachmentDraft.url = "";
}

function removeAttachment(index: number) {
  form.attachments.splice(index, 1);
}

function resetForm() {
  if (selectedAnnouncement.value) {
    selectAnnouncement(selectedAnnouncement.value);
    return;
  }
  openCreate();
}

async function saveAnnouncement() {
  if (validationIssues.value.length) {
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
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存公告失败";
  } finally {
    saving.value = false;
  }
}

async function removeAnnouncement(id: number) {
  await deleteAnnouncementApi(id);
  message.value = `公告 #${id} 已删除`;
  if (selectedAnnouncement.value?.ID === id) {
    selectedAnnouncement.value = null;
  }
  await loadAll();
}

async function copySummary() {
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

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void loadAll();
});
</script>
