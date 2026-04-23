<script setup lang="ts">
import type { ApiTokenRecord } from "#/types/gin-ai-admin";

import { computed, onMounted, reactive, ref } from "vue";

import { useAccess } from "@vben/access";
import { Page } from "@vben/common-ui";
import { useAccessStore } from "@vben/stores";

import {
  Alert,
  Button,
  Card,
  Collapse,
  CollapsePanel,
  Descriptions,
  DescriptionsItem,
  Form,
  FormItem,
  Input,
  Select,
  SelectOption,
  Space,
  Table,
  TableColumn,
  TabPane,
  Tabs,
  Tag,
} from "ant-design-vue";

import { clearApiTokenApi, getApiTokenListApi, issueApiTokenApi } from "#/api/gin-ai-admin/admin";

type ScopePreset = {
  key: string;
  label: string;
  name: string;
  scope: string;
  ttl: string;
};

type ApiTokenButtonAction = "clear" | "copy" | "export" | "issue" | "rotate";

const API_TOKEN_BUTTON_ACCESS_CODES: Record<ApiTokenButtonAction, string[]> = {
  clear: [
    "清空记录",
    "作废",
    "作废令牌",
    "删除",
    "btn:清空记录",
    "btn:作废",
    "btn:作废令牌",
    "btn:apiTokens:clear",
    "btn:apiToken:clear",
    "btn:/system/tools/api-tokens:clear",
    "btn:/admin/apiToken:clear",
    "btn:apiTokens:invalidate",
    "btn:apiToken:invalidate",
    "btn:/system/tools/api-tokens:invalidate",
    "btn:/admin/apiToken:invalidate",
    "btn:apiTokens:删除",
    "btn:apiToken:删除",
    "btn:/system/tools/api-tokens:删除",
    "btn:/admin/apiToken:删除",
  ],
  copy: [
    "复制",
    "复制令牌摘要",
    "Curl示例",
    "btn:复制",
    "btn:复制令牌摘要",
    "btn:Curl示例",
    "btn:apiTokens:copy",
    "btn:apiToken:copy",
    "btn:/system/tools/api-tokens:copy",
    "btn:/admin/apiToken:copy",
  ],
  export: [
    "导出",
    "导出当前筛选",
    "导出台账",
    "btn:导出",
    "btn:导出当前筛选",
    "btn:导出台账",
    "btn:apiTokens:export",
    "btn:apiToken:export",
    "btn:/system/tools/api-tokens:export",
    "btn:/admin/apiToken:export",
  ],
  issue: [
    "签发",
    "签发JWT",
    "登记令牌",
    "新增",
    "btn:签发",
    "btn:签发JWT",
    "btn:登记令牌",
    "btn:新增",
    "btn:apiTokens:issue",
    "btn:apiToken:issue",
    "btn:/system/tools/api-tokens:issue",
    "btn:/admin/apiToken:issue",
    "btn:apiTokens:create",
    "btn:apiToken:create",
    "btn:/system/tools/api-tokens:create",
    "btn:/admin/apiToken:create",
  ],
  rotate: [
    "轮转",
    "填充轮转建议",
    "补发",
    "btn:轮转",
    "btn:填充轮转建议",
    "btn:补发",
    "btn:apiTokens:rotate",
    "btn:apiToken:rotate",
    "btn:/system/tools/api-tokens:rotate",
    "btn:/admin/apiToken:rotate",
  ],
};

const API_TOKEN_BUTTON_ACCESS_LABELS: Record<ApiTokenButtonAction, string[]> = {
  clear: ["清空记录", "作废", "作废令牌", "删除", "clear", "delete", "invalidate"],
  copy: ["复制", "复制令牌摘要", "Curl示例", "copy"],
  export: ["导出", "导出当前筛选", "导出台账", "export"],
  issue: ["签发", "签发JWT", "登记令牌", "新增", "issue", "create"],
  rotate: ["轮转", "填充轮转建议", "补发", "rotate"],
};

const API_TOKEN_BUTTON_ACCESS_CODE_PREFIXES = [
  "btn:apiTokens:",
  "btn:apiToken:",
  "btn:/system/tools/api-tokens:",
  "btn:/admin/apiToken:",
];

const capabilities = [
  { name: "短期访问", desc: "适合批量任务或临时集成，优先使用 24h / 7d 生命周期" },
  { name: "按域授权", desc: "建议按审计、用户、发布等单一职责拆分 scope" },
  { name: "轮转落账", desc: "签发后保留台账，异常状态或超长 TTL 需优先复核" },
];

