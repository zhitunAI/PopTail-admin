<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">断点续传工作台</h3>
          <p class="subtitle">把续传页改为可观测、可推进、可恢复的任务看板，而不是单纯列表展示。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="load">刷新任务</button>
          <button class="btn ghost" :disabled="acting" @click="advance">推进进度</button>
          <button class="btn primary" :disabled="acting" @click="resume">恢复中断任务</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>总任务</h4>
        <p class="stat-value">{{ rows.length }}</p>
        <p class="subtitle">当前续传台账中的任务数</p>
      </div>
      <div class="stat-card">
        <h4>进行中</h4>
        <p class="stat-value">{{ uploadingCount }}</p>
        <p class="subtitle">尚未完成的任务数</p>
      </div>
      <div class="stat-card">
        <h4>已完成</h4>
        <p class="stat-value">{{ completedCount }}</p>
        <p class="subtitle">进度达到 100% 的任务数</p>
      </div>
      <div class="stat-card">
        <h4>平均进度</h4>
        <p class="stat-value">{{ averageProgress }}%</p>
        <p class="subtitle">{{ averageProgress >= 100 ? "全部收敛" : "用于观察推进效果" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">任务列表</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr><th>文件</th><th>进度</th><th>状态</th><th>进度条</th><th>结论</th></tr>
            </thead>
            <tbody>
              <tr
                v-for="item in rows"
                :key="item.ID"
                :class="{ selected: activeTask?.ID === item.ID }"
                @click="activeTask = item"
              >
                <td>{{ item.name }}</td>
                <td>{{ item.progress }}%</td>
                <td>{{ item.status }}</td>
                <td>
                  <div class="progress-track">
                    <div class="progress-fill" :style="{ width: `${item.progress}%` }" />
                  </div>
                </td>
                <td>{{ taskConclusion(item) }}</td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="5">暂无续传任务</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">任务解读</h3>
        <div v-if="activeTask" class="signal-list">
          <div class="signal-item">
            <strong>当前任务</strong>
            <p>{{ activeTask.name }} · {{ activeTask.progress }}% · {{ activeTask.status }}</p>
          </div>
          <div class="signal-item">
            <strong>处理建议</strong>
            <p>{{ taskConclusion(activeTask) }}</p>
          </div>
          <div class="signal-item">
            <strong>下一步</strong>
            <p>{{ taskNextStep(activeTask) }}</p>
          </div>
        </div>
        <div v-else class="signal-item">
          <strong>暂无选中任务</strong>
          <p>点击左侧任意任务可查看当前续传判断与下一步建议。</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  advanceResumeUploadApi,
  getResumeUploadListApi,
  recoverResumeUploadApi,
} from "../../api/admin";
import type { ResumeUploadRecord } from "../../types";

const rows = ref<ResumeUploadRecord[]>([]);
const activeTask = ref<ResumeUploadRecord | null>(null);
const message = ref("");
const error = ref("");
const loading = ref(false);
const acting = ref(false);

const uploadingCount = computed(() => rows.value.filter((item) => item.progress < 100).length);
const completedCount = computed(() => rows.value.filter((item) => item.progress >= 100).length);
const averageProgress = computed(() => {
  if (!rows.value.length) return 0;
  const total = rows.value.reduce((sum, item) => sum + item.progress, 0);
  return Math.round(total / rows.value.length);
});

function taskConclusion(item: ResumeUploadRecord) {
  if (item.progress >= 100) return "该任务已完成，可做收尾验收。";
  if (item.progress >= 80) return "任务即将完成，建议再推进一次并核对状态。";
  if (item.status.includes("recover") || item.status.includes("resume")) return "任务处于恢复链路，建议继续观察。";
  return "任务仍在处理中，可继续推进或主动恢复。";
}

function taskNextStep(item: ResumeUploadRecord) {
  if (item.progress >= 100) return "进入上传完成或示例中心继续检查其他链路。";
  if (item.progress === 0) return "优先尝试恢复中断任务，再继续推进进度。";
  return "先推进一次进度，再确认是否需要执行恢复。";
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const result = await getResumeUploadListApi();
    rows.value = result.List;
    if (!activeTask.value || !rows.value.some((item) => item.ID === activeTask.value?.ID)) {
      activeTask.value = rows.value[0] ?? null;
    }
    message.value = "已刷新断点续传任务。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取续传任务失败";
  } finally {
    loading.value = false;
  }
}

async function advance() {
  acting.value = true;
  error.value = "";
  try {
    const result = await advanceResumeUploadApi();
    rows.value = result.List;
    activeTask.value = rows.value[0] ?? null;
    message.value = "已向后端请求推进续传进度。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "推进续传失败";
  } finally {
    acting.value = false;
  }
}

async function resume() {
  acting.value = true;
  error.value = "";
  try {
    const result = await recoverResumeUploadApi();
    rows.value = result.List;
    activeTask.value = rows.value.find((item) => item.progress < 100) ?? rows.value[0] ?? null;
    message.value = "已向后端请求恢复中断任务。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "恢复续传失败";
  } finally {
    acting.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>

<style scoped>
.progress-track {
  width: 100%;
  height: 10px;
  border-radius: 999px;
  background: rgba(148, 163, 184, 0.25);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #2563eb, #22c55e);
}
</style>
