<template>
  <div class="page-shell">
    <section class="workspace">
      <div class="main-card">
        <header class="hero">
          <div>
            <p class="eyebrow">登录工作台</p>
            <h1 class="title">GAA Console</h1>
            <p class="subtitle">
              基于现有鉴权流程完成登录、回跳与权限装载。默认演示账号：
              <code>admin</code> / <code>123456</code>
            </p>
          </div>
          <div class="hero-actions">
            <button
              v-if="auth.isLoggedIn"
              class="btn ghost"
              type="button"
              @click="enterCurrentSession"
            >
              进入当前会话
            </button>
            <button class="btn ghost" type="button" @click="fillDemoAdmin">
              演示账号回填
            </button>
          </div>
        </header>

        <div class="stats">
          <article class="stat-card">
            <span class="stat-label">当前会话</span>
            <strong>{{ auth.isLoggedIn ? "已登录" : "待登录" }}</strong>
            <small>
              {{ auth.userInfo?.userName ? `${auth.userInfo.userName} · ${activeRoleLabel}` : "尚未建立会话" }}
            </small>
          </article>
          <article class="stat-card">
            <span class="stat-label">登录去向</span>
            <strong>{{ redirectLabel }}</strong>
            <small>{{ redirectHint }}</small>
          </article>
          <article class="stat-card">
            <span class="stat-label">记忆账号</span>
            <strong>{{ rememberedAccounts.length }}</strong>
            <small>最近成功 {{ successfulPersonaCount }} 个</small>
          </article>
          <article class="stat-card">
            <span class="stat-label">最近结果</span>
            <strong>{{ latestOutcomeLabel }}</strong>
            <small>{{ latestOutcomeHint }}</small>
          </article>
        </div>

        <div :class="['feedback', feedbackTone]">
          <strong>{{ feedbackTitle }}</strong>
          <p>{{ feedbackMessage }}</p>
          <small>{{ feedbackHint }}</small>
        </div>

        <form class="login-form" @submit.prevent="submit">
          <div class="field">
            <label for="username">用户名</label>
            <input
              id="username"
              v-model.trim="username"
              autocomplete="username"
              maxlength="64"
              placeholder="输入用户名"
              required
            />
            <small class="helper">支持回填记忆账号与最近成功人设。</small>
          </div>
          <div class="field">
            <label for="password">密码</label>
            <input
              id="password"
              v-model="password"
              type="password"
              autocomplete="current-password"
              maxlength="128"
              placeholder="输入当前密码"
              required
            />
            <small class="helper">
              本页不做本地鉴权判断，最终以服务端认证结果为准。
            </small>
          </div>

          <div class="options">
            <label class="checkbox">
              <input v-model="rememberAccount" type="checkbox" />
              <span>记住本次账号</span>
            </label>
            <span class="password-hint">
              {{ passwordHint }}
            </span>
          </div>

          <div class="row">
            <button class="btn primary" :disabled="loading || !canSubmit" type="submit">
              {{ loading ? "登录中..." : "进入控制台" }}
            </button>
            <button class="btn ghost" type="button" :disabled="loading" @click="resetForm">
              重置录入
            </button>
          </div>
        </form>

        <section class="panel">
          <div class="panel-header">
            <h2>快捷回填</h2>
            <span>演示账号、当前会话与最近成功账号可直接带入表单</span>
          </div>
          <div class="persona-list">
            <button
              v-for="profile in quickProfiles"
              :key="profile.key"
              class="persona-card"
              type="button"
              @click="applyProfile(profile)"
            >
              <strong>{{ profile.label }}</strong>
              <small>{{ profile.username }}</small>
              <span>{{ profile.note }}</span>
            </button>
          </div>
        </section>

        <section class="panel">
          <div class="panel-header">
            <h2>失败分层提示</h2>
            <span>根据最近一次失败结果，给出下一步排查建议</span>
          </div>
          <div :class="['failure-panel', failureGuide.tone]">
            <strong>{{ failureGuide.title }}</strong>
            <p>{{ failureGuide.message }}</p>
            <ul>
              <li v-for="item in failureGuide.steps" :key="item">{{ item }}</li>
            </ul>
          </div>
        </section>
      </div>

      <aside class="side-card">
        <section class="panel">
          <div class="panel-header">
            <h2>最近登录人设</h2>
            <span>保留最近成功的账号画像，便于切换测试角色</span>
          </div>
          <div v-if="recentPersonas.length" class="memory-list">
            <button
              v-for="account in recentPersonas"
              :key="account.username"
              class="memory-item"
              type="button"
              @click="useRememberedAccount(account)"
            >
              <div>
                <strong>{{ account.label }}</strong>
                <small>{{ account.username }}</small>
              </div>
              <span>{{ formatTime(account.lastUsedAt) }}</span>
            </button>
          </div>
          <p v-else class="empty">
            还没有成功登录记录。完成一次登录后，这里会保留最近的人设入口。
          </p>
        </section>

        <section class="panel">
          <div class="panel-header">
            <h2>记忆账号</h2>
            <span>仅在本地保存用户名、结果与最近时间，不存密码</span>
          </div>
          <div v-if="rememberedAccounts.length" class="memory-list">
            <button
              v-for="account in rememberedAccounts"
              :key="account.username"
              class="memory-item"
              type="button"
              @click="useRememberedAccount(account)"
            >
              <div>
                <strong>{{ account.username }}</strong>
                <small>{{ account.lastMessage || "等待下一次登录结果" }}</small>
              </div>
              <span :class="['status-dot', account.lastStatus]">
                {{ account.lastStatus === "success" ? "成功" : account.lastStatus === "failure" ? "失败" : "待验证" }}
              </span>
            </button>
          </div>
          <p v-else class="empty">勾选“记住本次账号”后，成功或失败结果都会在此记录。</p>
        </section>

        <section class="panel">
          <div class="panel-header">
            <h2>最近登录动态</h2>
            <span>本地记录最近 8 次登录结果，便于排查连续失败</span>
          </div>
          <ol v-if="loginEvents.length" class="event-list">
            <li v-for="event in loginEvents" :key="event.id">
              <strong>{{ event.username || "未命名账号" }}</strong>
              <span :class="['event-status', event.status]">
                {{ event.status === "success" ? "成功" : event.status === "failure" ? "失败" : "待提交" }}
              </span>
              <p>{{ event.detail }}</p>
              <small>{{ formatTime(event.timestamp) }}</small>
            </li>
          </ol>
          <p v-else class="empty">暂无登录动态。</p>
        </section>

        <section class="panel">
          <div class="panel-header">
            <h2>安全提示</h2>
            <span>登录前确认当前目标与最小权限原则</span>
          </div>
          <ul class="tips">
            <li>如地址携带 redirect，将在登录后优先回跳到目标页面。</li>
            <li>密码不会写入本地存储，账号记忆仅保存用户名与最近结果。</li>
            <li>连续失败时优先检查账号状态、目标环境与权限包是否同步。</li>
            <li>已有登录会话时可直接进入当前默认入口，避免重复认证。</li>
          </ul>
        </section>
      </aside>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

