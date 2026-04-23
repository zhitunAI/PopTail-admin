<template>
  <div class="stack">
    <div class="card">
      <div class="header-row">
        <div>
          <h1 class="title">扫码上传会话工作台</h1>
          <p class="subtitle">以扫码落地页视角呈现会话状态、入口上下文、上传队列与移动端操作提示，而不只是简单切换文案。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" @click="copyEntryUrl">复制入口链接</button>
          <button class="btn ghost" :disabled="busy" @click="copySessionSummary">复制会话摘要</button>
          <button class="btn ghost" :disabled="busy" @click="loadSession">刷新会话</button>
        </div>
      </div>

      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>

      <div class="muted-grid">
        <div class="stat-card">
          <h4>会话编号</h4>
          <p class="stat-value compact">{{ session.sessionId || "-" }}</p>
          <p class="subtitle">扫码页使用的实际会话标识</p>
        </div>
        <div class="stat-card">
          <h4>当前阶段</h4>
          <p class="stat-value">{{ session.status }}</p>
          <p class="subtitle">{{ sessionAdvice }}</p>
        </div>
        <div class="stat-card">
          <h4>上传目标</h4>
          <p class="stat-value">{{ classLabel }}</p>
          <p class="subtitle">来自扫码入口中的 classId</p>
        </div>
        <div class="stat-card">
          <h4>待传文件</h4>
          <p class="stat-value">{{ files.length }}</p>
          <p class="subtitle">仅接受图片文件，单文件上限 8MB</p>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">会话上下文</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><th>入口链接</th><td class="mono">{{ entryUrl }}</td></tr>
              <tr><th>访问令牌</th><td class="mono">{{ maskedToken }}</td></tr>
              <tr><th>会话时间戳</th><td>{{ issuedAtLabel }}</td></tr>
              <tr><th>建议过期时间</th><td>{{ expiresAtLabel }}</td></tr>
              <tr><th>入口状态</th><td>{{ hasRouteContext ? "来自扫码链接" : "直接打开页面" }}</td></tr>
              <tr><th>入口风险</th><td>{{ routeAdvice }}</td></tr>
            </tbody>
          </table>
        </div>

        <div class="step-list">
          <div
            v-for="(step, index) in steps"
            :key="step.status"
            class="step-item"
            :class="{ active: index <= currentStepIndex }"
          >
            <span class="step-index">{{ index + 1 }}</span>
            <div>
              <strong>{{ step.label }}</strong>
              <p class="subtitle">{{ step.desc }}</p>
            </div>
          </div>
        </div>

        <div class="row wrap">
          <button class="btn ghost" :disabled="busy" @click="markScanned">确认已扫码</button>
          <button class="btn ghost" :disabled="busy || !files.length" @click="markReadyToUpload">标记已选文件</button>
          <button class="btn primary" :disabled="busy || !files.length || session.status === '上传完成'" @click="finishUpload">
            完成上传
          </button>
          <button class="btn ghost" :disabled="busy" @click="resetSession">重置会话</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">扫码端上传区</h3>
        <p class="subtitle">选择图片后会将会话推进到“待上传”，完成后写回扫码会话状态。</p>
        <div class="field">
          <label>选择图片</label>
          <input accept="image/*" multiple type="file" @change="onPickFiles" />
        </div>

        <div class="tag-list">
          <span class="tag">仅支持图片</span>
          <span class="tag">单文件 ≤ 8MB</span>
          <span class="tag">累计 {{ totalSizeLabel }}</span>
        </div>

        <div class="preview-grid">
          <div v-for="file in files" :key="file.id" class="preview-card">
            <img v-if="file.previewUrl" :src="file.previewUrl" :alt="file.name" />
            <div class="preview-meta">
              <strong>{{ file.name }}</strong>
              <span>{{ file.sizeLabel }}</span>
              <small>{{ fileAdvice(file) }}</small>
            </div>
            <button class="btn ghost small" @click="removeFile(file.id)">移除</button>
          </div>
          <div v-if="!files.length" class="empty-panel">尚未选择待上传图片</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { getScanSessionApi, saveScanSessionApi } from "../../api/admin";
import type { ScanSessionRecord } from "../../types";

interface ScanDraftFile {
  id: string;
  file: File;
  name: string;
  sizeLabel: string;
  previewUrl: string;
}

const route = useRoute();
const busy = ref(false);
const message = ref("");
const error = ref("");
const files = ref<ScanDraftFile[]>([]);
const session = ref<ScanSessionRecord>({
  sessionId: "",
  status: "待扫码",
});

