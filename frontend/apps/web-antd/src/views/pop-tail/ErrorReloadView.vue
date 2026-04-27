
<template>
  <Page
    title="页面恢复中"
  >
    <div class="vben-page recovery-page relative">
      <Card :bordered="false" class="hero-card">
        <Result status="info" sub-title="自动恢复会先同步会话，再补权限包，最后跳回目标页。" title="恢复当前页面上下文">
          <template #icon>
            <IconifyIcon class="result-icon" icon="carbon:renew" />
          </template>
          <template #extra>
            <Space wrap>
              <Button :loading="running" type="primary" @click="runRecovery">
                <template #icon>
                  <IconifyIcon icon="ant-design:reload-outlined" />
                </template>
                立即重试
              </Button>
              <Button :disabled="running" @click="returnBack">返回上一页</Button>
              <Button :disabled="running" @click="goDashboard">前往控制台</Button>
              <Button :disabled="running" @click="openLogin">回到登录页</Button>
            </Space>
          </template>
        </Result>
        <div class="alert-stack">
          <Alert v-if="message" :message="message" show-icon type="info" />
          <Alert v-if="error" :message="error" show-icon type="error" />
        </div>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="8" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="currentStage" title="当前阶段">
              <template #prefix>
                <IconifyIcon icon="carbon:flow-stream" />
              </template>
            </Statistic>
          </Card>
        </Col>
        <Col :lg="8" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="destinationLabel" title="目标页">
              <template #prefix>
                <IconifyIcon icon="carbon:launch" />
              </template>
            </Statistic>
          </Card>
        </Col>
        <Col :lg="8" :md="24" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="countdownLabel" title="倒计时">
              <template #prefix>
                <IconifyIcon icon="carbon:timer" />
              </template>
            </Statistic>
          </Card>
        </Col>
      </Row>

      <Card :bordered="false" class="panel-card" title="恢复清单">
        <Steps :current="Math.max(steps.findIndex((item) => item.status === '进行中'), 0)" size="small">
          <Step
            v-for="item in steps"
            :key="item.id"
            :description="item.detail"
            :status="item.status === '完成' ? 'finish' : item.status === '失败' ? 'error' : item.status === '进行中' ? 'process' : 'wait'"
            :title="item.label"
          />
        </Steps>
        <Table
          :columns="[
            { title: '步骤', dataIndex: 'label', key: 'label' },
            { title: '状态', dataIndex: 'status', key: 'status' },
            { title: '说明', dataIndex: 'detail', key: 'detail' },
          ]"
          :data-source="steps"
          :pagination="false"
          class="top-gap"
          row-key="id"
          size="small"
        />
      </Card>

      <Card :bordered="false" class="panel-card" title="当前恢复上下文">
        <Descriptions :column="1" bordered size="small">
          <DescriptionsItem label="目标页">{{ destinationLabel }}</DescriptionsItem>
          <DescriptionsItem label="当前阶段">{{ currentStage }}</DescriptionsItem>
          <DescriptionsItem label="倒计时">{{ countdownLabel }}</DescriptionsItem>
          <DescriptionsItem label="会话状态">{{ auth.isLoggedIn ? '已登录' : '未登录' }}</DescriptionsItem>
          <DescriptionsItem label="权限包规模">{{ auth.policyPaths.length }} 条</DescriptionsItem>
        </Descriptions>
        <Space class="tag-wrap top-gap" wrap>
          <Tag :color="running ? 'processing' : 'default'">{{ running ? '恢复执行中' : '等待触发' }}</Tag>
          <Tag :color="auth.isLoggedIn ? 'success' : 'warning'">{{ auth.isLoggedIn ? '检测到有效会话' : '需要重新登录' }}</Tag>
          <Tag color="default">redirect: {{ route.query.redirect?.toString() || '-' }}</Tag>
        </Space>
      </Card>
    </div>
  </Page>
</template>

<script setup lang="ts">
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { Alert, Button, Card, Col, Descriptions, DescriptionsItem, Result, Row, Space, Statistic, Steps, Step, Table, Tag } from 'ant-design-vue';
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAuthStore } from "#/store/pop-tail/auth";
import { usePageRefresh } from '#/utils/page-refresh';

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

usePageRefresh(async () => {});

onMounted(() => {
  void runRecovery();
});

onBeforeUnmount(() => {
  stopTimer();
});
</script>


<style scoped>
.vben-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-card,
.panel-card,
.metric-card {
  border-radius: 16px;
}

.hero-head,
.hero-title-row,
.record-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.hero-title-row {
  align-items: center;
}

.hero-icon,
.result-icon {
  font-size: 28px;
  color: var(--ant-color-primary);
}

.hero-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric-card p,
.panel-card p,
.hero-card p,
.field-hint,
.soft-text,
.footnote,
.selection-card span {
  margin: 0;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.panel-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-wrap {
  width: 100%;
}

.alert-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.top-gap {
  margin-top: 12px;
}

.record-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.record-item,
.selection-card,
.inner-card :deep(.ant-card-body) {
  border: 1px solid var(--ant-color-border-secondary);
  border-radius: 14px;
  padding: 12px 14px;
  background: var(--ant-color-fill-quaternary);
}

.selection-card {
  text-align: left;
  width: 100%;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.selection-card:hover {
  border-color: var(--ant-color-primary);
  transform: translateY(-1px);
}

.selection-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-caption {
  margin: 0 0 8px;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.full-width {
  width: 100%;
}

.toolbar-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toolbar-field label,
.section-head h3,
.hero-card h2,
.inner-card h3 {
  margin: 0;
}

.section-head p {
  margin: 4px 0 0;
}

.profile-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.form-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.bullet-list {
  margin: 0;
  padding-left: 18px;
  color: var(--ant-color-text-description);
}

.bullet-list li + li {
  margin-top: 6px;
}

.session-summary {
  display: flex;
  gap: 12px;
  align-items: center;
}

.summary-avatar {
  background: color-mix(in srgb, var(--ant-color-primary) 12%, white);
  color: var(--ant-color-primary);
}

@media (max-width: 768px) {
  .hero-head,
  .hero-title-row,
  .record-head,
  .form-meta {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
