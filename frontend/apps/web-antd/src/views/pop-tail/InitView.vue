
<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { RouterLink } from "vue-router";

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Alert, Button, Card, Col, Descriptions, DescriptionsItem, Progress, Row, Space, Statistic, Table, Tag } from 'ant-design-vue';

import {
  getAuthorityListApi,
  getMcpToolListApi,
  getMenuListApi,
  getRuntimeInfoApi,
  getSkillListApi,
  getUserListApi,
} from "#/api/pop-tail/admin";
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

function getReadableError(error: unknown) {
  return error instanceof Error ? error.message : "获取初始化状态失败";
}

function isUnauthorizedError(error: unknown) {
  const message = getReadableError(error);
  return /\b401\b|Unauthorized/i.test(message);
}

const summary = reactive({
  users: 0,
  authorities: 0,
  menus: 0,
  skills: 0,
  mcpTools: 0,
});

const loading = ref(false);
const message = ref("");
const error = ref("");
const storageLabel = ref("-");
const lastRefresh = ref("未刷新");
const runtimeBackend = ref("-");
const redisLabel = ref("-");
usePageRefreshLoading(reload);

const readinessScore = computed(() => {
  let score = 40;
  if (summary.users > 0) score += 10;
  if (summary.authorities > 0) score += 10;
  if (summary.menus > 0) score += 10;
  if (summary.skills > 0) score += 10;
  if (summary.mcpTools > 0) score += 10;
  if (redisLabel.value === "已接入") score += 10;
  return Math.min(100, score);
});

const readinessLabel = computed(() => {
  if (readinessScore.value >= 90) return "已进入深验收阶段";
  if (readinessScore.value >= 70) return "主体恢复完成，仍需补深交互";
  return "仍在恢复基线搭建阶段";
});

const nextActions = computed(() => {
  const items = [
    "先从仪表盘或管理总览核对角色、菜单与权限包是否一致。",
    "再进入系统工具与运行态页面，确认工具链、配置与服务状态。",
  ];
  if (!summary.mcpTools) {
    items.push("当前 MCP 工具仍少，建议继续补齐工具定义与联调链路。");
  }
  if (redisLabel.value !== "已接入") {
    items.push("Redis 尚未就绪，建议优先核对会话与状态化链路。");
  }
  return items;
});

const phaseSignals = computed(() => [
  {
    title: "账号与角色基线",
    level: summary.users && summary.authorities ? "已具备" : "待补齐",
    detail: summary.users && summary.authorities ? "用户、角色已可用于继续权限回归。" : "先补全用户和角色基线再推进页面验收。",
  },
  {
    title: "导航与入口",
    level: summary.menus ? "已恢复" : "待恢复",
    detail: summary.menus ? "菜单入口已可分配，适合继续做页面链路验收。" : "菜单尚不足，建议优先恢复入口配置。",
  },
  {
    title: "工具链深度",
    level: summary.skills || summary.mcpTools ? "推进中" : "待建设",
    detail: summary.skills || summary.mcpTools ? "技能与 MCP 已接入一部分，可继续补实。" : "工具链资产还少，建议继续恢复。",
  },
]);

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [users, authorities, menus, skills, mcpTools, runtime] = await Promise.all([
      getUserListApi(),
      getAuthorityListApi(),
      getMenuListApi(),
      getSkillListApi(),
      getMcpToolListApi(),
      getRuntimeInfoApi(),
    ]);
    summary.users = users.Total;
    summary.authorities = authorities.length;
    summary.menus = menus.Total;
    summary.skills = skills.length;
    summary.mcpTools = mcpTools.length;
    runtimeBackend.value = runtime.dbBackend;
    redisLabel.value = runtime.redisEnabled ? "已接入" : "未接入";
    storageLabel.value = `${runtime.dbBackend}${runtime.redisEnabled ? " + Redis" : ""}`;
    lastRefresh.value = new Date().toLocaleString("zh-CN");
    message.value = "已同步初始化引导所需的资产、工具与运行态信息。";
  } catch (error_) {
    if (isUnauthorizedError(error_)) {
      summary.users = 0;
      summary.authorities = 0;
      summary.menus = 0;
      summary.skills = 0;
      summary.mcpTools = 0;
      runtimeBackend.value = "需登录后查看";
      redisLabel.value = "需登录后查看";
      storageLabel.value = "公开引导模式";
      lastRefresh.value = new Date().toLocaleString("zh-CN");
      message.value = "当前以公开引导模式展示；登录后可查看完整初始化台账与运行态。";
      error.value = "";
    } else {
      error.value = getReadableError(error_);
    }
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void reload();
});
</script>

