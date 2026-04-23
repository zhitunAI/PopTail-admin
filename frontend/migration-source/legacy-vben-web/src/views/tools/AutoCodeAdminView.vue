<template>
  <div class="stack">
    <div class="card">
      <div class="panel-header">
        <div>
          <h3 class="title">自动代码台账</h3>
          <p class="subtitle">按类型、模块和输出物管理最近生成记录，支持预览详情与进入对应工具页继续处理。</p>
        </div>
        <div class="row wrap-row">
          <button class="btn ghost" @click="reload">刷新</button>
          <button class="btn ghost" :disabled="!selectedRecord" @click="copySelectedPayload">复制详情</button>
          <button class="btn ghost" :disabled="!selectedRecord" @click="openSelectedEntry">打开入口</button>
          <button class="btn ghost" :disabled="!records.length" @click="clearAll">清空台账</button>
        </div>
      </div>
      <div class="summary-grid">
        <div class="summary-card">
          <span class="summary-label">台账总数</span>
          <strong class="summary-value">{{ filteredRecords.length }}</strong>
          <span class="summary-note">全部 {{ records.length }} 条</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">近 24 小时</span>
          <strong class="summary-value">{{ recentCount }}</strong>
          <span class="summary-note">持续有生成活动</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">模块数</span>
          <strong class="summary-value">{{ moduleCount }}</strong>
          <span class="summary-note">{{ moduleSummary }}</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">输出覆盖</span>
          <strong class="summary-value">{{ outputCoverage.length }}</strong>
          <span class="summary-note">{{ outputCoverageSummary }}</span>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="filter-grid">
        <div class="field">
          <label>关键词</label>
          <input v-model="filters.keyword" placeholder="实体、表名、模块、摘要" />
        </div>
        <div class="field">
          <label>记录类型</label>
          <select v-model="filters.kind">
            <option value="all">全部</option>
            <option v-for="kind in kindOptions" :key="kind" :value="kind">{{ kind }}</option>
          </select>
        </div>
        <div class="field">
          <label>模块</label>
          <select v-model="filters.module">
            <option value="all">全部</option>
            <option v-for="moduleName in moduleOptions" :key="moduleName" :value="moduleName">{{ moduleName }}</option>
          </select>
        </div>
        <div class="field">
          <label>输出物</label>
          <select v-model="filters.output">
            <option value="all">全部</option>
            <option v-for="item in outputOptions" :key="item" :value="item">{{ item }}</option>
          </select>
        </div>
      </div>
      <div class="tag-list filter-tags">
        <button
          v-for="kind in quickKinds"
          :key="kind"
          class="tag filter-tag"
          :class="{ active: filters.kind === kind }"
          @click="filters.kind = filters.kind === kind ? 'all' : kind"
        >
          {{ kind }} · {{ kindCounts[kind] ?? 0 }}
        </button>
      </div>
    </div>

    <div class="split-grid ledger-grid">
      <div class="card">
        <h3 class="title">记录列表</h3>
        <p class="subtitle">选中一条记录后，可查看结构化摘要、原始蓝图及对应入口；当前命中 {{ filteredRecords.length }} 条。</p>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>时间</th>
                <th>类型</th>
                <th>名称</th>
                <th>模块</th>
                <th>表名</th>
                <th>输出</th>
                <th>摘要</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in filteredRecords"
                :key="item.raw.ID"
                :class="{ selected: selectedRecord?.raw.ID === item.raw.ID }"
                @click="selectRecord(item)"
              >
                <td>{{ formatTime(item.raw.createdAt) }}</td>
                <td><span class="tag kind-tag">{{ item.kindLabel }}</span></td>
                <td>
                  <div class="cell-title">{{ item.title }}</div>
                  <div class="cell-subtitle">{{ item.routeLabel }}</div>
                </td>
                <td>{{ item.module }}</td>
                <td>{{ item.table }}</td>
                <td>{{ item.outputs.join(' / ') || '—' }}</td>
                <td>{{ item.summary }}</td>
              </tr>
              <tr v-if="!filteredRecords.length">
                <td colspan="7">当前筛选条件下暂无记录</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card detail-card">
        <template v-if="selectedRecord">
          <div class="panel-header">
            <div>
              <h3 class="title">记录详情</h3>
              <p class="subtitle">{{ selectedRecord.title }} · {{ selectedRecord.kindLabel }}</p>
            </div>
            <button class="btn ghost" @click="openSelectedEntry">前往对应入口</button>
          </div>
          <div class="detail-grid">
            <div class="detail-block">
              <h4>台账摘要</h4>
              <ul class="detail-list">
                <li><span>生成时间</span><strong>{{ formatTime(selectedRecord.raw.createdAt) }}</strong></li>
                <li><span>模块</span><strong>{{ selectedRecord.module }}</strong></li>
                <li><span>数据表</span><strong>{{ selectedRecord.table }}</strong></li>
                <li><span>模板</span><strong>{{ selectedRecord.template }}</strong></li>
                <li><span>输出物</span><strong>{{ selectedRecord.outputs.join(' / ') || '—' }}</strong></li>
                <li><span>入口</span><strong>{{ selectedRecord.entryPath }}</strong></li>
              </ul>
            </div>
            <div class="detail-block">
              <h4>内容摘要</h4>
              <p class="detail-summary">{{ selectedRecord.summary }}</p>
              <div class="tag-list">
                <span v-for="flag in selectedRecord.flags" :key="flag" class="tag">{{ flag }}</span>
                <span v-if="!selectedRecord.flags.length" class="tag">无附加标记</span>
              </div>
            </div>
          </div>
          <div class="detail-block">
            <h4>原始蓝图</h4>
            <pre class="code-block">{{ prettyPayload }}</pre>
          </div>
        </template>
        <template v-else>
          <h3 class="title">记录详情</h3>
          <p class="subtitle">请选择左侧台账记录，查看蓝图摘要、输出物和继续处理入口。</p>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { clearAutoCodeRegistryApi, getAutoCodeRegistryApi } from "../../api/admin";
