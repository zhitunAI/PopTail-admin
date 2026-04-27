<script setup lang="ts">
import { computed, ref } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";

import { Page } from "@vben/common-ui";
import { IconifyIcon } from "@vben/icons";

import {
  Alert,
  Button,
  Card,
  Col,
  Descriptions,
  DescriptionsItem,
  Result,
  Row,
  Space,
  Statistic,
  Tag,
} from "ant-design-vue";

import { useAuthStore } from "#/store/pop-tail/auth";
import { usePageRefresh } from '#/utils/page-refresh';

type EvidenceStep = {
  detail: string;
  label: string;
  status: "pending" | "ready";
};

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const helperMessage = ref("");
const helperError = ref("");

const routeSegments = computed(() => route.path.split("/").filter(Boolean));

const quickLinks = [
  { label: "管理总览", path: "/system/overview" },
  { label: "系统工具", path: "/system/tools" },
  { label: "示例中心", path: "/examples" },
  { label: "系统状态", path: "/system/state" },
  { label: "控制台", path: "/dashboard" },
];

const prioritizedLinks = computed(() => {
  const path = route.fullPath.toLowerCase();
  const candidates = [
    { label: "用户管理", path: "/system/users", reason: "当前路径与账号、用户或权限段可能相关。" },
    { label: "角色管理", path: "/system/authorities", reason: "如果是鉴权或菜单问题，可先核对角色分配。" },
    { label: "系统工具", path: "/system/tools", reason: "若怀疑工具链路异常，可先回到系统工具入口。" },
    { label: "管理总览", path: "/system/overview", reason: "从总览页重新进入已恢复入口，避免旧地址残留。" },
  ];

  if (path.includes("upload") || path.includes("scan")) {
    return [
      { label: "上传示例", path: "/examples/upload", reason: "当前地址疑似上传链路，可回到上传工作台。" },
      { label: "扫码上传", path: "/scan-upload", reason: "如果你在验证扫码链路，可直接进入扫码上传页。" },
      ...candidates.slice(2),
    ];
  }

  if (path.includes("tool") || path.includes("mcp") || path.includes("auto")) {
    return [
      { label: "系统工具", path: "/system/tools", reason: "当前地址疑似工具链路，可先回到工具总览。" },
      { label: "AI 工作流", path: "/system/tools/ai-workflow", reason: "若是 AI 流程相关问题，可从 AI 工作流页重新进入。" },
      ...candidates.slice(2),
    ];
  }

  return candidates;
});

const suggestedRoute = computed(() => prioritizedLinks.value[0] ?? null);

const permissionHint = computed(() => {
  if (authStore.policyPaths.length === 0) {
    return "当前权限包为空，建议先刷新鉴权";
  }
  if ((authStore.userInfo?.authorities?.length ?? 0) > 1) {
    return "当前账号支持多角色，可尝试切换后重进";
  }
  return "权限包已加载，可核对目标菜单是否已分配";
});

const suggestionText = computed(() => {
  if (authStore.policyPaths.length === 0) {
    return "先返回首页刷新权限包，再检查角色与菜单分配。";
  }
  return suggestedRoute.value?.reason || "建议从管理总览重新进入目标页面。";
});

const evidenceFingerprint = computed(() => {
  const role = authStore.userInfo?.authority?.authorityName || "-";
  const segments = routeSegments.value.join(" > ") || "root";
  return `${route.fullPath} · ${role} · ${segments}`;
});

const browserEvidenceSteps = computed<EvidenceStep[]>(() => [
  {
    label: "上下文已具备截图位",
    detail: "当前地址、当前角色、权限路径和推荐入口均已上屏，可直接作为 browser proof 的首屏截图。",
    status: "ready",
  },
  {
    label: "推荐入口已排序",
    detail: `当前已准备 ${prioritizedLinks.value.length} 个建议入口，后续只需逐个点击并记录实际落点。`,
    status: "ready",
  },
  {
    label: "快速入口已汇总",
    detail: `已收口 ${quickLinks.length} 个恢复入口，可验证 fallback 是否仍可回到主要工作台。`,
    status: "ready",
  },
  {
    label: "真实浏览器 walk 仍待执行",
    detail: "下一步需要在浏览器里访问一个失效地址，确认 404 页建议入口展示与跳转结果是否符合预期。",
    status: "pending",
  },
]);

const screenshotChecklist = computed(() => [
  `截图 1：404 主卡片 + 当前地址（${route.fullPath}）`,
  `截图 2：推荐入口清单，优先验证 ${suggestedRoute.value?.label || "dashboard"}`,
  `截图 3：点击后的实际落点与当前角色（${authStore.userInfo?.authority?.authorityName || "-"})`,
  `截图 4：若失败，记录缺失权限数量（${authStore.policyPaths.length}）与建议动作`,
]);

async function copyText(text: string, successMessage: string) {
  helperError.value = "";
  try {
    await navigator.clipboard.writeText(text);
    helperMessage.value = successMessage;
  } catch {
    if (typeof window !== "undefined") {
      window.prompt("当前环境不支持自动复制，请手动复制：", text);
    }
    helperMessage.value = `${successMessage}（已切换为手动复制）`;
  }
}

async function copyEvidenceSummary() {
  const text = [
    `404 路径：${route.fullPath}`,
    `当前角色：${authStore.userInfo?.authority?.authorityName || "-"}`,
    `默认入口：${authStore.userInfo?.authority?.defaultRouter || "dashboard"}`,
    `权限路径数：${authStore.policyPaths.length}`,
    `推荐入口：${prioritizedLinks.value.map((item) => `${item.label} -> ${item.path}`).join("；")}`,
    `快速入口：${quickLinks.map((item) => `${item.label} -> ${item.path}`).join("；")}`,
    `建议动作：${suggestionText.value}`,
  ].join("\n");
  await copyText(text, "已复制 404 证据摘要。");
}