type FeedbackTone = "info" | "success" | "warning" | "error";
type LoginEventTone = "draft" | "success" | "failure";

interface RememberedAccount {
  username: string;
  label: string;
  note: string;
  lastUsedAt: number;
  successCount: number;
  failureCount: number;
  remember: boolean;
  lastStatus: LoginEventTone;
  lastMessage: string;
}

interface LoginEventEntry {
  id: number;
  username: string;
  status: LoginEventTone;
  detail: string;
  timestamp: number;
}

interface QuickProfile {
  key: string;
  label: string;
  username: string;
  password: string;
  note: string;
}

const ACCOUNT_KEY = "gaa-login-remembered-accounts";
const EVENT_KEY = "gaa-login-events";

const auth = useAuthStore();
const router = useRouter();
const route = useRoute();

const username = ref("admin");
const password = ref("123456");
const rememberAccount = ref(true);
const loading = ref(false);
const feedbackTone = ref<FeedbackTone>("info");
const feedbackTitle = ref("准备登录");
const feedbackMessage = ref("输入账号后将沿用现有 auth store 完成登录、权限包装载与默认入口跳转。");
const feedbackHint = ref("如果带有 redirect 参数，登录后会优先回跳。");
const lastFailureMessage = ref("");
const rememberedAccounts = ref<RememberedAccount[]>([]);
const loginEvents = ref<LoginEventEntry[]>([]);