const ttlOptions = ["24h", "7d", "30d"];
const scopePresets: ScopePreset[] = [
  {
    key: "audit-read",
    label: "审计只读",
    name: "audit-reader",
    scope: "system.audit.read",
    ttl: "7d",
  },
  {
    key: "publish-bot",
    label: "发布机器人",
    name: "release-bot",
    scope: "release.deploy",
    ttl: "24h",
  },
  {
    key: "plugin-sync",
    label: "插件同步",
    name: "plugin-sync",
    scope: "plugin.install.manage",
    ttl: "30d",
  },
  {
    key: "example-upload",
    label: "示例上传",
    name: "upload-agent",
    scope: "example.upload.write",
    ttl: "24h",
  },
];

const tokens = ref<ApiTokenRecord[]>([]);
const selectedToken = ref<ApiTokenRecord | null>(null);
const loading = ref(false);
const submitting = ref(false);
const clearing = ref(false);
const statusText = ref("等待加载令牌台账。");
const accessStore = useAccessStore();
const { hasAccessByCodes } = useAccess();
const filters = reactive({
  keyword: "",
  status: "all",
  ttl: "all",
});
const draft = reactive({
  name: "gateway-sync",
  scope: "system.audit.read",
  ttl: "24h",
});

const statusOptions = computed(() =>
  [...new Set(tokens.value.map((item) => item.status).filter(Boolean))],
);

const filteredTokens = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();
  return tokens.value.filter((item) => {
    if (
      keyword &&
      !`${item.name} ${item.scope}`.toLowerCase().includes(keyword)
    ) {
      return false;
    }
    if (filters.status !== "all" && item.status !== filters.status) {
      return false;
    }
    if (filters.ttl !== "all" && item.ttl !== filters.ttl) {
      return false;
    }
    return true;
  });
});

const recentTokens = computed(() =>
  tokens.value.toSorted((left, right) => right.ID - left.ID).slice(0, 4),
);

const rotationCandidates = computed(() =>
  tokens.value
    .filter((item) => needsRotation(item))
    .toSorted((left, right) => right.ID - left.ID)
    .slice(0, 4),
);

const apiTokenButtonAccessCandidates = new Set<string>(
  Object.values(API_TOKEN_BUTTON_ACCESS_CODES).flat(),
);
const apiTokenButtonAccessLabels = Object.values(API_TOKEN_BUTTON_ACCESS_LABELS).flat();

const hasApiTokenButtonAccessEnvelope = computed(() =>
  accessStore.accessCodes.some(
    (code) =>
      apiTokenButtonAccessCandidates.has(code) ||
      API_TOKEN_BUTTON_ACCESS_CODE_PREFIXES.some((prefix) => code.startsWith(prefix)) ||
      apiTokenButtonAccessLabels.some(
        (label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`),
      ),
  ),
);

function hasFlexibleApiTokenButtonAccess(action: ApiTokenButtonAction) {
  if (hasAccessByCodes(API_TOKEN_BUTTON_ACCESS_CODES[action])) {
    return true;
  }
  const labels = API_TOKEN_BUTTON_ACCESS_LABELS[action];
  return accessStore.accessCodes.some((code) =>
    labels.some((label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`)),
  );
}

function canUseApiTokenAction(action: ApiTokenButtonAction) {
  return !hasApiTokenButtonAccessEnvelope.value || hasFlexibleApiTokenButtonAccess(action);
}

function denyApiTokenAction(action: ApiTokenButtonAction) {
  if (canUseApiTokenAction(action)) {
    return false;
  }
  statusText.value = "无按钮权限，当前账号不能执行该 API Token 操作。";
  return true;
}

function isHealthyStatus(status: string) {
  return ["active", "issued", "ready"].includes(status.toLowerCase());
}

function needsRotation(item: ApiTokenRecord) {
  return item.ttl === "30d" || !isHealthyStatus(item.status);
}

function getRotationReason(item: ApiTokenRecord) {
  if (!isHealthyStatus(item.status)) {
    return "状态异常，建议尽快复核或重发";
  }
  if (item.ttl === "30d") {
    return "长期令牌，建议纳入轮转计划";
  }
  if (item.ttl === "7d") {
    return "中周期令牌，建议在本周内确认续用";
  }
  return "短周期令牌，可维持当前策略";
}

function statusTone(status: string) {
  if (isHealthyStatus(status)) {
    return "ok";
  }
  if (status.toLowerCase().includes("pending")) {
    return "pending";
  }
  return "risk";
}

