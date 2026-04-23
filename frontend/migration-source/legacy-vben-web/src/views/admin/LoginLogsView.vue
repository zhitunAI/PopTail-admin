<template>
  <div class="page-shell">
    <section class="card">
      <div class="hero-row">
        <div>
          <h3 class="title">登录日志工作台</h3>
          <p class="subtitle">
            聚合查看登录成功、失败与高风险来源，支持快速筛选、复制、追踪最近失败记录。
          </p>
        </div>
        <div class="hero-actions">
          <button class="btn primary" :disabled="loading" @click="loadLogs">
            {{ loading ? "刷新中..." : "刷新日志" }}
          </button>
          <button class="btn ghost" :disabled="!latestFailedEntry" @click="focusLatestFailure">
            聚焦最近失败
          </button>
          <button class="btn ghost" :disabled="!selectedRecord" @click="copySelectedSummary">
            复制详情摘要
          </button>
        </div>
      </div>

      <div v-if="message" class="feedback-banner">{{ message }}</div>

      <div class="summary-grid">
        <article class="summary-card">
          <span class="summary-label">日志总数</span>
          <strong class="summary-value">{{ dashboard.total }}</strong>
          <span class="summary-note">当前可分析样本</span>
        </article>
        <article class="summary-card success">
          <span class="summary-label">成功次数</span>
          <strong class="summary-value">{{ dashboard.success }}</strong>
          <span class="summary-note">{{ dashboard.successRate }}</span>
        </article>
        <article class="summary-card danger">
          <span class="summary-label">失败次数</span>
          <strong class="summary-value">{{ dashboard.failed }}</strong>
          <span class="summary-note">{{ dashboard.failureRate }}</span>
        </article>
        <article class="summary-card warning">
          <span class="summary-label">涉及来源 IP</span>
          <strong class="summary-value">{{ dashboard.uniqueIpCount }}</strong>
          <span class="summary-note">失败来源 {{ dashboard.failedIpCount }} 个</span>
        </article>
      </div>

      <div class="toolbar-grid">
        <div class="field">
          <label>关键词</label>
          <input
            v-model.trim="filters.keyword"
            placeholder="按用户名、IP、结果说明搜索"
            @keyup.enter="applyFilters"
          />
        </div>
        <div class="field">
          <label>状态</label>
          <select v-model="filters.status">
            <option value="">全部</option>
            <option value="success">成功</option>
            <option value="failed">失败</option>
          </select>
        </div>
        <div class="field">
          <label>来源分组</label>
          <select v-model="filters.origin">
            <option value="">全部</option>
            <option value="local">本机</option>
            <option value="private">内网</option>
            <option value="public">公网</option>
            <option value="unknown">未识别</option>
          </select>
        </div>
        <div class="field">
          <label>IP 包含</label>
          <input
            v-model.trim="filters.ipKeyword"
            placeholder="例如 192.168 / 10.0"
            @keyup.enter="applyFilters"
          />
        </div>
      </div>

      <div class="row wrap">
        <button class="btn primary" @click="applyFilters">应用筛选</button>
        <button class="btn ghost" @click="clearFilters">清空筛选</button>
        <button class="btn ghost" :disabled="!topFailedSource" @click="focusTopFailedSource">
          锁定高频失败来源
        </button>
        <span class="hint">
          当前结果 {{ filteredRows.length }} / {{ sourceRows.length }}
        </span>
      </div>

      <div class="spotlight-grid">
        <article class="spotlight-card">
          <div class="spotlight-header">
            <h4>失败来源排行</h4>
            <span>{{ failureSources.length }} 个来源</span>
          </div>
          <ul v-if="failureSources.length" class="rank-list">
            <li
              v-for="entry in failureSources.slice(0, 5)"
              :key="entry.ip"
              class="rank-item"
            >
              <button class="link-btn" @click="applySourceFocus(entry.ip)">
                {{ entry.ip }}
              </button>
              <span class="rank-meta">{{ entry.count }} 次失败 · {{ entry.originLabel }}</span>
            </li>
          </ul>
          <p v-else class="empty-text">当前没有失败来源数据。</p>
        </article>

        <article class="spotlight-card">
          <div class="spotlight-header">
            <h4>重点账号</h4>
            <span>{{ highlightedUsers.length }} 个账号</span>
          </div>
          <ul v-if="highlightedUsers.length" class="rank-list">
            <li
              v-for="entry in highlightedUsers.slice(0, 5)"
              :key="entry.username"
              class="rank-item"
            >
              <button class="link-btn" @click="applyUserFocus(entry.username)">
                {{ entry.username || "未命名账号" }}
              </button>
              <span class="rank-meta">
                失败 {{ entry.failures }} / 成功 {{ entry.successes }}
              </span>
            </li>
          </ul>
          <p v-else class="empty-text">当前没有可聚焦的账号。</p>
        </article>
      </div>
    </section>

    <section class="workspace-grid">
      <article class="card">
        <div class="table-header">
          <div>
            <h3 class="title small">日志列表</h3>
            <p class="subtitle small">点击记录可查看详情并复制摘要。</p>
          </div>
          <span class="hint">失败率 {{ dashboard.failureRate }}</span>
        </div>

        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>用户名</th>
                <th>来源 IP</th>
                <th>来源分组</th>
                <th>状态</th>
                <th>结果说明</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody v-if="filteredRows.length">
              <tr
                v-for="item in filteredRows"
                :key="item.ID"
                :class="{
                  selected: selectedRecord?.ID === item.ID,
                  failed: !item.status,
                }"
                @click="selectRecord(item)"
              >
                <td>{{ item.ID }}</td>
                <td>{{ item.username || "-" }}</td>
                <td>{{ item.ip || "-" }}</td>
                <td>{{ resolveOriginLabel(item.ip) }}</td>
                <td>
                  <span :class="['badge', item.status ? 'success' : 'danger']">
                    {{ item.status ? "成功" : "失败" }}
                  </span>
                </td>
                <td>{{ resolveOutcome(item) }}</td>
                <td>
                  <div class="inline-actions">
                    <button class="btn ghost small" @click.stop="selectRecord(item)">
                      详情
                    </button>
                    <button class="btn ghost small" @click.stop="copyRecord(item)">
                      复制
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
            <tbody v-else>
              <tr>
                <td colspan="7" class="empty-cell">当前筛选条件下没有匹配记录。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </article>

      <article class="card detail-card">
        <div class="table-header">
          <div>
            <h3 class="title small">登录详情</h3>
            <p class="subtitle small">支持复制、快速定位同来源与同账号记录。</p>
          </div>
          <button class="btn ghost small" :disabled="!selectedRecord" @click="copySelectedSummary">
            复制详情
          </button>
        </div>

        <div v-if="selectedRecord" class="detail-grid">
          <div class="detail-item">
            <span>记录 ID</span>
            <strong>#{{ selectedRecord.ID }}</strong>
          </div>
          <div class="detail-item">
            <span>用户名</span>
            <strong>{{ selectedRecord.username || "-" }}</strong>
          </div>
          <div class="detail-item">
            <span>来源 IP</span>
            <strong>{{ selectedRecord.ip || "-" }}</strong>
          </div>
          <div class="detail-item">
            <span>来源分组</span>
            <strong>{{ resolveOriginLabel(selectedRecord.ip) }}</strong>
          </div>
          <div class="detail-item">
            <span>状态</span>
            <strong>{{ selectedRecord.status ? "成功" : "失败" }}</strong>
          </div>
          <div class="detail-item">
            <span>结果说明</span>
            <strong>{{ resolveOutcome(selectedRecord) }}</strong>
          </div>

          <div class="detail-actions">
            <button class="btn primary small" @click="applyUserFocus(selectedRecord.username)">
              查看同账号
            </button>
            <button class="btn ghost small" @click="applySourceFocus(selectedRecord.ip)">
              查看同来源
            </button>
          </div>

          <div class="detail-note">
            <h4>研判提示</h4>
            <ul>
              <li>同账号多次失败可用于追踪凭证问题或异常尝试。</li>
              <li>同来源多次失败可用于判断集中撞库、代理出口或设备异常。</li>
              <li>成功但来源异常时，建议联动操作日志继续核查后续行为。</li>
            </ul>
          </div>
        </div>
        <div v-else class="empty-panel">
          请从左侧列表选择一条日志，查看详细上下文与快捷操作。
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getLoginLogsApi } from "../../api/admin";
import type { LoginLogInfo } from "../../types";

