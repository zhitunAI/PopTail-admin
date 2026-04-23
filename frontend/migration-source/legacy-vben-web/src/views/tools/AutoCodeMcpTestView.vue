<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">MCP 联调测试工作台</h3>
          <p class="subtitle">联动服务状态、工具清单与真实参数校验结果，并保留最近测试记录用于回填。</p>
        </div>
        <div class="row wrap">
          <button class="btn primary" @click="startService">启动服务</button>
          <button class="btn ghost" @click="stopService">停用服务</button>
          <button class="btn ghost" :disabled="loading" @click="reload">刷新</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>服务状态</h4>
        <p class="stat-value">{{ status.state }}</p>
        <p class="subtitle">{{ status.reachable ? "服务可达" : "当前不可达" }}</p>
      </div>
      <div class="stat-card">
        <h4>可测工具</h4>
        <p class="stat-value">{{ tools.length }}</p>
        <p class="subtitle">当前已加载工具数</p>
      </div>
      <div class="stat-card">
        <h4>前置校验</h4>
        <p class="stat-value">{{ validationIssues.length ? `${validationIssues.length} 项` : "通过" }}</p>
        <p class="subtitle">{{ validationIssues[0] ?? "可以直接执行测试" }}</p>
      </div>
      <div class="stat-card">
        <h4>最近测试</h4>
        <p class="stat-value">{{ history.length }}</p>
        <p class="subtitle">{{ history[0]?.tool || "暂无历史记录" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">服务状态</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>服务状态</td><td>{{ status.state }}</td></tr>
              <tr><td>可达性</td><td>{{ status.reachable ? "reachable" : "stopped" }}</td></tr>
              <tr><td>服务地址</td><td>{{ status.baseURL }}</td></tr>
              <tr><td>健康检查</td><td>{{ status.healthURL }}</td></tr>
              <tr><td>启动时间</td><td>{{ status.startedAt ? formatTime(status.startedAt) : "-" }}</td></tr>
              <tr><td>说明</td><td>{{ status.message }}</td></tr>
              <tr><td>错误</td><td>{{ status.lastError || "-" }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">请求预检</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>当前工具</td><td>{{ selectedToolName || "-" }}</td></tr>
              <tr><td>参数字段</td><td>{{ selectedToolParamCount }}</td></tr>
              <tr><td>预检结论</td><td>{{ validationIssues.length ? "待修正" : "可执行" }}</td></tr>
            </tbody>
          </table>
        </div>
        <div v-if="validationIssues.length" class="state-banner error top-gap">
          <strong>执行前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">工具选择</h3>
        <div class="field">
          <label>工具</label>
          <select v-model="selectedToolName" @change="syncRequestFromTool">
            <option v-for="tool in tools" :key="tool.name" :value="tool.name">{{ tool.name }}</option>
          </select>
        </div>
        <div class="field">
          <label>请求体</label>
          <textarea v-model="requestPayload" rows="12" />
        </div>
        <div class="row wrap">
          <button class="btn primary" :disabled="!selectedToolName || validationIssues.length > 0 || acting" @click="runTest">
            {{ acting ? "执行中..." : "执行测试" }}
          </button>
          <button class="btn ghost" @click="copyPayload">复制请求体</button>
          <button class="btn ghost" :disabled="!history.length" @click="fillLatest">回填最近测试</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">执行结果</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>工具</td><td>{{ result.tool || "-" }}</td></tr>
              <tr><td>状态</td><td>{{ result.accepted ? "accepted" : "-" }}</td></tr>
              <tr><td>校验</td><td>{{ result.validation || "-" }}</td></tr>
              <tr><td>执行时间</td><td>{{ result.executedAt ? formatTime(result.executedAt) : "-" }}</td></tr>
              <tr><td>结果判断</td><td>{{ resultAdvice }}</td></tr>
            </tbody>
          </table>
        </div>
        <div class="row wrap top-gap">
          <button class="btn ghost" @click="copyResult">复制结果</button>
        </div>
        <pre class="code-block top-gap">{{ responsePreview }}</pre>
      </div>
    </div>

    <div class="card">
      <div class="row between wrap">
        <h3 class="title">最近测试记录</h3>
        <input v-model.trim="historyQuery" placeholder="按工具或校验信息筛选" />
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>时间</th>
              <th>工具</th>
              <th>状态</th>
              <th>校验</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in filteredHistory" :key="item.id">
              <td>{{ item.timeLabel }}</td>
              <td>{{ item.tool }}</td>
              <td>{{ item.accepted ? "accepted" : "rejected" }}</td>
              <td>{{ item.validation || "-" }}</td>
              <td>
                <div class="row wrap">
                  <button class="btn ghost" @click="restoreHistory(item)">回填</button>
                  <button class="btn ghost" @click="copyHistory(item)">复制</button>
                </div>
              </td>
            </tr>
            <tr v-if="!filteredHistory.length">
              <td colspan="5">暂无测试记录</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getMcpStatusApi,
  getMcpToolListApi,
  startMcpServiceApi,
  stopMcpServiceApi,
  testMcpToolApi,
} from "../../api/admin";
import type { McpServiceStatus, McpTestResult, McpToolDescriptor } from "../../types";

type TestHistoryRecord = {
  accepted: boolean;
  executedAt: number;
  id: string;
  output: Record<string, unknown>;
  payload: string;
  timeLabel: string;
  tool: string;
  validation: string;
};

const HISTORY_KEY = "gaa-mcp-test-history";

const status = ref<McpServiceStatus>({
  managed: true,
  state: "stopped",
  reachable: false,
  baseURL: "http://127.0.0.1:8889/mcp",
  healthURL: "http://127.0.0.1:8889/healthz",
  startedAt: null,
  lastError: "",
  message: "",
});
const tools = ref<McpToolDescriptor[]>([]);
const selectedToolName = ref("");
const requestPayload = ref("{}");
const message = ref("");
const error = ref("");
const loading = ref(false);
const acting = ref(false);
const history = ref<TestHistoryRecord[]>(readHistory());
const historyQuery = ref("");
const result = ref<McpTestResult>({
  tool: "",
  accepted: false,
  validation: "",
  executedAt: 0,
  output: {},
});

const selectedTool = computed(() => tools.value.find((item) => item.name === selectedToolName.value) ?? null);
const selectedToolParamCount = computed(() => Object.keys(selectedTool.value?.inputSchema.properties ?? {}).length);
const responsePreview = computed(() => JSON.stringify(result.value.output || {}, null, 2));
const resultAdvice = computed(() => {
  if (!result.value.tool) return "尚未执行测试";
  if (!result.value.accepted) return "结果未通过，建议先核对请求参数和工具 schema。";
  if (result.value.validation) return `已通过执行，但仍需关注校验反馈：${result.value.validation}`;
  return "执行通过，可继续在业务页集成该工具。";
});
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!selectedToolName.value) issues.push("请先选择工具。");
  if (!status.value.reachable) issues.push("当前 MCP 服务不可达。");
  try {
    const parsed = JSON.parse(requestPayload.value || "{}");
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      issues.push("请求体必须是 JSON 对象。");
    }
  } catch {
    issues.push("请求体不是合法 JSON。");
  }
  return issues;
});
const filteredHistory = computed(() => {
  const keyword = historyQuery.value.trim().toLowerCase();
  return history.value.filter(
    (item) =>
      !keyword ||
      item.tool.toLowerCase().includes(keyword) ||
      item.validation.toLowerCase().includes(keyword),
  );
});

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [nextStatus, nextTools] = await Promise.all([getMcpStatusApi(), getMcpToolListApi()]);
    status.value = nextStatus;
    tools.value = nextTools.sort((a, b) => b.updatedAt - a.updatedAt);
    if (!selectedToolName.value && nextTools[0]) {
      selectedToolName.value = nextTools[0].name;
    }
    syncRequestFromTool();
    message.value = "已刷新 MCP 服务状态与工具清单。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "刷新 MCP 状态失败";
  } finally {
    loading.value = false;
  }
}