import type { AutoCodeRegistryRecord } from "../../types";

type KindFilter = "all" | string;

type NormalizedRecord = {
  raw: AutoCodeRegistryRecord;
  title: string;
  module: string;
  table: string;
  summary: string;
  template: string;
  kindLabel: string;
  outputs: string[];
  flags: string[];
  routeLabel: string;
  entryPath: string;
};

const router = useRouter();
const records = ref<AutoCodeRegistryRecord[]>([]);
const selectedRecord = ref<NormalizedRecord | null>(null);
const filters = reactive({
  keyword: "",
  kind: "all" as KindFilter,
  module: "all",
  output: "all",
});

const normalizedRecords = computed(() => records.value.map(normalizeRecord));

const kindCounts = computed<Record<string, number>>(() => {
  return normalizedRecords.value.reduce<Record<string, number>>((acc, item) => {
    acc[item.kindLabel] = (acc[item.kindLabel] ?? 0) + 1;
    return acc;
  }, {});
});

const kindOptions = computed(() => Object.keys(kindCounts.value));
const quickKinds = computed(() => kindOptions.value.slice(0, 5));
const moduleOptions = computed(() => {
  return Array.from(new Set(normalizedRecords.value.map((item) => item.module).filter((item) => item !== "默认模块")));
});
const outputOptions = computed(() => {
  return Array.from(new Set(normalizedRecords.value.flatMap((item) => item.outputs))).sort();
});

const filteredRecords = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();
  return normalizedRecords.value.filter((item) => {
    if (filters.kind !== "all" && item.kindLabel !== filters.kind) {
      return false;
    }
    if (filters.module !== "all" && item.module !== filters.module) {
      return false;
    }
    if (filters.output !== "all" && !item.outputs.includes(filters.output)) {
      return false;
    }
    if (!keyword) {
      return true;
    }
    const source = [item.title, item.module, item.table, item.summary, item.routeLabel]
      .join(" ")
      .toLowerCase();
    return source.includes(keyword);
  });
});

const recentCount = computed(() => {
  const now = Date.now();
  return normalizedRecords.value.filter((item) => now - item.raw.createdAt <= 24 * 60 * 60 * 1000).length;
});

const moduleCount = computed(() => moduleOptions.value.length || (records.value.length ? 1 : 0));
const moduleSummary = computed(() => moduleOptions.value.slice(0, 3).join(" / ") || "当前无模块归档");
const outputCoverage = computed(() => outputOptions.value);
const outputCoverageSummary = computed(() => outputCoverage.value.slice(0, 4).join(" / ") || "暂无输出物");
const prettyPayload = computed(() =>
  selectedRecord.value ? JSON.stringify(selectedRecord.value.raw.payload, null, 2) : "",
);

function normalizeRecord(record: AutoCodeRegistryRecord): NormalizedRecord {
  const payload = (record.payload ?? {}) as Record<string, unknown>;
  const kind = String(payload.kind ?? "blueprint");
  const outputs = collectOutputs(payload);
  const entryPath = resolveEntryPath(kind, payload);

  return {
    raw: record,
    title: pickString(payload, ["entity", "entityName", "description", "pageType", "formName", "name"], "未命名记录"),
    module: pickString(payload, ["module", "moduleName"], "默认模块"),
    table: pickString(payload, ["table", "tableName"], "—"),
    summary: buildSummary(payload, outputs),
    template: pickString(payload, ["template", "templateName", "layout"], "默认"),
    kindLabel: mapKindLabel(kind, payload),
    outputs,
    flags: collectFlags(payload),
    routeLabel: buildRouteLabel(payload, entryPath),
    entryPath,
  };
}