const steps = [
  { status: "待扫码", label: "等待设备扫码", desc: "入口已生成，但尚未由移动端确认。" },
  { status: "已扫码，待选择文件", label: "扫码已确认", desc: "页面已进入会话，可继续选择需要上传的图片。" },
  { status: "已选择文件，待上传", label: "文件已就绪", desc: "移动端已选择文件，等待执行上传确认。" },
  { status: "上传完成", label: "上传已完成", desc: "扫码端文件已提交，页面可关闭或重新发起会话。" },
];

const classId = computed(() => {
  const raw = route.query.id;
  return Array.isArray(raw) ? raw[0] ?? "" : typeof raw === "string" ? raw : "";
});
const rawToken = computed(() => {
  const raw = route.query.token;
  return Array.isArray(raw) ? raw[0] ?? "" : typeof raw === "string" ? raw : "";
});
const entryTimestamp = computed(() => {
  const raw = route.query.t;
  const value = Array.isArray(raw) ? raw[0] : raw;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
});

const hasRouteContext = computed(() => Boolean(classId.value || rawToken.value || entryTimestamp.value));
const derivedSessionId = computed(() => (entryTimestamp.value ? `scan-${classId.value || "default"}-${entryTimestamp.value}` : ""));
const classLabel = computed(() => classId.value || "未指定");
const maskedToken = computed(() => {
  if (!rawToken.value) return "未携带";
  if (rawToken.value.length <= 8) return rawToken.value;
  return `${rawToken.value.slice(0, 4)}****${rawToken.value.slice(-4)}`;
});
const entryUrl = computed(() => (typeof window === "undefined" ? route.fullPath : `${window.location.origin}${route.fullPath}`));
const issuedAtLabel = computed(() => formatDate(entryTimestamp.value));
const expiresAtLabel = computed(() => formatDate(entryTimestamp.value ? entryTimestamp.value + 10 * 60 * 1000 : 0));
const totalSizeLabel = computed(() => {
  const total = files.value.reduce((sum, item) => sum + item.file.size, 0);
  if (!total) return "0 KB";
  return total > 1024 * 1024 ? `${(total / 1024 / 1024).toFixed(1)} MB` : `${Math.ceil(total / 1024)} KB`;
});
const currentStepIndex = computed(() => {
  const index = steps.findIndex((step) => step.status === session.value.status);
  return index >= 0 ? index : 0;
});
const routeAdvice = computed(() => {
  if (!hasRouteContext.value) return "当前不是从扫码入口进入，建议使用真实扫码链接验收。";
  if (!rawToken.value) return "入口未携带 token，建议确认移动端生成链路。";
  return "入口参数完整，可继续验证扫码与上传流程。";
});
const sessionAdvice = computed(() => {
  if (session.value.status === "上传完成") return "当前会话已完成，可复核产物或重置会话。";
  if (session.value.status === "已选择文件，待上传") return "文件已准备就绪，建议继续确认上传。";
  return "当前会话仍在流程中，可继续推进下一步。";
});

async function loadSession() {
  busy.value = true;
  error.value = "";
  try {
    const current = await getScanSessionApi();
    if (derivedSessionId.value) {
      if (current.sessionId === derivedSessionId.value) {
        session.value = current;
      } else {
        session.value = await saveScanSessionApi({
          sessionId: derivedSessionId.value,
          status: "待扫码",
        });
      }
    } else {
      session.value = current;
    }
    message.value = "已同步扫码会话状态";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取扫码会话失败";
  } finally {
    busy.value = false;
  }
}

