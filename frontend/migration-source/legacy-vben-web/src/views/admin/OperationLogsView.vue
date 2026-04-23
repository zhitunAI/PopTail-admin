<template>
  <div class="card">
    <div class="header">
      <div>
        <h3 class="title">操作审计工作台</h3>
        <p class="subtitle">围绕接口轨迹快速定位异常请求、慢操作与高频访问路径。</p>
      </div>
      <div class="row compact">
        <button class="btn ghost" @click="focusAbnormal">
          聚焦异常
        </button>
        <button class="btn ghost" @click="focusSlow">
          聚焦慢请求
        </button>
        <button class="btn primary" @click="load">
          刷新日志
        </button>
      </div>
    </div>

    <div class="summary-grid">
      <div class="summary-card">
        <span class="summary-label">日志总量</span>
        <strong>{{ sourceRows.length }}</strong>
        <small>当前已拉取的审计轨迹</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">异常请求</span>
        <strong>{{ abnormalRows.length }}</strong>
        <small>{{ abnormalRate }}% 状态异常占比</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">慢请求</span>
        <strong>{{ slowRows.length }}</strong>
        <small>阈值 {{ slowThreshold }} ms</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">平均耗时</span>
        <strong>{{ averageLatency }} ms</strong>
        <small>峰值 {{ maxLatency }} ms</small>
      </div>
    </div>

    <div class="analysis-grid">
      <div class="panel">
        <div class="panel-title">请求方法分布</div>
        <div class="tag-list">
          <button
            v-for="entry in methodBreakdown"
            :key="entry.label"
            class="tag-button"
            :class="{ active: filters.method === entry.label }"
            @click="toggleMethod(entry.label)"
          >
            <span>{{ entry.label }}</span>
            <strong>{{ entry.count }}</strong>
          </button>
        </div>
      </div>
      <div class="panel">
        <div class="panel-title">状态分组统计</div>
        <div class="tag-list">
          <button
            v-for="entry in statusBreakdown"
            :key="entry.label"
            class="tag-button"
            :class="{ active: filters.statusGroup === entry.label }"
            @click="toggleStatusGroup(entry.label)"
          >
            <span>{{ entry.label }}</span>
            <strong>{{ entry.count }}</strong>
          </button>
        </div>
      </div>
      <div class="panel">
        <div class="panel-title">热点路径</div>
        <div class="hot-path-list">
          <button
            v-for="entry in topPathBreakdown"
            :key="entry.label"
            class="hot-path-item"
            :class="{ active: filters.path === entry.label }"
            @click="togglePath(entry.label)"
          >
            <div>
              <strong>{{ entry.label }}</strong>
              <small>{{ entry.statusHint }}</small>
            </div>
            <span>{{ entry.count }} 次</span>
          </button>
        </div>
      </div>
    </div>

    <div class="toolbar-grid">
      <div class="field">
        <label>关键词</label>
        <input
          v-model.trim="filters.keyword"
          placeholder="搜索路径 / IP / 方法 / 状态"
        />
      </div>
      <div class="field">
        <label>请求方法</label>
        <input v-model.trim="filters.method" placeholder="如 GET / POST" />
      </div>
      <div class="field">
        <label>请求路径</label>
        <input v-model.trim="filters.path" placeholder="支持片段匹配" />
      </div>
      <div class="field">
        <label>状态码</label>
        <input v-model.trim="filters.status" placeholder="如 200 / 500" />
      </div>
      <div class="field">
        <label>状态分组</label>
        <select v-model="filters.statusGroup">
          <option value="">全部</option>
          <option value="成功">成功</option>
          <option value="重定向">重定向</option>
          <option value="客户端异常">客户端异常</option>
          <option value="服务端异常">服务端异常</option>
          <option value="未知">未知</option>
        </select>
      </div>
      <div class="field">
        <label>耗时模式</label>
        <select v-model="filters.speedMode">
          <option value="">全部</option>
          <option value="slow">仅慢请求</option>
          <option value="fast">仅常规请求</option>
        </select>
      </div>
    </div>

    <div class="row">
      <button class="btn primary" @click="applyFilters">查询</button>
      <button class="btn ghost" @click="focusRecentIssues">最近异常</button>
      <button class="btn ghost" @click="reset">重置</button>
    </div>

    <div class="table-layout">
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>请求概况</th>
              <th>状态</th>
              <th>耗时</th>
              <th>风险标签</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in rows" :key="item.ID">
              <td>#{{ item.ID }}</td>
              <td>
                <div class="cell-stack">
                  <strong>{{ item.method }} {{ item.path }}</strong>
                  <small>{{ item.ip }}</small>
                </div>
              </td>
              <td>
                <span
                  class="status-pill"
                  :class="statusClass(item.status)"
                >
                  {{ item.status }} · {{ statusGroupLabel(item.status) }}
                </span>
              </td>
              <td>
                <span
                  class="latency-pill"
                  :class="{ slow: isSlow(item.latency) }"
                >
                  {{ item.latency }} ms
                </span>
              </td>
              <td>
                <div class="risk-list">
                  <span
                    v-for="tag in riskTags(item)"
                    :key="`${item.ID}-${tag}`"
                    class="risk-tag"
                  >
                    {{ tag }}
                  </span>
                </div>
              </td>
              <td>
                <button class="btn ghost" @click="inspect(item)">查看</button>
              </td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="6" class="empty-cell">暂无符合条件的操作日志</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="side-panel">
        <div class="panel">
          <div class="panel-title">最近异常聚焦</div>
          <div v-if="recentAnomalies.length > 0" class="focus-list">
            <button
              v-for="entry in recentAnomalies"
              :key="entry.ID"
              class="focus-item"
              @click="inspect(entry)"
            >
              <div>
                <strong>{{ entry.method }} {{ entry.path }}</strong>
                <small>{{ entry.status }} · {{ entry.ip }}</small>
              </div>
              <span>{{ entry.latency }} ms</span>
            </button>
          </div>
          <div v-else class="empty-hint">最近未发现异常请求。</div>
        </div>

        <div class="panel" v-if="activeRow">
          <div class="panel-title">详情审阅</div>
          <div class="detail-grid">
            <div>
              <span>请求方法</span>
              <strong>{{ activeRow.method }}</strong>
            </div>
            <div>
              <span>请求路径</span>
              <strong>{{ activeRow.path }}</strong>
            </div>
            <div>
              <span>来源 IP</span>
              <strong>{{ activeRow.ip }}</strong>
            </div>
            <div>
              <span>状态码</span>
              <strong>{{ activeRow.status }}</strong>
            </div>
            <div>
              <span>状态分组</span>
              <strong>{{ statusGroupLabel(activeRow.status) }}</strong>
            </div>
            <div>
              <span>耗时</span>
              <strong>{{ activeRow.latency }} ms</strong>
            </div>
            <div>
              <span>审计结论</span>
              <strong>{{ reviewSummary(activeRow) }}</strong>
            </div>
            <div>
              <span>风险标签</span>
              <strong>{{ riskTags(activeRow).join(" / ") }}</strong>
            </div>
          </div>
          <div class="detail-note">
            {{ reviewDescription(activeRow) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getOperationLogsApi } from "../../api/admin";
import type { OperationLogInfo } from "../../types";

type StatusGroup = "" | "成功" | "重定向" | "客户端异常" | "服务端异常" | "未知";
type SpeedMode = "" | "slow" | "fast";

interface AuditFilters {
  keyword: string;
  method: string;
  path: string;
  status: string;
  statusGroup: StatusGroup;
  speedMode: SpeedMode;
}

const slowThreshold = 800;
const rows = ref<OperationLogInfo[]>([]);
const sourceRows = ref<OperationLogInfo[]>([]);
const activeRow = ref<OperationLogInfo | null>(null);
const filters = reactive<AuditFilters>({
  keyword: "",
  method: "",
  path: "",
  status: "",
  statusGroup: "",
  speedMode: "",
});

const abnormalRows = computed(() =>
  sourceRows.value.filter((item) => item.status >= 400),
);

const slowRows = computed(() =>
  sourceRows.value.filter((item) => isSlow(item.latency)),
);

const averageLatency = computed(() => {
  if (sourceRows.value.length === 0) {
    return 0;
  }
  const total = sourceRows.value.reduce((sum, item) => sum + item.latency, 0);
  return Math.round(total / sourceRows.value.length);
});

const maxLatency = computed(() =>
  sourceRows.value.reduce((max, item) => Math.max(max, item.latency), 0),
);

const abnormalRate = computed(() => {
  if (sourceRows.value.length === 0) {
    return 0;
  }
  return Math.round((abnormalRows.value.length / sourceRows.value.length) * 100);
});

const methodBreakdown = computed(() => buildCountBreakdown(sourceRows.value, (item) => item.method));

const statusBreakdown = computed<Array<{ count: number; label: Exclude<StatusGroup, ""> }>>(() =>
  buildCountBreakdown(sourceRows.value, (item) => statusGroupLabel(item.status)).map((entry) => ({
    ...entry,
    label: entry.label as Exclude<StatusGroup, "">,
  })),
);

const topPathBreakdown = computed(() => {
  const grouped = new Map<string, { count: number; abnormal: number; slow: number }>();
  sourceRows.value.forEach((item) => {
    const current = grouped.get(item.path) ?? { count: 0, abnormal: 0, slow: 0 };
    current.count += 1;
    if (item.status >= 400) {
      current.abnormal += 1;
    }
    if (isSlow(item.latency)) {
      current.slow += 1;
    }
    grouped.set(item.path, current);
  });
  return [...grouped.entries()]
    .map(([label, metrics]) => ({
      label,
      count: metrics.count,
      statusHint:
        metrics.abnormal > 0
          ? `异常 ${metrics.abnormal} 次`
          : metrics.slow > 0
            ? `慢请求 ${metrics.slow} 次`
            : "运行稳定",
    }))
    .sort((left, right) => right.count - left.count)
    .slice(0, 6);
});

const recentAnomalies = computed(() =>
  [...sourceRows.value]
    .filter((item) => item.status >= 400 || isSlow(item.latency))
    .sort((left, right) => right.ID - left.ID)
    .slice(0, 6),
);

async function load() {
  const result = await getOperationLogsApi();
  sourceRows.value = result.List;
  rows.value = result.List;
  if (!activeRow.value && result.List.length > 0) {
    activeRow.value = result.List[0] ?? null;
  }
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  rows.value = sourceRows.value.filter((item) => {
    const statusGroupMatched =
      !filters.statusGroup || statusGroupLabel(item.status) === filters.statusGroup;
    const speedMatched =
      !filters.speedMode ||
      (filters.speedMode === "slow" ? isSlow(item.latency) : !isSlow(item.latency));
    const keywordMatched =
      !keyword ||
      [item.ip, item.method, item.path, String(item.status), statusGroupLabel(item.status)]
        .join(" ")
        .toLowerCase()
        .includes(keyword);

    return (
      keywordMatched &&
      (!filters.method || item.method.toLowerCase().includes(filters.method.toLowerCase())) &&
      (!filters.path || item.path.toLowerCase().includes(filters.path.toLowerCase())) &&
      (!filters.status || String(item.status).includes(filters.status)) &&
      statusGroupMatched &&
      speedMatched
    );
  });
  if (rows.value.length > 0) {
    activeRow.value = rows.value[0] ?? null;
  }
}

function reset() {
  filters.keyword = "";
  filters.method = "";
  filters.path = "";
  filters.status = "";
  filters.statusGroup = "";
  filters.speedMode = "";
  rows.value = sourceRows.value;
  activeRow.value = rows.value[0] ?? null;
}

function inspect(row: OperationLogInfo) {
  activeRow.value = row;
}

function focusAbnormal() {
  filters.status = "";
  filters.statusGroup = "服务端异常";
  filters.speedMode = "";
  applyFilters();
}

function focusSlow() {
  filters.speedMode = "slow";
  filters.statusGroup = "";
  applyFilters();
}

function focusRecentIssues() {
  filters.keyword = "";
  filters.method = "";
  filters.path = "";
  filters.status = "";
  filters.statusGroup = "";
  filters.speedMode = "";
  rows.value = recentAnomalies.value;
  activeRow.value = rows.value[0] ?? null;
}

function toggleMethod(method: string) {
  filters.method = filters.method === method ? "" : method;
  applyFilters();
}

function toggleStatusGroup(group: StatusGroup) {
  filters.statusGroup = filters.statusGroup === group ? "" : group;
  applyFilters();
}

function togglePath(path: string) {
  filters.path = filters.path === path ? "" : path;
  applyFilters();
}

function isSlow(latency: number) {
  return latency >= slowThreshold;
}

function statusGroupLabel(status: number): Exclude<StatusGroup, ""> {
  if (status >= 200 && status < 300) {
    return "成功";
  }
  if (status >= 300 && status < 400) {
    return "重定向";
  }
  if (status >= 400 && status < 500) {
    return "客户端异常";
  }
  if (status >= 500) {
    return "服务端异常";
  }
  return "未知";
}

function riskTags(item: OperationLogInfo) {
  const tags: string[] = [];
  if (item.status >= 500) {
    tags.push("服务异常");
  } else if (item.status >= 400) {
    tags.push("访问失败");
  } else {
    tags.push("状态正常");
  }
  if (isSlow(item.latency)) {
    tags.push("慢请求");
  }
  if (item.method === "DELETE" || item.method === "PUT") {
    tags.push("高风险写操作");
  }
  if (item.path.includes("login") || item.path.includes("token")) {
    tags.push("鉴权链路");
  }
  return tags;
}

function reviewSummary(item: OperationLogInfo) {
  if (item.status >= 500) {
    return "建议优先排查后端异常";
  }
  if (item.status >= 400) {
    return "需确认请求参数与权限";
  }
  if (isSlow(item.latency)) {
    return "请求成功但存在性能风险";
  }
  return "链路稳定";
}

function reviewDescription(item: OperationLogInfo) {
  const base = `${item.method} ${item.path} 来自 ${item.ip}，状态 ${item.status}，耗时 ${item.latency} ms。`;
  if (item.status >= 500) {
    return `${base}该记录已进入服务端异常范围，建议结合错误日志与最近发布记录继续审阅。`;
  }
  if (item.status >= 400) {
    return `${base}该记录表现为客户端或权限侧异常，建议复核入参与角色授权。`;
  }
  if (isSlow(item.latency)) {
    return `${base}虽然请求已成功完成，但已超过慢请求阈值，适合继续追踪数据库或外部依赖耗时。`;
  }
  return `${base}当前未见明显异常，可作为稳定请求样本参考。`;
}

function statusClass(status: number) {
  if (status >= 500) {
    return "danger";
  }
  if (status >= 400) {
    return "warning";
  }
  return "success";
}

function buildCountBreakdown(
  list: OperationLogInfo[],
  pick: (item: OperationLogInfo) => string,
) {
  const grouped = new Map<string, number>();
  list.forEach((item) => {
    const key = pick(item) || "未分类";
    grouped.set(key, (grouped.get(key) ?? 0) + 1);
  });
  return [...grouped.entries()]
    .map(([label, count]) => ({ label, count }))
    .sort((left, right) => right.count - left.count);
}

onMounted(load);
</script>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.header,
.row,
.summary-grid,
.analysis-grid,
.toolbar-grid,
.table-layout,
.tag-list,
.risk-list,
.focus-list,
.detail-grid {
  display: flex;
}

.header,
.row {
  align-items: center;
  justify-content: space-between;
}

.row.compact {
  gap: 12px;
  justify-content: flex-end;
}

.title {
  margin: 0;
  font-size: 20px;
}

.subtitle {
  margin: 6px 0 0;
  color: #6b7280;
}

.summary-grid {
  gap: 12px;
  flex-wrap: wrap;
}

.summary-card,
.panel,
.side-panel {
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  background: #fff;
}

.summary-card {
  min-width: 180px;
  padding: 16px;
  flex: 1;
}

.summary-label,
.panel-title,
.field label,
.detail-grid span,
.cell-stack small,
.empty-hint,
.detail-note,
.summary-card small {
  color: #6b7280;
}

.summary-card strong {
  display: block;
  margin: 8px 0 4px;
  font-size: 24px;
}

.analysis-grid,
.toolbar-grid,
.table-layout,
.detail-grid {
  gap: 16px;
}

.analysis-grid,
.toolbar-grid {
  flex-wrap: wrap;
}

.panel {
  padding: 16px;
  flex: 1;
  min-width: 240px;
}

.panel-title {
  margin-bottom: 12px;
  font-weight: 600;
}

.tag-list,
.risk-list,
.focus-list {
  gap: 8px;
  flex-wrap: wrap;
}

.tag-button,
.hot-path-item,
.focus-item {
  border: 1px solid #d1d5db;
  border-radius: 10px;
  background: #f9fafb;
  cursor: pointer;
}

.tag-button {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
}

.tag-button.active,
.hot-path-item.active {
  border-color: #3b82f6;
  background: #eff6ff;
}

.hot-path-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.hot-path-item,
.focus-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  text-align: left;
}