const redirectTarget = computed(() => {
  const redirect = route.query.redirect;
  return typeof redirect === "string" && redirect.length > 0 ? redirect : "";
});

const redirectLabel = computed(() => {
  if (redirectTarget.value) {
    return "优先回跳";
  }
  if (auth.defaultRouterName) {
    return auth.defaultRouterName;
  }
  return "dashboard";
});

const redirectHint = computed(() =>
  redirectTarget.value
    ? `目标：${redirectTarget.value}`
    : `默认入口：${auth.defaultRouterName || "dashboard"}`
);

const activeRoleLabel = computed(
  () => auth.userInfo?.authority?.authorityName || `角色 ${auth.userInfo?.authorityId || "-"}`
);

const latestEvent = computed(() => loginEvents.value[0] || null);
const latestOutcomeLabel = computed(() => {
  if (!latestEvent.value) {
    return "暂无记录";
  }
  return latestEvent.value.status === "success"
    ? "最近成功"
    : latestEvent.value.status === "failure"
      ? "最近失败"
      : "等待提交";
});
const latestOutcomeHint = computed(() => {
  if (!latestEvent.value) {
    return "本地尚未保存登录结果";
  }
  return `${latestEvent.value.username} · ${formatTime(latestEvent.value.timestamp)}`;
});

const recentPersonas = computed(() =>
  rememberedAccounts.value
    .filter((item) => item.successCount > 0)
    .sort((left, right) => right.lastUsedAt - left.lastUsedAt)
    .slice(0, 4)
);

const successfulPersonaCount = computed(() => recentPersonas.value.length);

const quickProfiles = computed<QuickProfile[]>(() => {
  const profiles: QuickProfile[] = [
    {
      key: "demo-admin",
      label: "管理员演示",
      username: "admin",
      password: "123456",
      note: "默认演示账号，可直接联调登录链路。",
    },
  ];

  if (auth.userInfo?.userName) {
    profiles.push({
      key: "current-session",
      label: "当前会话账号",
      username: auth.userInfo.userName,
      password: "",
      note: `当前已登录角色：${activeRoleLabel.value}`,
    });
  }

  recentPersonas.value.slice(0, 2).forEach((account, index) => {
    profiles.push({
      key: `recent-${account.username}-${index}`,
      label: account.label,
      username: account.username,
      password: "",
      note: account.lastMessage || "最近成功登录账号",
    });
  });

  return profiles;
});

const passwordHint = computed(() => {
  if (!password.value) {
    return "密码为空时无法提交。";
  }
  if (password.value.length < 6) {
    return "密码长度偏短，若仍失败优先检查是否录入完整。";
  }
  return "密码已录入，提交后由服务端完成认证。";
});

const canSubmit = computed(() => username.value.length > 0 && password.value.length > 0);

const failureGuide = computed(() => {
  const message = lastFailureMessage.value.trim();

  if (!message) {
    return {
      tone: "info",
      title: "暂无失败记录",
      message: "最近没有登录失败，可直接使用演示账号或记忆账号进入系统。",
      steps: ["确认目标环境正确", "如需回跳，检查 redirect 参数", "首次联调建议先使用管理员演示账号"],
    };
  }

  const lowered = message.toLowerCase();
  if (
    lowered.includes("password") ||
    lowered.includes("credential") ||
    lowered.includes("账号") ||
    lowered.includes("密码") ||
    lowered.includes("unauthorized")
  ) {
    return {
      tone: "warning",
      title: "凭据校验失败",
      message,
      steps: ["重新核对用户名与密码", "优先尝试最近成功账号", "如果多次失败，检查账号是否被停用或角色已切换"],
    };
  }

  if (
    lowered.includes("network") ||
    lowered.includes("fetch") ||
    lowered.includes("timeout") ||
    lowered.includes("连接")
  ) {
    return {
      tone: "error",
      title: "连接链路异常",
      message,
      steps: ["确认 Rust 服务可访问", "检查当前环境地址与代理配置", "若只在回跳时失败，优先移除 redirect 再试"],
    };
  }

  return {
    tone: "error",
    title: "服务端拒绝登录",
    message,
    steps: ["查看最近失败动态确认是否连续异常", "如当前已登录，可先进入现有会话核对权限", "必要时刷新页面后再次尝试"],
  };
});

