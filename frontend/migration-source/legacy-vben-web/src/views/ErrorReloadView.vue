<template>
  <div class="stack">
    <div class="card">
      <div class="row between">
        <div>
          <h3 class="title">页面恢复中</h3>
          <p class="subtitle">重新拉取登录态、权限包和目标页面，避免卡在旧内容或失效权限上。</p>
        </div>
        <button class="btn ghost" :disabled="running" @click="runRecovery">
          {{ running ? "恢复中..." : "立即重试" }}
        </button>
      </div>
      <div class="muted-grid">
        <div class="stat-card">
          <h4>当前阶段</h4>
          <p class="stat-value">{{ currentStage }}</p>
        </div>
        <div class="stat-card">
          <h4>目标页</h4>
          <p class="stat-value">{{ destinationLabel }}</p>
        </div>
        <div class="stat-card">
          <h4>倒计时</h4>
          <p class="stat-value">{{ countdownLabel }}</p>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="card">
      <h3 class="title">恢复清单</h3>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>步骤</th>
              <th>状态</th>
              <th>说明</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in steps" :key="item.id">
              <td>{{ item.label }}</td>
              <td>{{ item.status }}</td>
              <td>{{ item.detail }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card">
      <h3 class="title">可用操作</h3>
      <div class="tag-list">
        <button class="btn ghost" :disabled="running" @click="returnBack">返回上一页</button>
        <button class="btn ghost" :disabled="running" @click="goDashboard">前往控制台</button>
        <button class="btn ghost" :disabled="running" @click="openLogin">回到登录页</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

type RecoveryStep = {
  detail: string;
  id: string;
  label: string;
  status: "等待" | "进行中" | "完成" | "跳过" | "失败";
};

const auth = useAuthStore();
const route = useRoute();
const router = useRouter();
const running = ref(false);
const error = ref("");
const message = ref("");
const countdown = ref(3);
const steps = ref<RecoveryStep[]>(createSteps());
let timer: ReturnType<typeof setInterval> | null = null;

const targetName = computed(() => {
  const fromQuery = route.query.redirect?.toString()?.trim();
  if (fromQuery && router.hasRoute(fromQuery)) {
    return fromQuery;
  }
  const authDefault = auth.defaultRouterName;
  if (authDefault && router.hasRoute(authDefault)) {
    return authDefault;
  }
  return "dashboard";
});

const currentStage = computed(
  () => steps.value.find((item) => item.status === "进行中")?.label ?? "等待开始",
);
const destinationLabel = computed(() => targetName.value || "dashboard");
const countdownLabel = computed(() => (running.value ? `${countdown.value}s` : "已暂停"));

function createSteps(): RecoveryStep[] {
  return [
    { id: "session", label: "刷新登录态", status: "等待", detail: "重新校验本地令牌与当前会话。" },
    { id: "policy", label: "拉取权限包", status: "等待", detail: "同步角色、策略路径和动态菜单。" },
    { id: "navigate", label: "返回目标页", status: "等待", detail: "恢复到默认页或指定 redirect 页面。" },
  ];
}

function updateStep(id: string, status: RecoveryStep["status"], detail?: string) {
  steps.value = steps.value.map((item) =>
    item.id === id ? { ...item, status, detail: detail ?? item.detail } : item,
  );
}

function stopTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

async function navigateTarget() {
  updateStep("navigate", "进行中", `准备进入 ${targetName.value}`);
  await router.replace({ name: targetName.value });
  updateStep("navigate", "完成", `已跳转到 ${targetName.value}`);
}

async function runRecovery() {
  running.value = true;
  error.value = "";
  message.value = "正在重新同步当前页面上下文。";
  countdown.value = 3;
  steps.value = createSteps();
  stopTimer();

  try {
    updateStep("session", "进行中");
    await auth.bootstrap();
    updateStep("session", "完成", auth.isLoggedIn ? "会话仍然有效。" : "未检测到有效会话。");

    if (auth.isLoggedIn) {
      updateStep("policy", "进行中");
      await auth.hydrateAccessEnvelope(auth.userInfo ?? undefined);
      updateStep("policy", "完成", `已同步 ${auth.policyPaths.length} 条权限路径。`);
    } else {
      updateStep("policy", "跳过", "当前未登录，跳过权限与菜单刷新。");
    }

    timer = setInterval(async () => {
      countdown.value -= 1;
      if (countdown.value > 0) {
        return;
      }
      stopTimer();
      if (auth.isLoggedIn) {
        await navigateTarget();
      } else {
        updateStep("navigate", "完成", "未登录，转到登录页。");
        await router.replace({ name: "login" });
      }
    }, 1000);
  } catch (err) {
    stopTimer();
    const detail = err instanceof Error ? err.message : "恢复失败";
    updateStep("session", "失败", detail);
    updateStep("policy", "失败", "依赖的会话刷新未成功。");
    updateStep("navigate", "等待", "请手动选择去向后重试。");
    error.value = detail;
    message.value = "自动恢复未完成，请手动重试或改走其他入口。";
    running.value = false;
    return;
  }

  message.value = auth.isLoggedIn
    ? `已同步会话与权限，${countdown.value} 秒后进入 ${targetName.value}。`
    : `当前无有效会话，${countdown.value} 秒后返回登录页。`;
}

async function returnBack() {
  await router.go(-1);
}

async function goDashboard() {
  await router.replace({ name: "dashboard" });
}

async function openLogin() {
  await router.replace({ name: "login" });
}

onMounted(() => {
  void runRecovery();
});

onBeforeUnmount(() => {
  stopTimer();
});
</script>