async function syncStatus(status: string) {
  const sessionId = session.value.sessionId || derivedSessionId.value;
  if (!sessionId) {
    await loadSession();
  }
  busy.value = true;
  error.value = "";
  try {
    session.value = await saveScanSessionApi({
      sessionId: session.value.sessionId || derivedSessionId.value,
      status,
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "更新扫码会话失败";
  } finally {
    busy.value = false;
  }
}

async function markScanned() {
  await syncStatus("已扫码，待选择文件");
  message.value = "扫码已确认，可继续选择上传图片";
}

async function markReadyToUpload() {
  if (!files.value.length) {
    message.value = "请先选择至少一张图片";
    return;
  }
  await syncStatus("已选择文件，待上传");
  message.value = `已登记 ${files.value.length} 个待上传文件`;
}

async function finishUpload() {
  if (!files.value.length) {
    message.value = "没有可上传的图片";
    return;
  }
  await syncStatus("上传完成");
  message.value = `扫码会话 ${session.value.sessionId} 已完成上传`;
}

async function resetSession() {
  files.value.forEach((item) => URL.revokeObjectURL(item.previewUrl));
  files.value = [];
  await syncStatus("待扫码");
  message.value = "会话已重置，可重新扫码并发起上传";
}

async function copyEntryUrl() {
  await copyText(entryUrl.value, "扫码入口链接已复制到剪贴板");
}

async function copySessionSummary() {
  const text = [
    `会话：${session.value.sessionId || "-"}`,
    `阶段：${session.value.status}`,
    `目标：${classLabel.value}`,
    `入口：${entryUrl.value}`,
    `文件数：${files.value.length}`,
  ].join("\n");
  await copyText(text, "扫码会话摘要已复制");
}

async function onPickFiles(event: Event) {
  const input = event.target as HTMLInputElement;
  const selected = Array.from(input.files ?? []);
  const accepted: ScanDraftFile[] = [];

  for (const file of selected) {
    if (!file.type.startsWith("image/")) {
      message.value = `${file.name} 不是图片文件，已跳过`;
      continue;
    }
    if (file.size > 8 * 1024 * 1024) {
      message.value = `${file.name} 超过 8MB，已跳过`;
      continue;
    }
    accepted.push({
      id: `${file.name}-${file.size}-${file.lastModified}`,
      file,
      name: file.name,
      sizeLabel: file.size > 1024 * 1024 ? `${(file.size / 1024 / 1024).toFixed(1)} MB` : `${Math.ceil(file.size / 1024)} KB`,
      previewUrl: URL.createObjectURL(file),
    });
  }

  files.value.forEach((item) => URL.revokeObjectURL(item.previewUrl));
  files.value = accepted;
  input.value = "";

  if (!files.value.length) return;
  if (session.value.status === "待扫码") {
    await markScanned();
  }
  await markReadyToUpload();
}

function fileAdvice(file: ScanDraftFile) {
  return file.file.size > 1024 * 1024 ? "体积较大，建议优先确认网络环境" : "可直接进入上传确认";
}

function removeFile(id: string) {
  const hit = files.value.find((item) => item.id === id);
  if (hit) URL.revokeObjectURL(hit.previewUrl);
  files.value = files.value.filter((item) => item.id !== id);
  message.value = files.value.length ? `已保留 ${files.value.length} 个待上传文件` : "待上传列表已清空";
  if (!files.value.length && session.value.status !== "上传完成") {
    void syncStatus("已扫码，待选择文件");
  }
}

async function copyText(text: string, success: string) {
  try {
    await navigator.clipboard.writeText(text);
    message.value = success;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制。";
  }
}

function formatDate(timestamp: number) {
  if (!timestamp) return "未提供";
  return new Date(timestamp).toLocaleString();
}

onMounted(() => {
  void loadSession();
});

onBeforeUnmount(() => {
  files.value.forEach((item) => URL.revokeObjectURL(item.previewUrl));
});
</script>

<style scoped>
.header-row {
  align-items: flex-start;
  display: flex;
  gap: 16px;
  justify-content: space-between;
}

.mono {
  font-family:
    ui-monospace, SFMono-Regular, SFMono-Regular, Menlo, Monaco, Consolas,
    Liberation Mono, Courier New, monospace;
  word-break: break-all;
}

.compact {
  font-size: 1rem;
}

.step-list {
  display: grid;
  gap: 12px;
  margin-top: 20px;
}

.step-item {
  align-items: flex-start;
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 16px;
  display: flex;
  gap: 12px;
  padding: 14px 16px;
}

.step-item.active {
  background: rgba(37, 99, 235, 0.08);
  border-color: rgba(37, 99, 235, 0.28);
}

.step-index {
  align-items: center;
  background: rgba(15, 23, 42, 0.08);
  border-radius: 999px;
  display: inline-flex;
  font-size: 0.875rem;
  font-weight: 700;
  height: 28px;
  justify-content: center;
  width: 28px;
}

.preview-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-top: 16px;
}

.preview-card {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 16px;
  display: grid;
  gap: 10px;
  overflow: hidden;
  padding: 12px;
}

.preview-card img {
  aspect-ratio: 1 / 1;
  border-radius: 12px;
  object-fit: cover;
  width: 100%;
}

.preview-meta {
  display: grid;
  gap: 4px;
}

.empty-panel {
  align-items: center;
  border: 1px dashed rgba(148, 163, 184, 0.35);
  border-radius: 16px;
  color: #64748b;
  display: flex;
  justify-content: center;
  min-height: 160px;
  padding: 24px;
}

.small {
  min-height: auto;
  padding: 8px 10px;
}

@media (max-width: 900px) {
  .header-row {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