watch([username, password], () => {
  if (feedbackTone.value === "error") {
    feedbackTone.value = "info";
    feedbackTitle.value = "录入已更新";
    feedbackMessage.value = "检测到新的账号或密码输入，可重新提交登录。";
    feedbackHint.value = redirectHint.value;
  }
});

onMounted(() => {
  rememberedAccounts.value = parseStorage<RememberedAccount>(ACCOUNT_KEY);
  loginEvents.value = parseStorage<LoginEventEntry>(EVENT_KEY).map((entry) => ({
    ...entry,
    id: Number(entry.id) || Date.now(),
    timestamp: Number(entry.timestamp) || Date.now(),
  }));

  const remembered = rememberedAccounts.value[0];
  if (remembered && !auth.isLoggedIn) {
    username.value = remembered.username || username.value;
    rememberAccount.value = remembered.remember;
    feedbackTitle.value = "已恢复本地记忆账号";
    feedbackMessage.value = `最近使用账号：${remembered.username}，可直接补充密码后登录。`;
    feedbackHint.value = remembered.lastMessage || "本地不会保存密码。";
  }
});

function parseStorage<T>(key: string): T[] {
  const raw = window.localStorage.getItem(key);
  if (!raw) {
    return [];
  }
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as T[]) : [];
  } catch {
    return [];
  }
}

function writeStorage<T>(key: string, value: T[]) {
  window.localStorage.setItem(key, JSON.stringify(value));
}

function applyProfile(profile: QuickProfile) {
  username.value = profile.username;
  if (profile.password) {
    password.value = profile.password;
  }
  feedbackTone.value = "info";
  feedbackTitle.value = `已回填：${profile.label}`;
  feedbackMessage.value = profile.note;
  feedbackHint.value = profile.password
    ? "密码已一起带入，可直接提交。"
    : "该快捷入口只回填账号，密码请按当前环境重新录入。";
}

function fillDemoAdmin() {
  applyProfile({
    key: "demo-admin-inline",
    label: "管理员演示",
    username: "admin",
    password: "123456",
    note: "已回填默认演示账号，可直接验证登录流程与默认跳转。",
  });
}

function useRememberedAccount(account: RememberedAccount) {
  username.value = account.username;
  password.value = "";
  rememberAccount.value = account.remember;
  feedbackTone.value = account.lastStatus === "failure" ? "warning" : "info";
  feedbackTitle.value = `已带入 ${account.username}`;
  feedbackMessage.value = account.lastMessage || "请补充当前密码后提交。";
  feedbackHint.value = `最近使用：${formatTime(account.lastUsedAt)}`;
}

function resetForm() {
  username.value = rememberedAccounts.value[0]?.username || "admin";
  password.value = rememberedAccounts.value[0]?.username ? "" : "123456";
  rememberAccount.value = true;
  lastFailureMessage.value = "";
  feedbackTone.value = "info";
  feedbackTitle.value = "已重置录入";
  feedbackMessage.value = "表单已恢复到默认输入状态。";
  feedbackHint.value = rememberedAccounts.value[0]
    ? "已优先保留最近记忆账号。"
    : "可使用演示账号快速回填。";
}

async function enterCurrentSession() {
  if (!auth.isLoggedIn) {
    return;
  }
  await navigateAfterLogin();
}

async function navigateAfterLogin() {
  if (redirectTarget.value) {
    await router.push(redirectTarget.value);
    return;
  }
  const defaultName = auth.defaultRouterName;
  if (defaultName && router.hasRoute(defaultName)) {
    await router.push({ name: defaultName });
    return;
  }
  await router.push({ name: "dashboard" });
}

