<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">上传示例工作台</h3>
          <p class="subtitle">恢复为真实上传队列工作台：支持追加入队、队列覆盖、逐项移除、完成确认和后台状态回填。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="saving" @click="loadQueue">刷新队列</button>
          <button class="btn ghost" :disabled="!rows.length" @click="copyQueueSummary">复制队列摘要</button>
        </div>
      </div>
      <div class="toolbar-grid">
        <div class="field">
          <label>待入队文件</label>
          <input
            ref="fileInputRef"
            type="file"
            multiple
            :disabled="saving"
            @change="onPick"
          />
        </div>
        <div class="field">
          <label>当前状态</label>
          <p class="subtitle">{{ statusText }}</p>
        </div>
      </div>
      <div class="row wrap">
        <button class="btn ghost" :disabled="saving || !pendingSelection.length" @click="replaceQueue">覆盖队列</button>
        <button class="btn primary" :disabled="saving || !pendingSelection.length" @click="appendQueue">追加到队列</button>
        <button class="btn ghost" :disabled="saving || !rows.length" @click="completeQueue">确认全部上传</button>
        <button class="btn ghost" :disabled="saving || !pendingSelection.length" @click="clearSelection">清空本次选择</button>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>队列文件</h4>
        <p class="stat-value">{{ rows.length }}</p>
        <p class="subtitle">当前后端上传队列中的文件数</p>
      </div>
      <div class="stat-card">
        <h4>待上传</h4>
        <p class="stat-value">{{ pendingCount }}</p>
        <p class="subtitle">尚未确认完成的文件数</p>
      </div>
      <div class="stat-card">
        <h4>已完成</h4>
        <p class="stat-value">{{ completedCount }}</p>
        <p class="subtitle">后台已标记上传完成的文件数</p>
      </div>
      <div class="stat-card">
        <h4>队列体积</h4>
        <p class="stat-value">{{ formattedTotalSize }}</p>
        <p class="subtitle">{{ duplicateCount ? `检测到 ${duplicateCount} 个重复签名` : "当前未发现重复签名" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">本次待入队文件</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>文件名</th>
                <th>大小</th>
                <th>说明</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in pendingSelection" :key="`${item.name}-${item.size}`">
                <td>{{ item.name }}</td>
                <td>{{ item.size }}</td>
                <td>{{ pendingAdvice(item) }}</td>
              </tr>
              <tr v-if="!pendingSelection.length">
                <td colspan="3">尚未选择文件</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">操作说明</h3>
        <div class="signal-list">
          <div class="signal-item">
            <strong>追加</strong>
            <p>保留既有队列，并把本次选择文件补充进去，适合连续补交。</p>
          </div>
          <div class="signal-item">
            <strong>覆盖</strong>
            <p>按本次选择重建队列，适合重新提交批次或移除旧队列污染。</p>
          </div>
          <div class="signal-item">
            <strong>确认全部上传</strong>
            <p>调用完成接口，把全部项标记为上传完成，便于示例链路收口。</p>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="row between wrap">
        <h3 class="title">上传队列</h3>
        <span class="tag">{{ queueHealth }}</span>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>文件</th>
              <th>大小</th>
              <th>状态</th>
              <th>建议</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in rows" :key="item.ID">
              <td>{{ item.ID }}</td>
              <td>{{ item.name }}</td>
              <td>{{ item.size }}</td>
              <td>
                <span class="status-badge" :class="item.status === '上传完成' ? 'done' : 'pending'">
                  {{ item.status }}
                </span>
              </td>
              <td>{{ rowAdvice(item) }}</td>
              <td>
                <div class="row">
                  <button class="btn ghost" :disabled="saving" @click="removeFromQueue(item.ID)">移除</button>
                </div>
              </td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="6">当前暂无上传队列</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  completeUploadQueueApi,
  getUploadQueueApi,
  saveUploadQueueApi,
} from "../../api/admin";
import type { UploadFileRecord } from "../../types";

type QueueDraftFile = {
  name: string;
  size: string;
};

const fileInputRef = ref<HTMLInputElement | null>(null);
const rows = ref<UploadFileRecord[]>([]);
const pendingSelection = ref<QueueDraftFile[]>([]);
const message = ref("");
const error = ref("");
const saving = ref(false);

const pendingCount = computed(() => rows.value.filter((item) => item.status !== "上传完成").length);
const completedCount = computed(() => rows.value.filter((item) => item.status === "上传完成").length);
const totalBytes = computed(() => rows.value.reduce((sum, item) => sum + parseSize(item.size), 0));
const formattedTotalSize = computed(() => formatBytes(totalBytes.value));
const duplicateCount = computed(() => {
  const counter = new Map<string, number>();
  for (const item of rows.value) {
    const key = `${item.name}__${item.size}`;
    counter.set(key, (counter.get(key) ?? 0) + 1);
  }
  return [...counter.values()].filter((count) => count > 1).length;
});
const statusText = computed(() => {
  if (saving.value) return "正在同步后台上传队列…";
  if (pendingSelection.value.length) return `已选 ${pendingSelection.value.length} 个文件，等待写入队列`;
  if (!rows.value.length) return "当前队列为空，可先选择文件后追加或覆盖";
  return `队列中 ${rows.value.length} 项，待上传 ${pendingCount.value} 项`;
});
const queueHealth = computed(() => {
  if (!rows.value.length) return "空队列";
  if (!pendingCount.value) return "已全部完成";
  if (duplicateCount.value) return "需清理重复项";
  return "待继续上传";
});

function mapPickedFiles(fileList: FileList | null) {
  const next = Array.from(fileList ?? []).map((file) => ({
    name: file.name,
    size: formatBytes(file.size),
  }));
  pendingSelection.value = uniqueBySignature(next);
}

function onPick(event: Event) {
  const input = event.target as HTMLInputElement;
  mapPickedFiles(input.files);
  if (pendingSelection.value.length) {
    message.value = `已选择 ${pendingSelection.value.length} 个文件，等待入队`;
  }
}

async function loadQueue() {
  saving.value = true;
  error.value = "";
  try {
    const result = await getUploadQueueApi();
    rows.value = result.List;
    message.value = rows.value.length ? "已从后台刷新上传队列" : "后台上传队列当前为空";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取上传队列失败";
  } finally {
    saving.value = false;
  }
}

async function replaceQueue() {
  if (!pendingSelection.value.length) {
    message.value = "请先选择至少一个文件";
    return;
  }
  await persistQueue("replace", "已按本次选择覆盖上传队列");
}

async function appendQueue() {
  if (!pendingSelection.value.length) {
    message.value = "请先选择至少一个文件";
    return;
  }
  await persistQueue("append", "已将本次文件追加到上传队列");
}

async function persistQueue(mode: "replace" | "append", successText: string) {
  saving.value = true;
  error.value = "";
  try {
    const result = await saveUploadQueueApi({
      mode,
      files: pendingSelection.value,
    });
    rows.value = result.List;
    clearSelection();
    message.value = successText;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存上传队列失败";
  } finally {
    saving.value = false;
  }
}

async function completeQueue() {
  saving.value = true;
  error.value = "";
  try {
    const result = await completeUploadQueueApi();
    rows.value = result.List;
    message.value = rows.value.length ? "后台已将当前队列标记为上传完成" : "当前没有可完成的队列项";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "完成上传队列失败";
  } finally {
    saving.value = false;
  }
}

async function removeFromQueue(id: number) {
  saving.value = true;
  error.value = "";
  try {
    const remaining = rows.value
      .filter((item) => item.ID !== id)
      .map((item) => ({ name: item.name, size: item.size }));
    const result = await saveUploadQueueApi({
      mode: "replace",
      files: remaining,
    });
    rows.value = result.List;
    message.value = `已从后台队列移除文件 #${id}`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "移除队列项失败";
  } finally {
    saving.value = false;
  }
}

async function copyQueueSummary() {
  const text = [
    `队列总数：${rows.value.length}`,
    `待上传：${pendingCount.value}`,
    `已完成：${completedCount.value}`,
    `总大小：${formattedTotalSize.value}`,
    `状态：${queueHealth.value}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = "已复制上传队列摘要。";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制上传队列摘要。";
  }
}

function pendingAdvice(item: QueueDraftFile) {
  return rows.value.some((row) => row.name === item.name && row.size === item.size)
    ? "与当前队列存在同签名文件，追加前请确认是否允许重复"
    : "可安全写入当前上传队列";
}

function rowAdvice(item: UploadFileRecord) {
  if (item.status === "上传完成") return "该文件已完成上传，可留作验收记录";
  return "该文件仍在待传队列，可继续上传或直接移除";
}

function clearSelection() {
  pendingSelection.value = [];
  if (fileInputRef.value) {
    fileInputRef.value.value = "";
  }
}

function uniqueBySignature(files: QueueDraftFile[]) {
  const store = new Map<string, QueueDraftFile>();
  for (const item of files) {
    store.set(`${item.name}__${item.size}`, item);
  }
  return Array.from(store.values());
}

function parseSize(size: string) {
  const match = size.trim().match(/^([\d.]+)\s*(B|KB|MB|GB)$/i);
  if (!match) return 0;
  const value = Number(match[1]);
  const unit = match[2].toUpperCase();
  const unitMap: Record<string, number> = { B: 1, KB: 1024, MB: 1024 ** 2, GB: 1024 ** 3 };
  return Math.round(value * (unitMap[unit] ?? 1));
}

function formatBytes(bytes: number) {
  if (bytes <= 0) return "0 B";
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
  if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(2)} MB`;
  if (bytes >= 1024) return `${Math.ceil(bytes / 1024)} KB`;
  return `${bytes} B`;
}

onMounted(() => {
  void loadQueue();
});
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
}

.status-badge.pending {
  background: rgba(37, 99, 235, 0.12);
  color: #1d4ed8;
}

.status-badge.done {
  background: rgba(34, 197, 94, 0.14);
  color: #15803d;
}
</style>