async function copyBrowserChecklist() {
  const evidenceSteps = browserEvidenceSteps.value.map(
    (item, index) =>
      `${index + 1}. [${item.status === "ready" ? "ready" : "pending"}] ${item.label}：${item.detail}`,
  );
  const screenshotSteps = screenshotChecklist.value.map((item) => `- ${item}`);
  const text = [...evidenceSteps, ...screenshotSteps].join("\n");
  await copyText(text, "已复制浏览器 walk 清单。");
}

function goBack() {
  router.go(-1);
}

usePageRefresh(async () => {});
</script>

<template>
  <Page
    title="页面未找到"
  >
    <div class="vben-page error-page relative">
      <Card :bordered="false" class="hero-card">
        <Result status="404" sub-title="建议先确认当前地址、角色权限和是否仍在使用旧路由。" title="页面未找到">
          <template #icon>
            <IconifyIcon class="result-icon" icon="carbon:warning-alt" />
          </template>
          <template #extra>
            <Space wrap>
              <Button type="primary" @click="goBack">返回上一页</Button>
              <RouterLink to="/dashboard"><Button>返回仪表盘</Button></RouterLink>
              <RouterLink to="/system/overview"><Button>打开管理总览</Button></RouterLink>
              <RouterLink to="/system/tools"><Button>系统工具入口</Button></RouterLink>
            </Space>
          </template>
        </Result>
      </Card>

      <Alert v-if="helperMessage" show-icon type="info" :message="helperMessage" />
      <Alert v-if="helperError" show-icon type="error" :message="helperError" />

      <Row :gutter="[16, 16]">
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="route.fullPath" title="当前地址">
              <template #prefix>
                <IconifyIcon icon="carbon:link" />
              </template>
            </Statistic>
            <p>请核对是否存在拼写或旧路由残留</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="authStore.userInfo?.authority?.authorityName || '-'" title="当前角色">
              <template #prefix>
                <IconifyIcon icon="carbon:user-role" />
              </template>
            </Statistic>
            <p>默认入口：{{ authStore.userInfo?.authority?.defaultRouter || 'dashboard' }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="authStore.policyPaths.length" title="权限路径">
              <template #prefix>
                <IconifyIcon icon="carbon:security" />
              </template>
            </Statistic>
            <p>{{ permissionHint }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="suggestedRoute?.label || 'dashboard'" title="推荐入口">
              <template #prefix>
                <IconifyIcon icon="carbon:compass" />
              </template>
            </Statistic>
            <p>{{ suggestedRoute?.reason || '可先返回首页继续巡检' }}</p>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="当前请求上下文">
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="当前地址">{{ route.fullPath }}</DescriptionsItem>
              <DescriptionsItem label="当前角色">{{ authStore.userInfo?.authority?.authorityName || '-' }}</DescriptionsItem>
              <DescriptionsItem label="热点分段">
                <Space class="tag-wrap" wrap>
                  <Tag v-for="segment in routeSegments" :key="segment">{{ segment }}</Tag>
                  <Tag v-if="routeSegments.length === 0">无</Tag>
                </Space>
              </DescriptionsItem>
              <DescriptionsItem label="建议动作">{{ suggestionText }}</DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="可能想去的页面">
            <div class="record-list">
              <div v-for="item in prioritizedLinks" :key="item.path" class="record-item">
                <div class="record-head">
                  <strong>{{ item.label }}</strong>
                  <RouterLink :to="item.path"><Button size="small">打开</Button></RouterLink>
                </div>
                <p>{{ item.reason }}</p>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="浏览器证据准备">
            <div class="form-meta">
              <p class="soft-text">
                已将当前 404 场景的关键上下文、建议入口和浏览器 walk 清单收口到页面里，后续只需在真实浏览器里点击并截图。
              </p>
              <Space wrap>
                <Button size="small" @click="copyEvidenceSummary">复制证据摘要</Button>
                <Button size="small" @click="copyBrowserChecklist">复制浏览器清单</Button>
              </Space>
            </div>
            <div class="record-list">
              <div v-for="item in browserEvidenceSteps" :key="item.label" class="record-item">
                <div class="record-head">
                  <strong>{{ item.label }}</strong>
                  <Tag :color="item.status === 'ready' ? 'success' : 'warning'">
                    {{ item.status === "ready" ? "已就绪" : "待浏览器验证" }}
                  </Tag>
                </div>
                <p>{{ item.detail }}</p>
              </div>
            </div>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="证据指纹">
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="指纹">{{ evidenceFingerprint }}</DescriptionsItem>
              <DescriptionsItem label="推荐入口数">{{ prioritizedLinks.length }}</DescriptionsItem>
              <DescriptionsItem label="快速入口数">{{ quickLinks.length }}</DescriptionsItem>
              <DescriptionsItem label="建议落点">{{ suggestedRoute?.path || "/dashboard" }}</DescriptionsItem>
            </Descriptions>
            <p class="section-caption top-gap">建议截图 / 记录项</p>
            <ul class="bullet-list">
              <li v-for="item in screenshotChecklist" :key="item">{{ item }}</li>
            </ul>
          </Card>
        </Col>
      </Row>

      <Card :bordered="false" class="panel-card" title="已恢复入口">
        <Space class="tag-wrap" wrap>
          <RouterLink v-for="item in quickLinks" :key="item.path" :to="item.path">
            <Button>{{ item.label }}</Button>
          </RouterLink>
        </Space>
      </Card>
    </div>
  </Page>
</template>

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