function rememberOutcome(status: LoginEventTone, detail: string) {
  const now = Date.now();
  const label =
    username.value === "admin"
      ? "管理员演示"
      : rememberedAccounts.value.find((item) => item.username === username.value)?.label ||
        `${username.value} 登录人设`;

  const nextEvent: LoginEventEntry = {
    id: now,
    username: username.value,
    status,
    detail,
    timestamp: now,
  };

  loginEvents.value = [
    nextEvent,
    ...loginEvents.value.filter((item) => item.id !== nextEvent.id),
  ].slice(0, 8);
  writeStorage(EVENT_KEY, loginEvents.value);

  if (!rememberAccount.value && status !== "success") {
    return;
  }

  const current = rememberedAccounts.value.find((item) => item.username === username.value);
  const nextAccount: RememberedAccount = {
    username: username.value,
    label,
    note: detail,
    lastUsedAt: now,
    successCount: current?.successCount || 0,
    failureCount: current?.failureCount || 0,
    remember: rememberAccount.value,
    lastStatus: status,
    lastMessage: detail,
  };

  if (status === "success") {
    nextAccount.successCount += 1;
  }
  if (status === "failure") {
    nextAccount.failureCount += 1;
  }

  rememberedAccounts.value = [
    nextAccount,
    ...rememberedAccounts.value.filter((item) => item.username !== username.value),
  ].slice(0, 6);
  writeStorage(ACCOUNT_KEY, rememberedAccounts.value);
}

async function submit() {
  if (!canSubmit.value) {
    feedbackTone.value = "warning";
    feedbackTitle.value = "表单未完成";
    feedbackMessage.value = "用户名和密码都不能为空。";
    feedbackHint.value = "可使用快捷回填减少手工录入。";
    return;
  }

  loading.value = true;
  feedbackTone.value = "info";
  feedbackTitle.value = "登录请求已发出";
  feedbackMessage.value = `正在以 ${username.value} 登录，并同步权限包与菜单。`;
  feedbackHint.value = redirectHint.value;

  try {
    await auth.login(username.value, password.value);
    lastFailureMessage.value = "";
    const successMessage = `登录成功，默认角色：${activeRoleLabel.value}`;
    rememberOutcome("success", successMessage);
    feedbackTone.value = "success";
    feedbackTitle.value = "登录成功";
    feedbackMessage.value = successMessage;
    feedbackHint.value = "正在进入控制台...";
    await navigateAfterLogin();
  } catch (err) {
    const message = err instanceof Error ? err.message : "登录失败";
    lastFailureMessage.value = message;
    rememberOutcome("failure", message);
    feedbackTone.value = "error";
    feedbackTitle.value = "登录失败";
    feedbackMessage.value = message;
    feedbackHint.value = failureGuide.value.steps[0];
  } finally {
    loading.value = false;
  }
}

function formatTime(value: number) {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(value);
}
</script>

<style scoped>
.page-shell {
  min-height: 100vh;
  padding: 32px 20px;
  background:
    radial-gradient(circle at top left, rgba(59, 130, 246, 0.18), transparent 32%),
    radial-gradient(circle at bottom right, rgba(16, 185, 129, 0.14), transparent 28%),
    #0f172a;
  color: #e2e8f0;
}

.workspace {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(320px, 1fr);
  gap: 20px;
  max-width: 1280px;
  margin: 0 auto;
}

