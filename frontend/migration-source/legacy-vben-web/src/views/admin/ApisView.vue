<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">API 台账工作台</h3>
          <p class="subtitle">按路径、分组、描述与方法管理接口台账，并给出方法分布、风险判断与详情联动。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="load">刷新接口</button>
          <button class="btn primary" @click="openCreate">新增接口</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>接口总数</h4>
        <p class="stat-value">{{ rows.length }}</p>
        <p class="subtitle">当前拉取到的接口数量</p>
      </div>
      <div class="stat-card">
        <h4>分组数</h4>
        <p class="stat-value">{{ groupCount }}</p>
        <p class="subtitle">按 apiGroup 聚合</p>
      </div>
      <div class="stat-card">
        <h4>高风险接口</h4>
        <p class="stat-value">{{ riskyCount }}</p>
        <p class="subtitle">DELETE / 鉴权 / 管理类接口</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeApi?.path || "-" }}</p>
        <p class="subtitle">{{ activeApi ? apiRisk(activeApi) : "请选择接口查看详情" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">筛选与统计</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>路径</label>
            <input v-model.trim="filters.path" />
          </div>
          <div class="field">
            <label>分组</label>
            <input v-model.trim="filters.apiGroup" />
          </div>
          <div class="field">
            <label>描述</label>
            <input v-model.trim="filters.description" />
          </div>
          <div class="field">
            <label>方法</label>
            <input v-model.trim="filters.method" />
          </div>
        </div>
        <div class="row wrap">
          <button class="btn ghost" @click="load">查询</button>
          <button class="btn ghost" @click="reset">重置</button>
          <button class="btn ghost" :disabled="!activeApi" @click="copySummary">复制摘要</button>
        </div>
        <div class="tag-list top-gap">
          <button v-for="item in methodSummary" :key="item.name" class="tag action-tag" @click="filters.method = item.name">
            {{ item.name }} · {{ item.count }}
          </button>
          <button v-for="item in groupSummary" :key="item.name" class="tag action-tag" @click="filters.apiGroup = item.name">
            {{ item.name }} · {{ item.count }}
          </button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">接口详情</h3>
        <div v-if="activeApi" class="data-table">
          <table>
            <tbody>
              <tr><td>路径</td><td>{{ activeApi.path }}</td></tr>
              <tr><td>分组</td><td>{{ activeApi.apiGroup }}</td></tr>
              <tr><td>描述</td><td>{{ activeApi.description }}</td></tr>
              <tr><td>方法</td><td>{{ activeApi.method }}</td></tr>
              <tr><td>风险</td><td>{{ apiRisk(activeApi) }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">请选择接口查看详情。</p>
      </div>
    </div>

    <div class="card">
      <h3 class="title">接口列表</h3>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>路径</th>
              <th>分组</th>
              <th>描述</th>
              <th>方法</th>
              <th>风险</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in rows"
              :key="item.ID"
              :class="{ selected: activeApi?.ID === item.ID }"
              @click="inspect(item)"
            >
              <td>{{ item.ID }}</td>
              <td>{{ item.path }}</td>
              <td>{{ item.apiGroup }}</td>
              <td>{{ item.description }}</td>
              <td>{{ item.method }}</td>
              <td>{{ apiRisk(item) }}</td>
              <td><button class="btn ghost" @click.stop="inspect(item)">详情</button></td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="7">暂无接口数据</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="activeApi" class="card">
      <h3 class="title">{{ form.ID ? "编辑接口" : "新增接口" }}</h3>
      <div class="toolbar-grid">
        <div class="field">
          <label>路径</label>
          <input v-model.trim="form.path" />
        </div>
        <div class="field">
          <label>分组</label>
          <input v-model.trim="form.apiGroup" />
        </div>
        <div class="field">
          <label>描述</label>
          <input v-model.trim="form.description" />
        </div>
        <div class="field">
          <label>方法</label>
          <input v-model.trim="form.method" />
        </div>
      </div>

      <div v-if="validationIssues.length" class="state-banner error">
        <strong>保存前请修正：</strong>
        <ul>
          <li v-for="item in validationIssues" :key="item">{{ item }}</li>
        </ul>
      </div>

      <div class="row wrap">
        <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="save">
          {{ saving ? "保存中..." : "保存接口" }}
        </button>
        <button class="btn ghost" @click="inspect(activeApi)">重置</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getApiListApi, saveApiApi } from "../../api/admin";
import type { ApiInfo } from "../../types";

const rows = ref<ApiInfo[]>([]);
const activeApi = ref<ApiInfo | null>(null);
const message = ref("");
const error = ref("");
const loading = ref(false);
const saving = ref(false);
const filters = reactive({
  path: "",
  apiGroup: "",
  description: "",
  method: "",
});
const form = reactive({
  ID: undefined as number | undefined,
  path: "",
  apiGroup: "",
  description: "",
  method: "POST",
});

const groupCount = computed(() => new Set(rows.value.map((item) => item.apiGroup)).size);
const riskyCount = computed(() => rows.value.filter((item) => apiRisk(item) !== "低").length);
const methodSummary = computed(() => summarize(rows.value, (item) => item.method));
const groupSummary = computed(() => summarize(rows.value, (item) => item.apiGroup).slice(0, 6));
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.path.trim()) issues.push("接口路径不能为空。");
  if (!form.apiGroup.trim()) issues.push("接口分组不能为空。");
  if (!form.description.trim()) issues.push("接口描述不能为空。");
  if (!form.method.trim()) issues.push("请求方法不能为空。");
  return issues;
});

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const result = await getApiListApi(filters);
    rows.value = result.List;
    if (activeApi.value) {
      const refreshed = rows.value.find((item) => item.ID === activeApi.value?.ID);
      if (refreshed) {
        activeApi.value = refreshed;
        hydrateForm(refreshed);
      }
    }
    message.value = `已加载 ${rows.value.length} 条接口记录。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取接口失败";
  } finally {
    loading.value = false;
  }
}

function apiRisk(api: Pick<ApiInfo, "apiGroup" | "description" | "method" | "path">) {
  const text = `${api.path} ${api.apiGroup} ${api.description}`.toLowerCase();
  if (api.method === "DELETE" || text.includes("auth") || text.includes("token")) return "高";
  if (api.method === "PUT" || text.includes("system") || text.includes("admin")) return "中";
  return "低";
}

function summarize(list: ApiInfo[], picker: (item: ApiInfo) => string) {
  const bucket = new Map<string, number>();
  for (const item of list) {
    const key = picker(item) || "-";
    bucket.set(key, (bucket.get(key) ?? 0) + 1);
  }
  return [...bucket.entries()].map(([name, count]) => ({ name, count })).sort((a, b) => b.count - a.count);
}

function reset() {
  filters.path = "";
  filters.apiGroup = "";
  filters.description = "";
  filters.method = "";
  void load();
}

function inspect(api: ApiInfo) {
  activeApi.value = api;
  hydrateForm(api);
}

function hydrateForm(api: ApiInfo) {
  form.ID = api.ID;
  form.path = api.path;
  form.apiGroup = api.apiGroup;
  form.description = api.description;
  form.method = api.method;
}

function openCreate() {
  activeApi.value = {
    ID: 0,
    path: "/new/api/path",
    apiGroup: "新分组",
    description: "新接口",
    method: "POST",
  };
  hydrateForm(activeApi.value);
  form.ID = undefined;
  message.value = "已打开新接口草稿。";
}

async function copySummary() {
  if (!activeApi.value) return;
  const text = [
    `接口：${activeApi.value.path}`,
    `分组：${activeApi.value.apiGroup}`,
    `描述：${activeApi.value.description}`,
    `方法：${activeApi.value.method}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = `已复制接口 ${activeApi.value.path} 摘要。`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = `已切换为手动复制接口 ${activeApi.value.path} 摘要。`;
  }
}

async function save() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "接口信息不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    const saved = await saveApiApi({ ...form });
    message.value = `接口 ${saved.path} 已保存`;
    filters.path = saved.path;
    await load();
    const current = rows.value.find((item) => item.ID === saved.ID);
    if (current) {
      activeApi.value = current;
      hydrateForm(current);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存接口失败";
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>
