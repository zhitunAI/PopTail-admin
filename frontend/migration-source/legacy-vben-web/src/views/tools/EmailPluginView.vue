<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">邮件插件工作台</h3>
          <p class="subtitle">对齐原系统邮件插件入口，支持测试邮件、正式发送与发送记录回看。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="loadAll">刷新</button>
          <button class="btn ghost" :disabled="!records.length" @click="copyLatest">复制最近记录</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>发送记录</h4>
        <p class="stat-value">{{ records.length }}</p>
        <p class="subtitle">测试与正式邮件合并展示</p>
      </div>
      <div class="stat-card">
        <h4>最近模式</h4>
        <p class="stat-value">{{ records[0]?.mode || "-" }}</p>
        <p class="subtitle">{{ records[0]?.status || "暂无记录" }}</p>
      </div>
      <div class="stat-card">
        <h4>当前收件人</h4>
        <p class="stat-value">{{ form.to || "-" }}</p>
        <p class="subtitle">建议先使用测试邮箱验证链路</p>
      </div>
      <div class="stat-card">
        <h4>建议</h4>
        <p class="stat-value">{{ validationIssues.length ? "待修正" : "可发送" }}</p>
        <p class="subtitle">{{ validationIssues[0] || "当前表单已满足发送条件" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">发送表单</h3>
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
          <textarea v-model.trim="form.body" rows="8" placeholder="请输入邮件正文" />
        </div>
        <div v-if="validationIssues.length" class="state-banner error">
          <strong>发送前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="sending || validationIssues.length > 0" @click="sendTest">
            {{ sending ? "发送中..." : "发送测试邮件" }}
          </button>
          <button class="btn primary" :disabled="sending || validationIssues.length > 0" @click="sendEmail">
            {{ sending ? "发送中..." : "发送邮件" }}
          </button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">发送提示</h3>
        <div class="signal-list">
          <div class="signal-item">
            <strong>测试邮件</strong>
            <p>用于先验证链路通路和模板文本，不影响正式发送记录判断。</p>
          </div>
          <div class="signal-item">
            <strong>正式发送</strong>
            <p>会写入后端发送记录，适合在恢复验收中验证插件链路已接通。</p>
          </div>
          <div class="signal-item">
            <strong>收件建议</strong>
            <p>建议先使用 clean-room 测试邮箱，再切换真实业务收件地址。</p>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="title">发送记录</h3>
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
                <button class="btn ghost" @click="copyRecord(item)">复制摘要</button>
              </td>
            </tr>
            <tr v-if="!records.length">
              <td colspan="6">暂无邮件记录</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { emailTestApi, getEmailListApi, sendEmailApi } from "../../api/admin";
import type { EmailRecord } from "../../types";

const records = ref<EmailRecord[]>([]);
const message = ref("");
const error = ref("");
const loading = ref(false);
const sending = ref(false);
const form = reactive({
  to: "ops-team@gaa.local",
  subject: "GAA 邮件测试",
  body: "当前邮件由 Rust / Vben 重构版邮件插件工作台登记。",
});

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.to.trim()) issues.push("目标邮箱不能为空。");
  if (form.to && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.to.trim())) issues.push("目标邮箱格式不正确。");
  if (!form.subject.trim()) issues.push("邮件标题不能为空。");
  if (!form.body.trim()) issues.push("邮件内容不能为空。");
  return issues;
});

async function loadAll() {
  loading.value = true;
  error.value = "";
  try {
    const result = await getEmailListApi();
    records.value = result.List.sort((a, b) => b.createdAt - a.createdAt);
    message.value = "已刷新邮件记录。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取邮件记录失败";
  } finally {
    loading.value = false;
  }
}

async function sendTest() {
  await dispatch("test");
}

async function sendEmail() {
  await dispatch("send");
}

async function dispatch(mode: "test" | "send") {
  if (validationIssues.value.length) {
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
  } catch (err) {
    error.value = err instanceof Error ? err.message : "发送邮件失败";
  } finally {
    sending.value = false;
  }
}

async function copyLatest() {
  if (records.value[0]) {
    await copyRecord(records.value[0]);
  }
}

async function copyRecord(item: EmailRecord) {
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

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void loadAll();
});
</script>
