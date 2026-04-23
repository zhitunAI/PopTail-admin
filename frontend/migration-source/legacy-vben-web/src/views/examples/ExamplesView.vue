<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">示例中心工作台</h3>
          <p class="subtitle">读取真实后端状态，把客户、上传、续传、扫码四条示例链路汇总为一个验收入口。</p>
        </div>
        <button class="btn ghost" :disabled="loading" @click="reload">
          {{ loading ? "刷新中..." : "刷新概览" }}
        </button>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>客户</h4>
        <p class="stat-value">{{ summary.customers }}</p>
        <p class="subtitle">客户示例台账总数</p>
      </div>
      <div class="stat-card">
        <h4>上传队列</h4>
        <p class="stat-value">{{ summary.uploadPending }}</p>
        <p class="subtitle">当前待处理文件数</p>
      </div>
      <div class="stat-card">
        <h4>断点续传</h4>
        <p class="stat-value">{{ summary.resumeInFlight }}</p>
        <p class="subtitle">进行中的任务数</p>
      </div>
      <div class="stat-card">
        <h4>扫码会话</h4>
        <p class="stat-value">{{ summary.scanStatus }}</p>
        <p class="subtitle">{{ summary.scanAdvice }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">示例入口</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>示例</th>
                <th>状态摘要</th>
                <th>健康度</th>
                <th>入口</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in rows" :key="item.path">
                <td>{{ item.name }}</td>
                <td>{{ item.desc }}</td>
                <td>{{ item.health }}</td>
                <td><RouterLink class="btn ghost" :to="item.path">打开</RouterLink></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">当前检查项</h3>
        <div class="signal-list">
          <div v-for="item in checks" :key="item.title" class="signal-item">
            <div class="row between wrap">
              <strong>{{ item.title }}</strong>
              <RouterLink v-if="item.path" class="btn ghost" :to="item.path">打开</RouterLink>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { RouterLink } from "vue-router";
import {
  getCustomerListApi,
  getResumeUploadListApi,
  getScanSessionApi,
  getUploadQueueApi,
} from "../../api/admin";

const loading = ref(false);
const message = ref("");
const error = ref("");

const summary = reactive({
  customers: 0,
  uploadPending: 0,
  resumeInFlight: 0,
  scanStatus: "-",
  scanAdvice: "待拉取会话状态",
});

const rows = ref<Array<{ desc: string; health: string; name: string; path: string }>>([]);
const checks = ref<Array<{ detail: string; path?: string; title: string }>>([]);

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [customers, uploadQueue, resumeList, scanSession] = await Promise.all([
      getCustomerListApi(),
      getUploadQueueApi(),
      getResumeUploadListApi(),
      getScanSessionApi(),
    ]);

    summary.customers = customers.Total;
    summary.uploadPending = uploadQueue.List.filter((item) => item.status !== "done").length;
    summary.resumeInFlight = resumeList.List.filter((item) => item.progress < 100).length;
    summary.scanStatus = scanSession.status;
    summary.scanAdvice =
      scanSession.status === "active"
        ? "扫码链路可继续验证"
        : scanSession.status === "completed"
          ? "最近会话已完成"
          : "建议重新打开扫码上传页";

    rows.value = [
      {
        name: "客户示例",
        desc: `客户台账 ${customers.Total} 条，支持详情、编辑与筛选`,
        health: customers.Total > 0 ? "已接后端" : "待补数据",
        path: "/examples/customer",
      },
      {
        name: "上传示例",
        desc: `待处理文件 ${summary.uploadPending} 个`,
        health: summary.uploadPending > 0 ? "有任务" : "队列平稳",
        path: "/examples/upload",
      },
      {
        name: "断点续传",
        desc: `进行中 ${summary.resumeInFlight} 项`,
        health: summary.resumeInFlight > 0 ? "需跟进" : "已收敛",
        path: "/examples/breakpoint",
      },
      {
        name: "扫码上传",
        desc: `最近状态：${scanSession.status}`,
        health: scanSession.status === "active" ? "可联调" : "待复查",
        path: "/scan-upload",
      },
    ];

    checks.value = [
      {
        title: "客户链路",
        detail: customers.Total ? "客户示例已接后端，可继续验证增删改查。" : "客户示例当前无数据，建议先补一条测试数据。",
        path: "/examples/customer",
      },
      {
        title: "上传队列",
        detail: summary.uploadPending
          ? `当前仍有 ${summary.uploadPending} 个文件待处理，建议进入上传页确认完成状态。`
          : "上传队列已清空，可直接发起新一轮验收。",
        path: "/examples/upload",
      },
      {
        title: "续传任务",
        detail: summary.resumeInFlight
          ? `还有 ${summary.resumeInFlight} 个断点任务未收敛，建议继续推进或恢复。`
          : "续传任务当前已全部完成。",
        path: "/examples/breakpoint",
      },
      {
        title: "扫码状态",
        detail: `当前扫码会话为 ${scanSession.status}，${summary.scanAdvice}。`,
        path: "/scan-upload",
      },
    ];

    message.value = "已刷新示例链路概览。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取示例中心状态失败";
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void reload();
});
</script>