.toolbar-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field input,
.field select {
  height: 36px;
  padding: 0 12px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
}

.btn {
  height: 36px;
  padding: 0 14px;
  border-radius: 8px;
  border: 1px solid #d1d5db;
  cursor: pointer;
  background: white;
}

.btn.primary {
  border-color: #2563eb;
  background: #2563eb;
  color: white;
}

.btn.ghost {
  color: #374151;
}

.table-layout {
  align-items: flex-start;
}

.data-table {
  flex: 1;
  overflow: auto;
}

.data-table table {
  width: 100%;
  border-collapse: collapse;
}

.data-table th,
.data-table td {
  padding: 12px;
  border-bottom: 1px solid #e5e7eb;
  vertical-align: top;
}

.cell-stack {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-pill,
.latency-pill,
.risk-tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
}

.status-pill.success,
.latency-pill,
.risk-tag {
  background: #ecfdf5;
  color: #047857;
}

.status-pill.warning {
  background: #fff7ed;
  color: #c2410c;
}

.status-pill.danger,
.latency-pill.slow {
  background: #fef2f2;
  color: #b91c1c;
}

.risk-list {
  min-height: 28px;
}

.side-panel {
  width: 360px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  border: none;
  background: transparent;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.detail-grid div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-note,
.empty-cell,
.empty-hint {
  font-size: 13px;
}

.detail-note {
  margin-top: 12px;
  line-height: 1.6;
}

.empty-cell {
  text-align: center;
  color: #9ca3af;
}

@media (max-width: 1200px) {
  .table-layout {
    flex-direction: column;
  }

  .side-panel {
    width: 100%;
  }
}
</style>
