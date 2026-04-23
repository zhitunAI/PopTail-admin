<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">MCP 工具设计工作台</h3>
          <p class="subtitle">补齐工具定义、输入 schema、响应类型与质量提示，并持久化到 Rust 后端。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="loadTools">
            {{ loading ? "刷新中..." : "刷新工具" }}
          </button>
          <button class="btn ghost" @click="copyPreview">复制 JSON 预览</button>
          <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="saveTool">
            {{ saving ? "保存中..." : "保存工具" }}
          </button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>工具总数</h4>
        <p class="stat-value">{{ tools.length }}</p>
        <p class="subtitle">当前已登记的 MCP 工具数</p>
      </div>
      <div class="stat-card">
        <h4>参数总数</h4>
        <p class="stat-value">{{ draft.params.length }}</p>
        <p class="subtitle">当前草稿内的 schema 字段数</p>
      </div>
      <div class="stat-card">
        <h4>必填参数</h4>
        <p class="stat-value">{{ requiredCount }}</p>
        <p class="subtitle">{{ schemaHint }}</p>
      </div>
      <div class="stat-card">
        <h4>当前工具</h4>
        <p class="stat-value">{{ draft.name || "-" }}</p>
        <p class="subtitle">{{ selectedToolLabel }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">工具草稿</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>工具名称</label>
            <input v-model.trim="draft.name" placeholder="例如 CurrentTime" />
          </div>
          <div class="field">
            <label>输出类型</label>
            <select v-model="draft.responseType">
              <option value="text">text</option>
              <option value="image">image</option>
            </select>
          </div>
        </div>

        <div class="field">
          <label>工具描述</label>
          <textarea v-model.trim="draft.description" rows="3" placeholder="说明工具用途、边界与返回语义" />
        </div>

        <div class="row wrap">
          <button class="btn ghost" @click="addParam">新增参数</button>
          <button class="btn ghost" @click="appendPresetParams">补常用参数</button>
          <button class="btn ghost" :disabled="!tools.length" @click="fillFromLatest">回填最近工具</button>
        </div>

        <div v-if="validationIssues.length" class="state-banner error">
          <strong>保存前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>

        <div class="stack top-gap">
          <div v-for="(item, index) in draft.params" :key="`${item.name}-${index}`" class="param-row">
            <input v-model.trim="item.name" placeholder="参数名" />
            <input v-model.trim="item.description" placeholder="说明" />
            <select v-model="item.type">
              <option value="string">string</option>
              <option value="number">number</option>
              <option value="boolean">boolean</option>
              <option value="object">object</option>
              <option value="array">array</option>
            </select>
            <input v-model.trim="item.defaultValue" placeholder="默认值，可空" />
            <label class="tag"><input v-model="item.required" type="checkbox" /> 必填</label>
            <button class="btn ghost" @click="removeParam(index)">删除</button>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="title">Schema 质量提示</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>参数分布</td><td>{{ componentSummary }}</td></tr>
              <tr><td>响应语义</td><td>{{ responseAdvice }}</td></tr>
              <tr><td>质量判断</td><td>{{ qualityAdvice }}</td></tr>
              <tr><td>最近更新</td><td>{{ latestUpdatedAt }}</td></tr>
            </tbody>
          </table>
        </div>
        <div class="section top-gap">
          <h4>工作流提示</h4>
          <ul class="bullet-list">
            <li>优先保证工具名和描述足够稳定，再扩展参数。</li>
            <li>必填参数建议同时给出清晰描述，减少测试页联调歧义。</li>
            <li>对象与数组类型建议在后续联调页进一步验证默认值与输入结构。</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">JSON 预览</h3>
        <pre class="code-block">{{ preview }}</pre>
      </div>
      <div class="card">
        <div class="row between wrap">
          <h3 class="title">已登记工具</h3>
          <input v-model.trim="toolQuery" placeholder="按工具名或描述筛选" />
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>工具</th>
                <th>参数</th>
                <th>更新时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in filteredTools" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ Object.keys(item.inputSchema.properties).length }}</td>
                <td>{{ formatTime(item.updatedAt) }}</td>
                <td>
                  <div class="row wrap">
                    <button class="btn ghost" @click="fillFromTool(item)">回填</button>
                    <button class="btn ghost" @click="copyToolSummary(item)">复制摘要</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!filteredTools.length">
                <td colspan="4">暂无工具记录</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getMcpToolListApi, saveMcpToolApi } from "../../api/admin";
import type { McpToolDescriptor, McpToolParam } from "../../types";

const tools = ref<McpToolDescriptor[]>([]);
const loading = ref(false);
const saving = ref(false);
const message = ref("");
const error = ref("");
const toolQuery = ref("");
const draft = reactive({
  name: "CurrentTime",
  description: "返回当前时区时间，用于界面打点与任务追踪。",
  responseType: "text",
  params: [
    {
      name: "timezone",
      description: "IANA 时区名",
      type: "string",
      required: true,
      defaultValue: "Asia/Shanghai",
    },
  ] as McpToolParam[],
});