function syncRequestFromTool() {
  const tool = selectedTool.value;
  if (!tool) {
    requestPayload.value = "{}";
    return;
  }
  const payload = Object.fromEntries(
    Object.entries(tool.inputSchema.properties).map(([name, value]) => [
      name,
      value.defaultValue ?? sampleValue(value.type),
    ]),
  );
  requestPayload.value = JSON.stringify(payload, null, 2);
}

async function startService() {
  status.value = await startMcpServiceApi();
  message.value = "MCP 服务已启动。";
}

async function stopService() {
  status.value = await stopMcpServiceApi();
  message.value = "MCP 服务已停用。";
}

async function runTest() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "测试请求不合法";
    return;
  }
  acting.value = true;
  error.value = "";
  try {
    const args = JSON.parse(requestPayload.value || "{}") as Record<string, unknown>;
    result.value = await testMcpToolApi({
      name: selectedToolName.value,
      args,
    });
    const record: TestHistoryRecord = {
      id: `${result.value.tool}-${result.value.executedAt}`,
      tool: result.value.tool,
      accepted: result.value.accepted,
      validation: result.value.validation,
      executedAt: result.value.executedAt,
      output: result.value.output,
      payload: requestPayload.value,
      timeLabel: formatTime(result.value.executedAt),
    };
    history.value = [record, ...history.value.filter((item) => item.id !== record.id)].slice(0, 12);
    persistHistory();
    message.value = `${selectedToolName.value} 已完成一次联调测试。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "执行 MCP 测试失败";
  } finally {
    acting.value = false;
  }
}

function fillLatest() {
  if (history.value[0]) {
    restoreHistory(history.value[0]);
  }
}

function restoreHistory(item: TestHistoryRecord) {
  selectedToolName.value = item.tool;
  requestPayload.value = item.payload;
  result.value = {
    tool: item.tool,
    accepted: item.accepted,
    validation: item.validation,
    executedAt: item.executedAt,
    output: item.output,
  };
  message.value = `已回填 ${item.tool} 的测试记录。`;
}

async function copyPayload() {
  await copyText(requestPayload.value, "已复制当前请求体。");
}

async function copyResult() {
  await copyText(responsePreview.value, "已复制当前测试结果。");
}

async function copyHistory(item: TestHistoryRecord) {
  const text = [
    `工具：${item.tool}`,
    `时间：${item.timeLabel}`,
    `状态：${item.accepted ? "accepted" : "rejected"}`,
    `校验：${item.validation || "-"}`,
    `请求：${item.payload}`,
  ].join("\n");
  await copyText(text, `已复制 ${item.tool} 历史测试摘要。`);
}

async function copyText(text: string, success: string) {
  try {
    await navigator.clipboard.writeText(text);
    message.value = success;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = "已切换为手动复制。";
  }
}

function sampleValue(type: string) {
  switch (type) {
    case "number":
      return 1;
    case "boolean":
      return true;
    case "array":
      return [];
    case "object":
      return {};
    default:
      return "";
  }
}

function readHistory() {
  try {
    const raw = window.localStorage.getItem(HISTORY_KEY);
    return raw ? (JSON.parse(raw) as TestHistoryRecord[]) : [];
  } catch {
    return [];
  }
}

function persistHistory() {
  window.localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value));
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void reload();
});
</script>