function buildDigest(item: ApiTokenRecord) {
  return `${item.name} | ${item.scope} | ${item.ttl} | ${item.status}`;
}

function applyScopePreset(preset: ScopePreset) {
  if (denyApiTokenAction("issue")) {
    return;
  }
  draft.name = preset.name;
  draft.scope = preset.scope;
  draft.ttl = preset.ttl;
  statusText.value = `已应用作用域预设：${preset.label}。`;
}

function fillDraftFromRecommendation() {
  if (denyApiTokenAction("rotate")) {
    return;
  }
  const candidate = rotationCandidates.value[0] ?? recentTokens.value[0];
  if (!candidate) {
    statusText.value = "当前没有可参考的令牌记录，请先新增一条令牌。";
    return;
  }
  draft.name = `${candidate.name}-next`;
  draft.scope = candidate.scope;
  draft.ttl = candidate.ttl === "30d" ? "7d" : "24h";
  statusText.value = `已根据 ${candidate.name} 生成轮转草稿，建议复核后再登记。`;
}

function selectToken(item: ApiTokenRecord) {
  selectedToken.value = item;
  statusText.value = `已定位令牌 ${item.name}，可复制摘要或按此补发新令牌。`;
}

function rowClassName(record: ApiTokenRecord) {
  return selectedToken.value?.ID === record.ID ? "selected-row" : "";
}

function applyFilters() {
  statusText.value = `已应用筛选，命中 ${filteredTokens.value.length} 条令牌记录。`;
  if (
    selectedToken.value &&
    !filteredTokens.value.some((item) => item.ID === selectedToken.value?.ID)
  ) {
    selectedToken.value = filteredTokens.value[0] ?? null;
  }
}

function resetFilters() {
  filters.keyword = "";
  filters.status = "all";
  filters.ttl = "all";
  statusText.value = `已重置筛选，当前展示 ${tokens.value.length} 条令牌记录。`;
  selectedToken.value = tokens.value[0] ?? null;
}

async function copyTokenDigest() {
  if (denyApiTokenAction("copy")) {
    return;
  }
  if (!selectedToken.value) {
    statusText.value = "请先选择一条令牌记录。";
    return;
  }
  const text = buildDigest(selectedToken.value);
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      statusText.value = `已复制令牌摘要：${selectedToken.value.name}。`;
      return;
    }
  } catch {
    // fall through
  }
  if (typeof window !== "undefined") {
    window.prompt("当前环境不支持自动写入剪贴板，请手动复制：", text);
    statusText.value = `已提供手动复制窗口：${selectedToken.value.name}。`;
  }
}