type StatusFilter = "" | "success" | "failed";
type OriginFilter = "" | "local" | "private" | "public" | "unknown";

interface FailureSourceSummary {
  count: number;
  ip: string;
  originLabel: string;
}

interface UserFailureSummary {
  failures: number;
  successes: number;
  username: string;
}

const loading = ref(false);
const message = ref("");
const sourceRows = ref<LoginLogInfo[]>([]);
const filteredRows = ref<LoginLogInfo[]>([]);
const selectedRecord = ref<LoginLogInfo | null>(null);

const filters = reactive<{
  ipKeyword: string;
  keyword: string;
  origin: OriginFilter;
  status: StatusFilter;
}>({
  ipKeyword: "",
  keyword: "",
  origin: "",
  status: "",
});

const dashboard = computed(() => {
  const total = sourceRows.value.length;
  const success = sourceRows.value.filter((item) => item.status).length;
  const failed = total - success;
  const uniqueIp = new Set(sourceRows.value.map((item) => item.ip).filter(Boolean));
  const failedIp = new Set(
    sourceRows.value.filter((item) => !item.status).map((item) => item.ip).filter(Boolean),
  );

  return {
    failed,
    failedIpCount: failedIp.size,
    failureRate: total ? `${((failed / total) * 100).toFixed(1)}%` : "0.0%",
    success,
    successRate: total ? `${((success / total) * 100).toFixed(1)}%` : "0.0%",
    total,
    uniqueIpCount: uniqueIp.size,
  };
});

