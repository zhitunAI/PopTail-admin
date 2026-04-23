<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">参数工作台</h3>
          <p class="subtitle">展示系统参数键值、用途与变更影响，支持筛选、编辑、复制摘要和详情联动。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="load">刷新参数</button>
          <button class="btn primary" @click="openCreate">新增参数</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>参数总数</h4>
        <p class="stat-value">{{ sourceRows.length }}</p>
        <p class="subtitle">当前参数台账总量</p>
      </div>
      <div class="stat-card">
        <h4>高风险参数</h4>
        <p class="stat-value">{{ riskyCount }}</p>
        <p class="subtitle">涉及 token、redis、db 等关键字</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeRow?.key || "-" }}</p>
        <p class="subtitle">{{ activeRow ? paramAdvice(activeRow) : "请选择参数查看详情" }}</p>
      </div>
      <div class="stat-card">
        <h4>筛选命中</h4>
        <p class="stat-value">{{ rows.length }}</p>
        <p class="subtitle">{{ rows.length === sourceRows.length ? "未应用筛选" : "当前筛选结果" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">参数列表</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>参数键</label>
            <input v-model.trim="filters.key" placeholder="例如 system.redis" @keydown.enter="applyFilters" />
          </div>
          <div class="field">
            <label>参数值</label>
            <input v-model.trim="filters.value" placeholder="按值片段匹配" @keydown.enter="applyFilters" />
          </div>
        </div>
        <div class="row wrap">
          <button class="btn ghost" @click="applyFilters">应用筛选</button>
          <button class="btn ghost" @click="reset">重置筛选</button>
          <button class="btn ghost" :disabled="!activeRow" @click="copySummary">复制摘要</button>
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>键</th>
                <th>值</th>
                <th>说明</th>
                <th>风险</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in rows"
                :key="item.ID"
                :class="{ selected: activeRow?.ID === item.ID }"
                @click="inspect(item)"
              >
                <td>{{ item.ID }}</td>
                <td>{{ item.key }}</td>
                <td>{{ item.value }}</td>
                <td>{{ item.desc }}</td>
                <td>{{ paramRisk(item) }}</td>
                <td>
                  <div class="row wrap">
                    <button class="btn ghost" @click.stop="inspect(item)">查看</button>
                    <button class="btn ghost" @click.stop="edit(item)">编辑</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="6">暂无匹配参数</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">参数详情</h3>
        <div v-if="activeRow" class="stack">
          <div class="data-table">
            <table>
              <tbody>
                <tr><td>键</td><td>{{ activeRow.key }}</td></tr>
                <tr><td>值</td><td>{{ activeRow.value }}</td></tr>
                <tr><td>说明</td><td>{{ activeRow.desc }}</td></tr>
                <tr><td>风险等级</td><td>{{ paramRisk(activeRow) }}</td></tr>
                <tr><td>建议</td><td>{{ paramAdvice(activeRow) }}</td></tr>
              </tbody>
            </table>
          </div>
        </div>
        <p v-else class="subtitle">请选择参数查看详情。</p>
      </div>
    </div>

    <div class="card">
      <h3 class="title">{{ editing ? "编辑参数" : "新增参数" }}</h3>
      <div class="toolbar-grid">
        <div class="field">
          <label>参数键</label>
          <input v-model.trim="form.key" />
        </div>
        <div class="field">
          <label>参数值</label>
          <input v-model.trim="form.value" />
        </div>
      </div>
      <div class="field">
        <label>说明</label>
        <textarea v-model.trim="form.desc" rows="3" />
      </div>

      <div v-if="validationIssues.length" class="state-banner error">
        <strong>保存前请修正：</strong>
        <ul>
          <li v-for="item in validationIssues" :key="item">{{ item }}</li>
        </ul>
      </div>

      <div class="row wrap">
        <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="save">
          {{ saving ? "保存中..." : "保存参数" }}
        </button>
        <button class="btn ghost" @click="resetForm">清空</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getParamsListApi, saveParamApi } from "../../api/admin";
import type { ParamInfo } from "../../types";

const rows = ref<ParamInfo[]>([]);
const sourceRows = ref<ParamInfo[]>([]);
const activeRow = ref<ParamInfo | null>(null);
const editing = ref(false);
const message = ref("");
const error = ref("");
const loading = ref(false);
const saving = ref(false);
const filters = reactive({
  key: "",
  value: "",
});
const form = reactive({
  ID: undefined as number | undefined,
  key: "",
  value: "",
  desc: "",
});

const riskyCount = computed(() =>
  sourceRows.value.filter((item) => paramRisk(item) !== "低").length,
);

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.key.trim()) issues.push("参数键不能为空。");
  if (!form.value.trim()) issues.push("参数值不能为空。");
  if (!form.desc.trim()) issues.push("说明不能为空。");
  return issues;
});

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const result = await getParamsListApi();
    rows.value = result.List;
    sourceRows.value = result.List;
    if (activeRow.value) {
      const refreshed = sourceRows.value.find((item) => item.ID === activeRow.value?.ID);
      if (refreshed) {
        activeRow.value = refreshed;
      }
    }
    message.value = "已刷新参数台账。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取参数失败";
  } finally {
    loading.value = false;
  }
}

function paramRisk(row: Pick<ParamInfo, "desc" | "key" | "value">) {
  const text = `${row.key} ${row.value} ${row.desc}`.toLowerCase();
  if (["redis", "database", "db", "token", "session"].some((item) => text.includes(item))) return "高";
  if (["url", "host", "port", "compatibility"].some((item) => text.includes(item))) return "中";
  return "低";
}

function paramAdvice(row: Pick<ParamInfo, "desc" | "key" | "value">) {
  const risk = paramRisk(row);
  if (risk === "高") return "涉及关键基础设施或会话能力，修改前建议先确认联动影响。";
  if (risk === "中") return "建议先在运行态或系统配置中核对关联项。";
  return "常规参数，保存后可继续做页面回归。";
}

function applyFilters() {
  rows.value = sourceRows.value.filter((item) => {
    return (
      (!filters.key || item.key.includes(filters.key)) &&
      (!filters.value || item.value.includes(filters.value))
    );
  });
  message.value = `已筛选出 ${rows.value.length} 条参数。`;
}

function reset() {
  filters.key = "";
  filters.value = "";
  rows.value = sourceRows.value;
  message.value = "已重置参数筛选。";
}

function inspect(row: ParamInfo) {
  activeRow.value = row;
}

function edit(row: ParamInfo) {
  editing.value = true;
  form.ID = row.ID;
  form.key = row.key;
  form.value = row.value;
  form.desc = row.desc;
  activeRow.value = row;
}

function openCreate() {
  editing.value = false;
  resetForm();
  message.value = "已打开新参数草稿。";
}

function resetForm() {
  form.ID = undefined;
  form.key = "";
  form.value = "";
  form.desc = "";
  error.value = "";
}

async function copySummary() {
  if (!activeRow.value) return;
  const text = `参数 ${activeRow.value.key}\n值：${activeRow.value.value}\n说明：${activeRow.value.desc}`;
  try {
    await navigator.clipboard.writeText(text);
    message.value = `已复制参数 ${activeRow.value.key} 摘要。`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = `已切换为手动复制参数 ${activeRow.value.key} 摘要。`;
  }
}

async function save() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "参数信息不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    const saved = await saveParamApi(form);
    message.value = `参数 ${saved.key} 已保存`;
    await load();
    activeRow.value = saved;
    edit(saved);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存参数失败";
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>