.main-card,
.side-card {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.main-card {
  padding: 28px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  border-radius: 24px;
  background: rgba(15, 23, 42, 0.84);
  box-shadow: 0 24px 80px rgba(15, 23, 42, 0.36);
}

.hero {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  align-items: flex-start;
}

.eyebrow {
  margin: 0 0 8px;
  color: #93c5fd;
  font-size: 12px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.title {
  margin: 0;
  font-size: 36px;
  line-height: 1.1;
}

.subtitle {
  margin: 12px 0 0;
  max-width: 720px;
  color: #cbd5e1;
  line-height: 1.6;
}

.subtitle code {
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(59, 130, 246, 0.16);
  color: #dbeafe;
}

.hero-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.stat-card,
.panel,
.feedback,
.failure-panel {
  border: 1px solid rgba(148, 163, 184, 0.14);
  border-radius: 18px;
  background: rgba(15, 23, 42, 0.64);
}

.stat-card {
  padding: 16px;
}

.stat-label {
  display: block;
  margin-bottom: 10px;
  color: #94a3b8;
  font-size: 13px;
}

.stat-card strong {
  display: block;
  font-size: 20px;
}

.stat-card small {
  display: block;
  margin-top: 8px;
  color: #cbd5e1;
  line-height: 1.5;
}

.feedback {
  padding: 16px 18px;
}

.feedback strong,
.failure-panel strong {
  display: block;
  margin-bottom: 8px;
}

.feedback p,
.failure-panel p {
  margin: 0;
  line-height: 1.6;
}

.feedback small {
  display: block;
  margin-top: 8px;
  color: #cbd5e1;
}

.feedback.info,
.failure-panel.info {
  border-color: rgba(59, 130, 246, 0.32);
  background: rgba(30, 41, 59, 0.84);
}

.feedback.success {
  border-color: rgba(34, 197, 94, 0.32);
  background: rgba(20, 83, 45, 0.28);
}

.feedback.warning,
.failure-panel.warning {
  border-color: rgba(250, 204, 21, 0.32);
  background: rgba(113, 63, 18, 0.24);
}

.feedback.error,
.failure-panel.error {
  border-color: rgba(248, 113, 113, 0.32);
  background: rgba(127, 29, 29, 0.24);
}

.login-form,
.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel {
  padding: 18px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
}

.panel-header h2 {
  margin: 0;
  font-size: 18px;
}

.panel-header span {
  color: #94a3b8;
  font-size: 13px;
  text-align: right;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field label {
  font-weight: 600;
}

.field input {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 14px;
  padding: 12px 14px;
  color: #e2e8f0;
  background: rgba(15, 23, 42, 0.88);
  outline: none;
}

.field input:focus {
  border-color: rgba(96, 165, 250, 0.7);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.18);
}

.helper,
.password-hint,
.empty,
.tips,
.event-list p,
.memory-item small {
  color: #cbd5e1;
}

.options,
.row,
.persona-list,
.memory-list {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.options {
  justify-content: space-between;
  align-items: center;
}

.checkbox {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  color: #e2e8f0;
}

.btn {
  border: none;
  border-radius: 14px;
  padding: 12px 18px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.btn:hover {
  transform: translateY(-1px);
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.7;
  transform: none;
}

.btn.primary {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  color: #eff6ff;
}

.btn.ghost {
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: rgba(15, 23, 42, 0.7);
  color: #e2e8f0;
}

.persona-card,
.memory-item {
  border: 1px solid rgba(148, 163, 184, 0.14);
  border-radius: 16px;
  padding: 14px 16px;
  background: rgba(15, 23, 42, 0.72);
  color: inherit;
  cursor: pointer;
  text-align: left;
}

.persona-card {
  min-width: 180px;
  flex: 1 1 180px;
}

.persona-card strong,
.memory-item strong {
  display: block;
}

.persona-card small,
.persona-card span {
  display: block;
  margin-top: 6px;
  color: #cbd5e1;
}

.memory-list {
  flex-direction: column;
}

.memory-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-dot,
.event-status {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 54px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.status-dot.success,
.event-status.success {
  background: rgba(34, 197, 94, 0.18);
  color: #86efac;
}

.status-dot.failure,
.event-status.failure {
  background: rgba(248, 113, 113, 0.18);
  color: #fca5a5;
}

.status-dot.draft,
.event-status.draft {
  background: rgba(148, 163, 184, 0.16);
  color: #cbd5e1;
}

.failure-panel {
  padding: 16px 18px;
}

.failure-panel ul,
.tips,
.event-list {
  margin: 0;
  padding-left: 18px;
}

.event-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.event-list li {
  display: grid;
  grid-template-columns: auto auto;
  gap: 6px 12px;
  padding-bottom: 12px;
  border-bottom: 1px dashed rgba(148, 163, 184, 0.16);
}

.event-list li:last-child {
  padding-bottom: 0;
  border-bottom: none;
}

.event-list p,
.event-list small {
  grid-column: 1 / -1;
  margin: 0;
}

.tips {
  display: flex;
  flex-direction: column;
  gap: 10px;
  line-height: 1.6;
}

@media (max-width: 1080px) {
  .workspace {
    grid-template-columns: 1fr;
  }

  .stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 720px) {
  .page-shell {
    padding: 18px 14px;
  }

  .main-card {
    padding: 20px;
  }

  .hero,
  .panel-header,
  .options {
    flex-direction: column;
    align-items: stretch;
  }

  .stats {
    grid-template-columns: 1fr;
  }

  .row .btn,
  .hero-actions .btn {
    width: 100%;
  }
}
</style>