const latestFailedEntry = computed(() => {
  return [...sourceRows.value]
    .filter((item) => !item.status)
    .sort((left, right) => right.ID - left.ID)[0] ?? null;
});

const failureSources = computed<FailureSourceSummary[]>(() => {
  const bucket = new Map<string, number>();
  for (const item of sourceRows.value) {
    if (item.status || !item.ip) {
      continue;
    }
    bucket.set(item.ip, (bucket.get(item.ip) ?? 0) + 1);
  }
  return [...bucket.entries()]
    .map(([ip, count]) => ({
      count,
      ip,
      originLabel: resolveOriginLabel(ip),
    }))
    .sort((left, right) => right.count - left.count || right.ip.localeCompare(left.ip));
});

const highlightedUsers = computed<UserFailureSummary[]>(() => {
  const bucket = new Map<string, UserFailureSummary>();
  for (const item of sourceRows.value) {
    const key = item.username || "未命名账号";
    const current = bucket.get(key) ?? {
      failures: 0,
      successes: 0,
      username: key,
    };
    if (item.status) {
      current.successes += 1;
    } else {
      current.failures += 1;
    }
    bucket.set(key, current);
  }
  return [...bucket.values()]
    .filter((item) => item.failures > 0)
    .sort((left, right) => right.failures - left.failures || right.successes - left.successes);
});

const topFailedSource = computed(() => failureSources.value[0] ?? null);

async function loadLogs() {
  loading.value = true;
  message.value = "";
  try {
    const result = await getLoginLogsApi();
    const sorted = [...result.List].sort((left, right) => right.ID - left.ID);
    sourceRows.value = sorted;
    filteredRows.value = sorted;
    if (selectedRecord.value) {
      selectedRecord.value =
        sorted.find((item) => item.ID === selectedRecord.value?.ID) ?? sorted[0] ?? null;
    } else {
      selectedRecord.value = sorted[0] ?? null;
    }
    message.value = sorted.length
      ? `已加载 ${sorted.length} 条登录日志`
      : "当前还没有可展示的登录日志";
    performApplyFilters(false);
  } catch (error) {
    sourceRows.value = [];
    filteredRows.value = [];
    selectedRecord.value = null;
    message.value = error instanceof Error ? error.message : "登录日志加载失败";
  } finally {
    loading.value = false;
  }
}

function performApplyFilters(showMessage = true) {
  const keyword = filters.keyword.trim().toLowerCase();
  const ipKeyword = filters.ipKeyword.trim().toLowerCase();

  filteredRows.value = sourceRows.value.filter((item) => {
    const outcome = resolveOutcome(item).toLowerCase();
    const username = (item.username || "").toLowerCase();
    const ip = (item.ip || "").toLowerCase();
    const origin = resolveOrigin(item.ip);

    const matchKeyword =
      !keyword || username.includes(keyword) || ip.includes(keyword) || outcome.includes(keyword);
    const matchStatus =
      !filters.status ||
      (filters.status === "success" ? item.status : !item.status);
    const matchOrigin = !filters.origin || filters.origin === origin;
    const matchIp = !ipKeyword || ip.includes(ipKeyword);

    return matchKeyword && matchStatus && matchOrigin && matchIp;
  });

  if (
    selectedRecord.value &&
    !filteredRows.value.some((item) => item.ID === selectedRecord.value?.ID)
  ) {
    selectedRecord.value = filteredRows.value[0] ?? null;
  }

  if (showMessage) {
    message.value = `筛选完成，命中 ${filteredRows.value.length} 条记录`;
  }
}