function collectOutputs(payload: Record<string, unknown>) {
  const raw = payload.outputs;
  if (Array.isArray(raw)) {
    return raw.map((item) => String(item)).filter(Boolean);
  }
  if (raw && typeof raw === "object") {
    return Object.entries(raw as Record<string, unknown>)
      .filter(([, value]) => Boolean(value))
      .map(([key]) => key.toUpperCase());
  }
  return [];
}

function collectFlags(payload: Record<string, unknown>) {
  const flags = payload.flags;
  if (!flags || typeof flags !== "object") {
    return [];
  }
  return Object.entries(flags as Record<string, unknown>)
    .filter(([, enabled]) => Boolean(enabled))
    .map(([key]) => key);
}

function pickString(payload: Record<string, unknown>, keys: string[], fallback: string) {
  for (const key of keys) {
    const value = payload[key];
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return fallback;
}

function buildSummary(payload: Record<string, unknown>, outputs: string[]) {
  const summary = pickString(payload, ["summary", "prompt", "sections"], "");
  if (summary) {
    return summary;
  }
  const fields = Array.isArray(payload.fields) ? payload.fields.length : 0;
  if (fields > 0) {
    return `包含 ${fields} 个字段，输出 ${outputs.join(" / ") || "未指定"}`;
  }
  return `输出 ${outputs.join(" / ") || "未指定"}`;
}

function mapKindLabel(kind: string, payload: Record<string, unknown>) {
  if (kind === "page-draft") {
    return "页面草图";
  }
  if (kind === "form-designer") {
    return "表单草稿";
  }
  if (kind === "mcp-tool") {
    return "MCP 工具";
  }
  if (payload.pageType) {
    return "页面蓝图";
  }
  if (payload.formName) {
    return "表单设计";
  }
  return "自动代码";
}

function resolveEntryPath(kind: string, payload: Record<string, unknown>) {
  if (kind === "page-draft" || payload.pageType) {
    return "/system/tools/autocode/picture";
  }
  if (kind === "form-designer" || payload.formName) {
    return "/system/tools/form-create";
  }
  if (kind === "mcp-tool") {
    return "/system/tools/autocode/mcp";
  }
  return "/system/tools/autocode";
}

function buildRouteLabel(payload: Record<string, unknown>, entryPath: string) {
  const routeSource = payload.routes;
  if (routeSource && typeof routeSource === "object") {
    return pickString(routeSource as Record<string, unknown>, ["list", "form"], entryPath);
  }
  return entryPath;
}

function selectRecord(record: NormalizedRecord) {
  selectedRecord.value = record;
}

async function reload() {
  const result = await getAutoCodeRegistryApi();
  records.value = result.List;
  const nextSelected = selectedRecord.value
    ? normalizedRecords.value.find((item) => item.raw.ID === selectedRecord.value?.raw.ID)
    : null;
  selectedRecord.value = nextSelected ?? normalizedRecords.value[0] ?? null;
}

async function clearAll() {
  await clearAutoCodeRegistryApi();
  records.value = [];
  selectedRecord.value = null;
}

async function copySelectedPayload() {
  if (!selectedRecord.value) {
    return;
  }
  await navigator.clipboard.writeText(prettyPayload.value);
}

async function openSelectedEntry() {
  if (!selectedRecord.value) {
    return;
  }
  await router.push(selectedRecord.value.entryPath);
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(reload);
</script>

<style scoped>
.panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.wrap-row {
  flex-wrap: wrap;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.summary-card,
.detail-block {
  border: 1px solid var(--vp-c-divider, #e5e7eb);
  border-radius: 12px;
  padding: 14px;
  background: var(--vp-c-bg-soft, #fafafa);
}

.summary-label,
.summary-note,
.cell-subtitle {
  color: var(--vp-c-text-2, #6b7280);
}

.summary-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.summary-value {
  font-size: 24px;
  line-height: 1.1;
}

.filter-grid,
.detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.filter-tags {
  margin-top: 14px;
}

.filter-tag {
  border: none;
  cursor: pointer;
}

.filter-tag.active {
  background: #2563eb;
  color: white;
}

.ledger-grid {
  align-items: start;
}

.selected {
  background: rgba(37, 99, 235, 0.08);
}

tbody tr {
  cursor: pointer;
}

.cell-title {
  font-weight: 600;
}

.kind-tag {
  white-space: nowrap;
}

.detail-card {
  min-height: 100%;
}

.detail-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  gap: 10px;
}

.detail-list li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.detail-list span,
.detail-summary {
  color: var(--vp-c-text-2, #6b7280);
}

@media (max-width: 960px) {
  .panel-header {
    flex-direction: column;
  }
}
</style>