const requiredCount = computed(() => draft.params.filter((item) => item.required && item.name.trim()).length);
const selectedToolLabel = computed(() => {
  const current = tools.value.find((item) => item.name === draft.name.trim());
  return current ? `已存在，最近更新 ${formatTime(current.updatedAt)}` : "新工具草稿";
});
const filteredTools = computed(() => {
  const keyword = toolQuery.value.trim().toLowerCase();
  return tools.value.filter(
    (item) =>
      !keyword ||
      item.name.toLowerCase().includes(keyword) ||
      item.description.toLowerCase().includes(keyword),
  );
});
const preview = computed(() =>
  JSON.stringify(
    {
      name: draft.name.trim(),
      description: draft.description.trim(),
      inputSchema: {
        type: "object",
        properties: Object.fromEntries(
          draft.params
            .filter((item) => item.name.trim())
            .map((item) => [
              item.name.trim(),
              {
                type: item.type,
                description: item.description.trim(),
                defaultValue: item.defaultValue || undefined,
              },
            ]),
        ),
        required: draft.params.filter((item) => item.required && item.name.trim()).map((item) => item.name.trim()),
      },
      response: [{ type: draft.responseType }],
    },
    null,
    2,
  ),
);
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!draft.name.trim()) issues.push("工具名称不能为空。");
  if (!draft.description.trim()) issues.push("工具描述不能为空。");
  if (!draft.params.length) issues.push("至少保留一个参数，或显式建立空参数设计。");
  if (draft.params.some((item) => !item.name.trim())) issues.push("参数名不能为空。");
  if (draft.params.some((item) => item.required && !item.description.trim())) {
    issues.push("必填参数建议填写清晰说明。");
  }
  return issues;
});
const componentSummary = computed(() => {
  const counter = new Map<string, number>();
  for (const item of draft.params) {
    counter.set(item.type, (counter.get(item.type) ?? 0) + 1);
  }
  return [...counter.entries()].map(([key, count]) => `${key} × ${count}`).join(" / ") || "-";
});
const schemaHint = computed(() => (requiredCount.value ? `${requiredCount.value} 个字段需在测试页显式传入` : "当前没有必填参数"));
const responseAdvice = computed(() => (draft.responseType === "image" ? "适合产出视觉结果，测试时需关注输出格式。" : "适合文本或结构化结果，便于联调输出校验。"));
const qualityAdvice = computed(() => {
  if (validationIssues.value.length) return "当前 schema 仍有阻塞项";
  if (draft.params.some((item) => item.type === "object" || item.type === "array")) return "存在复杂参数，建议尽快到测试页做样本联调";
  return "当前 schema 结构清晰，可进入测试联调";
});
const latestUpdatedAt = computed(() => (tools.value[0] ? formatTime(tools.value[0].updatedAt) : "暂无记录"));

async function loadTools() {
  loading.value = true;
  error.value = "";
  try {
    tools.value = (await getMcpToolListApi()).sort((a, b) => b.updatedAt - a.updatedAt);
    message.value = `已加载 ${tools.value.length} 个 MCP 工具。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取 MCP 工具失败";
  } finally {
    loading.value = false;
  }
}

async function saveTool() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "工具草稿不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    await saveMcpToolApi({
      name: draft.name.trim(),
      description: draft.description.trim(),
      params: draft.params.map((item) => ({ ...item })),
      response: [{ type: draft.responseType }],
    });
    message.value = `工具 ${draft.name.trim()} 已保存`;
    await loadTools();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存 MCP 工具失败";
  } finally {
    saving.value = false;
  }
}

function addParam() {
  draft.params.push({
    name: "",
    description: "",
    type: "string",
    required: false,
    defaultValue: "",
  });
}

function appendPresetParams() {
  const presets: McpToolParam[] = [
    { name: "traceId", description: "链路追踪标识", type: "string", required: false, defaultValue: "" },
    { name: "locale", description: "输出语言区域", type: "string", required: false, defaultValue: "zh-CN" },
  ];
  for (const item of presets) {
    if (!draft.params.some((param) => param.name === item.name)) {
      draft.params.push({ ...item });
    }
  }
  message.value = "已补充常用参数模板。";
}

function removeParam(index: number) {
  if (draft.params.length === 1) {
    error.value = "至少保留一个参数定义。";
    return;
  }
  draft.params.splice(index, 1);
}

function fillFromTool(item: McpToolDescriptor) {
  draft.name = item.name;
  draft.description = item.description;
  draft.responseType = item.response[0]?.type || "text";
  const nextParams = Object.entries(item.inputSchema.properties).map(([name, value]) => ({
    name,
    description: value.description,
    type: value.type,
    required: item.inputSchema.required.includes(name),
    defaultValue: value.defaultValue || "",
  }));
  draft.params.splice(0, draft.params.length, ...(nextParams.length ? nextParams : [{
    name: "",
    description: "",
    type: "string",
    required: false,
    defaultValue: "",
  }]));
  message.value = `已回填工具 ${item.name}`;
}

function fillFromLatest() {
  if (tools.value[0]) {
    fillFromTool(tools.value[0]);
  }
}

async function copyPreview() {
  await copyText(preview.value, "已复制 MCP 工具 JSON 预览。");
}

async function copyToolSummary(item: McpToolDescriptor) {
  const text = [
    `工具：${item.name}`,
    `描述：${item.description}`,
    `参数：${Object.keys(item.inputSchema.properties).join(", ") || "-"}`,
    `输出：${item.response.map((entry) => entry.type).join(", ") || "-"}`,
  ].join("\n");
  await copyText(text, `已复制工具 ${item.name} 摘要。`);
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

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void loadTools();
});
</script>