function exportTokenLedger() {
  if (denyApiTokenAction("export")) {
    return;
  }
  if (typeof window === "undefined") {
    statusText.value = "当前环境不支持导出令牌台账。";
    return;
  }
  const content = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      filters: { ...filters },
      selectedToken: selectedToken.value,
      records: filteredTokens.value,
    },
    null,
    2,
  );
  const blob = new Blob([content], { type: "application/json;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `api-token-ledger-${Date.now()}.json`;
  link.click();
  window.URL.revokeObjectURL(url);
  statusText.value = `已导出 ${filteredTokens.value.length} 条令牌记录。`;
}

async function load(showMessage = false) {
  loading.value = true;
  try {
    const result = await getApiTokenListApi();
    tokens.value = result.List;
    if (!selectedToken.value || !tokens.value.some((item) => item.ID === selectedToken.value?.ID)) {
      selectedToken.value = recentTokens.value[0] ?? tokens.value[0] ?? null;
    }
    if (showMessage) {
      statusText.value = `已刷新令牌台账，共 ${tokens.value.length} 条记录。`;
    } else if (tokens.value.length > 0) {
      statusText.value = `令牌台账已加载，最近签发 ${recentTokens.value[0]?.name ?? "无"}。`;
    } else {
      statusText.value = "当前台账为空，可先签发一条服务令牌。";
    }
  } finally {
    loading.value = false;
  }
}

async function reloadTokens() {
  await load(true);
}

async function issueToken() {
  if (denyApiTokenAction("issue")) {
    return;
  }
  const name = draft.name.trim();
  const scope = draft.scope.trim();
  if (!name || !scope) {
    statusText.value = "令牌名称和作用域不能为空。";
    return;
  }
  submitting.value = true;
  try {
    const created = await issueApiTokenApi({
      name,
      scope,
      ttl: draft.ttl,
    });
    draft.name = `${name}-next`;
    selectedToken.value = created;
    statusText.value = `已登记令牌 ${created.name}，建议复制摘要并同步给调用方。`;
    await load(false);
  } finally {
    submitting.value = false;
  }
}

async function clearTokens() {
  if (denyApiTokenAction("clear")) {
    return;
  }
  clearing.value = true;
  try {
    await clearApiTokenApi();
    tokens.value = [];
    selectedToken.value = null;
    statusText.value = "已清空令牌台账记录。";
  } finally {
    clearing.value = false;
  }
}

onMounted(async () => {
  await load(false);
});
</script>

<template>
  <Page
    title="API Token 工作台"
  >
    <div class="workbench">
      <div class="content-grid">
        <Card class="panel-card" :bordered="false">
          <template #title>
            <div class="card-title-row">
              <span>签发与轮转</span>
              <Button :loading="loading" @click="reloadTokens">刷新台账</Button>
            </div>
          </template>

          <Alert :message="statusText" show-icon type="info" />

          <Form class="mt-4" layout="vertical">
            <div class="form-grid">
              <FormItem label="令牌名称">
                <Input v-model:value="draft.name" placeholder="例如 gateway-sync" />
              </FormItem>
              <FormItem label="作用域">
                <Input
                  v-model:value="draft.scope"
                  placeholder="例如 system.audit.read"
                  @keydown.enter="issueToken"
                />
              </FormItem>
              <FormItem label="有效期">
                <Select v-model:value="draft.ttl">
                  <SelectOption v-for="item in ttlOptions" :key="item" :value="item">{{ item }}</SelectOption>
                </Select>
              </FormItem>
            </div>
          </Form>

          <Collapse ghost>
            <CollapsePanel key="presets" header="作用域预设">
              <div class="tag-wrap">
                <Tag
                  v-for="preset in scopePresets"
                  :key="preset.key"
                  class="cursor-tag"
                  :class="{ 'cursor-tag-disabled': !canUseApiTokenAction('issue') }"
                  :tabindex="canUseApiTokenAction('issue') ? 0 : -1"
                  color="blue"
                  @click="applyScopePreset(preset)"
                >
                  {{ preset.label }}
                </Tag>
              </div>
            </CollapsePanel>
            <CollapsePanel key="capabilities" header="签发建议">
              <div class="tag-wrap">
                <Tag v-for="item in capabilities" :key="item.name" color="default">
                  {{ item.name }}：{{ item.desc }}
                </Tag>
              </div>
            </CollapsePanel>
          </Collapse>

          <Space class="mt-4" wrap>
            <Button
              v-if="canUseApiTokenAction('issue')"
              type="primary"
              :loading="submitting"
              @click="issueToken"
            >
              登记令牌
            </Button>
            <Button
              :disabled="submitting || !canUseApiTokenAction('rotate')"
              @click="fillDraftFromRecommendation"
            >
              填充轮转建议
            </Button>
            <Button
              v-if="canUseApiTokenAction('clear')"
              danger
              :disabled="clearing || tokens.length === 0"
              :loading="clearing"
              @click="clearTokens"
            >
              清空记录
            </Button>
          </Space>
        </Card>

        <Card class="panel-card" :bordered="false">
          <template #title>
            <div class="card-title-row">
              <span>令牌台账</span>
              <Space wrap>
                <Button
                  :disabled="!selectedToken || !canUseApiTokenAction('copy')"
                  @click="copyTokenDigest"
                >
                  复制令牌摘要
                </Button>
                <Button
                  :disabled="filteredTokens.length === 0 || !canUseApiTokenAction('export')"
                  @click="exportTokenLedger"
                >
                  导出当前筛选
                </Button>
              </Space>
            </div>
          </template>

          <Form layout="vertical">
            <div class="form-grid compact-grid">
              <FormItem label="关键词">
                <Input v-model:value="filters.keyword" placeholder="搜索名称或作用域" @keydown.enter="applyFilters" />
              </FormItem>
              <FormItem label="状态">
                <Select v-model:value="filters.status">
                  <SelectOption value="all">全部状态</SelectOption>
                  <SelectOption v-for="item in statusOptions" :key="item" :value="item">{{ item }}</SelectOption>
                </Select>
              </FormItem>
              <FormItem label="有效期">
                <Select v-model:value="filters.ttl">
                  <SelectOption value="all">全部有效期</SelectOption>
                  <SelectOption v-for="item in ttlOptions" :key="item" :value="item">{{ item }}</SelectOption>
                </Select>
              </FormItem>
            </div>
          </Form>

          <Space wrap>
            <Button @click="applyFilters">应用筛选</Button>
            <Button @click="resetFilters">重置筛选</Button>
          </Space>

          <Tabs class="mt-4">
            <TabPane key="ledger" tab="令牌列表">
              <Table
                :data-source="filteredTokens"
                :pagination="{ pageSize: 8 }"
                :row-class-name="rowClassName"
                :row-key="(record) => record.ID"
                :scroll="{ x: 860 }"
                size="small"
              >
                <TableColumn data-index="name" key="name" title="名称" />
                <TableColumn data-index="scope" key="scope" title="作用域" />
                <TableColumn data-index="ttl" key="ttl" title="有效期" />
                <TableColumn key="status" title="状态">
                  <template #default="{ record }">
                    <Tag :color="statusTone(record.status) === 'ok' ? 'success' : statusTone(record.status) === 'pending' ? 'warning' : 'error'">
                      {{ record.status }}
                    </Tag>
                  </template>
                </TableColumn>
                <TableColumn key="advice" title="建议">
                  <template #default="{ record }">{{ getRotationReason(record) }}</template>
                </TableColumn>
                <TableColumn fixed="right" key="actions" title="操作" width="100">
                  <template #default="{ record }">
                    <Button size="small" @click="selectToken(record)">查看</Button>
                  </template>
                </TableColumn>
              </Table>
            </TabPane>
            <TabPane key="recent" tab="最近签发">
              <div class="tag-wrap">
                <Tag
                  v-for="item in recentTokens"
                  :key="item.ID"
                  class="cursor-tag"
                  color="blue"
                  @click="selectToken(item)"
                >
                  {{ item.name }} · {{ item.scope }}
                </Tag>
                <Tag v-if="recentTokens.length === 0">暂无最近签发记录</Tag>
              </div>
            </TabPane>
            <TabPane key="rotation" tab="轮转建议">
              <div class="tag-wrap">
                <Tag
                  v-for="item in rotationCandidates"
                  :key="item.ID"
                  class="cursor-tag"
                  color="warning"
                  @click="selectToken(item)"
                >
                  {{ item.name }} · {{ getRotationReason(item) }}
                </Tag>
                <Tag v-if="rotationCandidates.length === 0">当前没有明显需要轮转的令牌</Tag>
              </div>
            </TabPane>
          </Tabs>

          <Descriptions v-if="selectedToken" class="mt-4" :column="1" bordered size="small">
            <DescriptionsItem label="令牌名称">{{ selectedToken.name }}</DescriptionsItem>
            <DescriptionsItem label="作用域">{{ selectedToken.scope }}</DescriptionsItem>
            <DescriptionsItem label="有效期">{{ selectedToken.ttl }}</DescriptionsItem>
            <DescriptionsItem label="状态">{{ selectedToken.status }}</DescriptionsItem>
            <DescriptionsItem label="轮转判断">{{ getRotationReason(selectedToken) }}</DescriptionsItem>
            <DescriptionsItem label="摘要">{{ buildDigest(selectedToken) }}</DescriptionsItem>
          </Descriptions>
          <Alert v-else class="mt-4" message="点击任意记录查看详情。" show-icon type="warning" />
        </Card>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.workbench {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-grid,
.content-grid,
.form-grid {
  display: grid;
  gap: 16px;
}

.metric-grid {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.content-grid {
  align-items: start;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
}

.form-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.compact-grid {
  grid-template-columns: minmax(0, 1.2fr) minmax(160px, 160px) minmax(160px, 160px);
}

.metric-head,
.card-title-row {
  align-items: center;
  display: flex;
  gap: 8px;
  justify-content: space-between;
}

.metric-head {
  color: var(--ant-color-text-secondary);
  justify-content: flex-start;
  margin-bottom: 12px;
}

.metric-icon {
  font-size: 18px;
}

.metric-note {
  color: var(--ant-color-text-secondary);
}

.panel-card {
  border-radius: 16px;
}

.tag-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.cursor-tag {
  cursor: pointer;
}

.cursor-tag-disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

:deep(.selected-row td) {
  background: rgba(22, 119, 255, 0.08) !important;
}

@media (max-width: 960px) {
  .content-grid,
  .compact-grid {
    grid-template-columns: 1fr;
  }

  .card-title-row {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