function applyFilters() {
  performApplyFilters(true);
}

function clearFilters() {
  filters.keyword = "";
  filters.status = "";
  filters.origin = "";
  filters.ipKeyword = "";
  filteredRows.value = [...sourceRows.value];
  selectedRecord.value = filteredRows.value[0] ?? null;
  message.value = "已清空筛选条件";
}

function selectRecord(record: LoginLogInfo) {
  selectedRecord.value = record;
}

function focusLatestFailure() {
  if (!latestFailedEntry.value) {
    message.value = "当前没有失败日志";
    return;
  }
  filters.status = "failed";
  filters.keyword = latestFailedEntry.value.username || "";
  filters.ipKeyword = latestFailedEntry.value.ip || "";
  filters.origin = resolveOrigin(latestFailedEntry.value.ip);
  performApplyFilters(false);
  selectedRecord.value = latestFailedEntry.value;
  message.value = `已聚焦最近失败记录 #${latestFailedEntry.value.ID}`;
}

function focusTopFailedSource() {
  if (!topFailedSource.value) {
    message.value = "当前没有高频失败来源";
    return;
  }
  applySourceFocus(topFailedSource.value.ip);
}

function applyUserFocus(username: string) {
  filters.keyword = username || "";
  filters.status = "";
  performApplyFilters(false);
  selectedRecord.value =
    filteredRows.value.find((item) => item.username === username) ?? filteredRows.value[0] ?? null;
  message.value = username ? `已聚焦账号 ${username}` : "已聚焦未命名账号";
}

function applySourceFocus(ip: string) {
  filters.ipKeyword = ip || "";
  filters.origin = resolveOrigin(ip);
  filters.status = "failed";
  performApplyFilters(false);
  selectedRecord.value =
    filteredRows.value.find((item) => item.ip === ip) ?? filteredRows.value[0] ?? null;
  message.value = ip ? `已锁定来源 ${ip}` : "已锁定未识别来源";
}

function resolveOutcome(record: LoginLogInfo) {
  if (record.status) {
    return record.errorMessage || "登录成功";
  }
  return record.errorMessage || "登录失败";
}

function resolveOrigin(ip: string) {
  const value = (ip || "").trim();
  if (!value) {
    return "unknown";
  }
  if (value === "::1" || value === "127.0.0.1" || value.startsWith("localhost")) {
    return "local";
  }
  if (
    value.startsWith("10.") ||
    value.startsWith("192.168.") ||
    value.startsWith("172.16.") ||
    value.startsWith("172.17.") ||
    value.startsWith("172.18.") ||
    value.startsWith("172.19.") ||
    value.startsWith("172.2") ||
    value.startsWith("172.30.") ||
    value.startsWith("172.31.")
  ) {
    return "private";
  }
  return "public";
}

function resolveOriginLabel(ip: string) {
  const origin = resolveOrigin(ip);
  if (origin === "local") {
    return "本机";
  }
  if (origin === "private") {
    return "内网";
  }
  if (origin === "public") {
    return "公网";
  }
  return "未识别";
}

async function copyRecord(record: LoginLogInfo) {
  const summary = buildRecordSummary(record);
  await navigator.clipboard.writeText(summary);
  selectedRecord.value = record;
  message.value = `已复制记录 #${record.ID} 摘要`;
}

async function copySelectedSummary() {
  if (!selectedRecord.value) {
    message.value = "请先选择一条日志";
    return;
  }
  await copyRecord(selectedRecord.value);
}

function buildRecordSummary(record: LoginLogInfo) {
  return [
    `登录日志 #${record.ID}`,
    `账号：${record.username || "-"}`,
    `来源 IP：${record.ip || "-"}`,
    `来源分组：${resolveOriginLabel(record.ip)}`,
    `状态：${record.status ? "成功" : "失败"}`,
    `结果说明：${resolveOutcome(record)}`,
  ].join("\n");
}

onMounted(loadLogs);
</script>

<style scoped>
.page-shell {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  box-shadow: 0 10px 30px rgb(15 23 42 / 6%);
  padding: 20px;
}