<template>
  <Page
    title="初始化引导工作台"
  >
    <div class="vben-page init-page relative flex h-full min-h-0 flex-1 flex-col">
      <Card :bordered="false" class="hero-card">
        <div class="hero-head">
          <div class="hero-title-row">
            <IconifyIcon class="hero-icon" icon="carbon:rocket" />
            <div>
              <h2>恢复基线与建议入口</h2>
              <p>把初始化状态、资产规模、运行底座和下一步操作统一到一个引导页，方便交接和自检。</p>
            </div>
          </div>
          <Button :loading="loading" type="primary" @click="reload">
            <template #icon>
              <IconifyIcon icon="ant-design:reload-outlined" />
            </template>
            重新拉取初始化状态
          </Button>
        </div>
        <div class="alert-stack">
          <Alert v-if="message" :message="message" show-icon type="success" />
          <Alert v-if="error" :message="error" show-icon type="error" />
        </div>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="summary.users" title="用户">
              <template #prefix>
                <IconifyIcon icon="carbon:user-multiple" />
              </template>
            </Statistic>
            <p>当前可登录或可管理账户数</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="summary.authorities" title="角色">
              <template #prefix>
                <IconifyIcon icon="carbon:user-role" />
              </template>
            </Statistic>
            <p>角色与默认路由已恢复</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="summary.menus" title="菜单">
              <template #prefix>
                <IconifyIcon icon="carbon:menu" />
              </template>
            </Statistic>
            <p>可分配导航与页面入口</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :precision="0" :value="readinessScore" suffix="%" title="就绪度">
              <template #prefix>
                <IconifyIcon icon="carbon:checkmark-outline" />
              </template>
            </Statistic>
            <p>{{ readinessLabel }}</p>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="恢复状态">
            <Progress
              :percent="readinessScore"
              :status="readinessScore >= 90 ? 'success' : readinessScore >= 70 ? 'active' : 'exception'"
            />
            <Descriptions :column="1" bordered class="top-gap" size="small">
              <DescriptionsItem label="登录">已恢复</DescriptionsItem>
              <DescriptionsItem label="核心管理模块">已恢复并持续做深交互</DescriptionsItem>
              <DescriptionsItem label="系统工具页">
                {{ summary.mcpTools ? '已接入真实后端' : '持续推进中' }}
              </DescriptionsItem>
              <DescriptionsItem label="页面 parity">{{ readinessLabel }}</DescriptionsItem>
              <DescriptionsItem label="Tauri 2 重构">Web 验收完成后进入下一阶段</DescriptionsItem>
              <DescriptionsItem label="当前存储">{{ storageLabel }}</DescriptionsItem>
              <DescriptionsItem label="最近刷新">{{ lastRefresh }}</DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="建议入口与下一步">
            <Space class="tag-wrap" wrap>
              <RouterLink to="/dashboard"><Button type="primary">仪表盘</Button></RouterLink>
              <RouterLink to="/system/overview"><Button>管理总览</Button></RouterLink>
              <RouterLink to="/system/state"><Button>系统状态</Button></RouterLink>
              <RouterLink to="/system/tools"><Button>系统工具入口</Button></RouterLink>
              <RouterLink to="/examples"><Button>示例中心</Button></RouterLink>
            </Space>
            <div class="record-list top-gap">
              <div v-for="item in nextActions" :key="item" class="record-item">
                <div class="record-head">
                  <strong>建议动作</strong>
                  <Tag color="processing">Next</Tag>
                </div>
                <p>{{ item }}</p>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="关键资产概览">
            <Table
              :columns="[
                { title: '资产', dataIndex: 'asset', key: 'asset' },
                { title: '数量', dataIndex: 'count', key: 'count' },
                { title: '状态', dataIndex: 'status', key: 'status' },
              ]"
              :data-source="[
                { key: 'users', asset: '用户', count: summary.users, status: summary.users ? '已接入' : '待补齐' },
                { key: 'authorities', asset: '角色', count: summary.authorities, status: summary.authorities ? '已接入' : '待补齐' },
                { key: 'menus', asset: '菜单', count: summary.menus, status: summary.menus ? '已恢复' : '待恢复' },
                { key: 'skills', asset: '技能', count: summary.skills, status: summary.skills ? '已接入' : '待补齐' },
                { key: 'mcp', asset: 'MCP 工具', count: summary.mcpTools, status: summary.mcpTools ? '已定义' : '待补定义' },
                { key: 'db', asset: '数据库后端', count: '-', status: runtimeBackend },
                { key: 'redis', asset: 'Redis', count: '-', status: redisLabel },
              ]"
              :pagination="false"
              row-key="key"
              size="small"
            />
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="阶段判断">
            <div class="record-list">
              <div v-for="item in phaseSignals" :key="item.title" class="record-item">
                <div class="record-head">
                  <strong>{{ item.title }}</strong>
                  <Tag :color="item.level === '待补齐' || item.level === '待建设' || item.level === '待恢复' ? 'warning' : 'success'">
                    {{ item.level }}
                  </Tag>
                </div>
                <p>{{ item.detail }}</p>
              </div>
            </div>
          </Card>
        </Col>
      </Row>
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