.hero-row,
.table-header,
.row,
.spotlight-header,
.inline-actions,
.hero-actions {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.hero-row,
.table-header {
  align-items: flex-start;
}

.hero-actions,
.inline-actions,
.row.wrap {
  flex-wrap: wrap;
  justify-content: flex-start;
}

.title {
  color: #111827;
  font-size: 22px;
  font-weight: 700;
  margin: 0;
}

.title.small {
  font-size: 18px;
}

.subtitle {
  color: #6b7280;
  margin: 8px 0 0;
}

.subtitle.small {
  font-size: 13px;
  margin-top: 4px;
}

.feedback-banner {
  background: #eff6ff;
  border: 1px solid #bfdbfe;
  border-radius: 12px;
  color: #1d4ed8;
  margin-top: 16px;
  padding: 10px 12px;
}

.summary-grid,
.spotlight-grid,
.workspace-grid,
.toolbar-grid,
.detail-grid {
  display: grid;
  gap: 12px;
}

.summary-grid {
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: 18px;
}

.spotlight-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-top: 16px;
}

.workspace-grid {
  grid-template-columns: minmax(0, 1.8fr) minmax(320px, 1fr);
}

.toolbar-grid {
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: 18px;
}

.summary-card,
.spotlight-card,
.detail-item {
  background: linear-gradient(180deg, #fff, #f8fafc);
  border: 1px solid #e5e7eb;
  border-radius: 14px;
  padding: 14px;
}

.summary-card.success {
  border-color: #bbf7d0;
}

.summary-card.danger {
  border-color: #fecaca;
}

.summary-card.warning {
  border-color: #fde68a;
}

.summary-label,
.detail-item span,
.hint,
.rank-meta {
  color: #6b7280;
  font-size: 13px;
}

.summary-value {
  color: #111827;
  display: block;
  font-size: 28px;
  margin: 10px 0 6px;
}

.summary-note {
  color: #475569;
  font-size: 13px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  color: #374151;
  font-size: 13px;
  font-weight: 600;
}

.field input,
.field select {
  border: 1px solid #d1d5db;
  border-radius: 10px;
  font-size: 14px;
  min-height: 40px;
  outline: none;
  padding: 0 12px;
}

.field input:focus,
.field select:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgb(59 130 246 / 15%);
}

.btn {
  border: 1px solid #d1d5db;
  border-radius: 10px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  min-height: 38px;
  padding: 0 14px;
}

.btn.small {
  font-size: 12px;
  min-height: 32px;
  padding: 0 10px;
}

.btn.primary {
  background: #2563eb;
  border-color: #2563eb;
  color: #fff;
}

.btn.ghost {
  background: #fff;
  color: #334155;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.rank-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  list-style: none;
  margin: 14px 0 0;
  padding: 0;
}

.rank-item {
  align-items: center;
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.link-btn {
  background: transparent;
  border: none;
  color: #2563eb;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  padding: 0;
  text-align: left;
}

.empty-text,
.empty-panel,
.empty-cell {
  color: #6b7280;
  text-align: center;
}

.data-table {
  margin-top: 16px;
  overflow-x: auto;
}

.data-table table {
  border-collapse: collapse;
  width: 100%;
}

.data-table th,
.data-table td {
  border-bottom: 1px solid #e5e7eb;
  font-size: 14px;
  padding: 12px 10px;
  text-align: left;
  vertical-align: top;
}

.data-table th {
  color: #475569;
  font-weight: 700;
}

.data-table tbody tr {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.data-table tbody tr:hover {
  background: #f8fafc;
}

.data-table tbody tr.selected {
  background: #eff6ff;
}

.data-table tbody tr.failed td {
  background-image: linear-gradient(180deg, rgb(254 242 242 / 35%), transparent);
}

.badge {
  border-radius: 999px;
  display: inline-flex;
  font-size: 12px;
  font-weight: 700;
  padding: 4px 10px;
}

.badge.success {
  background: #dcfce7;
  color: #166534;
}

.badge.danger {
  background: #fee2e2;
  color: #b91c1c;
}

.detail-card {
  min-height: 100%;
}

.detail-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-top: 16px;
}

.detail-item strong {
  color: #111827;
  display: block;
  margin-top: 8px;
}

.detail-actions,
.detail-note {
  grid-column: 1 / -1;
}

.detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.detail-note {
  background: #f8fafc;
  border-radius: 14px;
  padding: 14px;
}

.detail-note h4 {
  color: #111827;
  margin: 0 0 10px;
}

.detail-note ul {
  color: #475569;
  margin: 0;
  padding-left: 18px;
}

@media (max-width: 1080px) {
  .summary-grid,
  .toolbar-grid,
  .workspace-grid,
  .spotlight-grid,
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
