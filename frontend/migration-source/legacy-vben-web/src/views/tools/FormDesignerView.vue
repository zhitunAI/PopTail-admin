<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">表单设计工作台</h3>
          <p class="subtitle">支持字段设计、代码生成、台账入库、历史回填与设计校验，不再只是本地临时预览。</p>
        </div>
        <div class="tag-list">
          <span class="tag">字段 {{ fields.length }}</span>
          <span class="tag">组件 {{ componentSummary }}</span>
          <span class="tag">草稿 {{ records.length }}</span>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>表单名称</h4>
        <p class="stat-value">{{ formName || "-" }}</p>
        <p class="subtitle">当前编辑中的表单标识</p>
      </div>
      <div class="stat-card">
        <h4>字段数</h4>
        <p class="stat-value">{{ fields.length }}</p>
        <p class="subtitle">已定义字段总数</p>
      </div>
      <div class="stat-card">
        <h4>校验项</h4>
        <p class="stat-value">{{ validationIssues.length }}</p>
        <p class="subtitle">{{ validationIssues[0] ?? "当前草稿可生成" }}</p>
      </div>
      <div class="stat-card">
        <h4>最近保存</h4>
        <p class="stat-value">{{ lastSavedLabel }}</p>
        <p class="subtitle">来自 auto-code 台账记录</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">设计区</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>表单名称</label>
            <input v-model.trim="formName" placeholder="例如 risk-policy-form" />
          </div>
          <div class="field">
            <label>提交按钮</label>
            <input v-model.trim="submitLabel" placeholder="例如 保存" />
          </div>
        </div>

        <div class="row wrap">
          <button class="btn ghost" @click="addField">新增字段</button>
          <button class="btn ghost" @click="appendCommonFields">补常用字段</button>
          <button class="btn ghost" @click="copyCode">复制模板</button>
          <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="saveDraft">
            {{ saving ? "保存中..." : "保存草稿" }}
          </button>
        </div>

        <div v-if="validationIssues.length" class="state-banner error">
          <strong>生成前请先修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>

        <div class="stack top-gap">
          <div v-for="(item, index) in fields" :key="`${item.name}-${index}`" class="field-row">
            <input v-model.trim="item.label" placeholder="字段标题" />
            <input v-model.trim="item.name" placeholder="字段名" />
            <select v-model="item.type">
              <option value="input">输入框</option>
              <option value="select">选择器</option>
              <option value="switch">开关</option>
              <option value="date-picker">日期</option>
            </select>
            <input v-model.trim="item.placeholder" placeholder="占位提示" />
            <button class="btn ghost" @click="removeField(index)">删除</button>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="title">表单摘要</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>组件分布</td><td>{{ componentSummary }}</td></tr>
              <tr><td>默认提交文案</td><td>{{ submitLabel || "保存" }}</td></tr>
              <tr><td>首个字段</td><td>{{ fields[0]?.label || "-" }}</td></tr>
              <tr><td>设计建议</td><td>{{ designAdvice }}</td></tr>
            </tbody>
          </table>
        </div>
        <div class="section top-gap">
          <h4>工作流提示</h4>
          <ul class="bullet-list">
            <li>先保证字段名和中文说明完整，再复制模板给后续页面或生成链路。</li>
            <li>保存草稿后可在最近草稿中回填，避免重复录入。</li>
            <li>如包含 select、switch 等组件，建议后续补充对应选项和默认值策略。</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">模板代码</h3>
        <pre class="code-block">{{ preview }}</pre>
      </div>
      <div class="card">
        <div class="row between wrap">
          <h3 class="title">最近草稿</h3>
          <input v-model.trim="recordQuery" placeholder="按名称筛选草稿" />
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr><th>时间</th><th>名称</th><th>字段数</th><th>操作</th></tr>
            </thead>
            <tbody>
              <tr v-for="item in filteredRecords" :key="item.ID">
                <td>{{ formatTime(item.createdAt) }}</td>
                <td>{{ item.payload.formName }}</td>
                <td>{{ item.payload.fields.length }}</td>
                <td>
                  <div class="row wrap">
                    <button class="btn ghost" @click="hydrate(item.payload)">回填</button>
                    <button class="btn ghost" @click="copyRecord(item.payload)">复制摘要</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!filteredRecords.length">
                <td colspan="4">暂无表单草稿</td>
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
import { getAutoCodeRegistryApi, saveAutoCodeRegistryApi } from "../../api/admin";
import type { AutoCodeRegistryRecord } from "../../types";

type FormField = {
  label: string;
  name: string;
  type: string;
  placeholder: string;
};

type FormDraftPayload = {
  kind: "form-designer";
  formName: string;
  submitLabel: string;
  fields: FormField[];
};

const formName = ref("risk-policy-form");
const submitLabel = ref("保存");
const fields = reactive<FormField[]>([
  { label: "名称", name: "name", type: "input", placeholder: "请输入名称" },
  { label: "状态", name: "enabled", type: "switch", placeholder: "" },
]);
const message = ref("");
const error = ref("");
const saving = ref(false);
const recordQuery = ref("");
const records = ref<Array<AutoCodeRegistryRecord & { payload: FormDraftPayload }>>([]);

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!formName.value.trim()) issues.push("表单名称不能为空。");
  if (!submitLabel.value.trim()) issues.push("提交按钮文案不能为空。");
  if (!fields.length) issues.push("至少需要一个字段。");
  if (fields.some((item) => !item.label.trim() || !item.name.trim())) {
    issues.push("字段标题和字段名必须完整填写。");
  }
  return issues;
});

const preview = computed(() => `<template>
  <el-form :model="formState">
${fields
  .map(
    (field) => `    <el-form-item label="${field.label}" prop="${field.name}">
      <el-${field.type} v-model="formState.${field.name}" placeholder="${field.placeholder}" />
    </el-form-item>`,
  )
  .join("\n")}
    <el-form-item>
      <el-button type="primary">${submitLabel.value || "保存"}</el-button>
    </el-form-item>
  </el-form>
</template>`);

const componentSummary = computed(() => {
  const counter = new Map<string, number>();
  for (const item of fields) {
    counter.set(item.type, (counter.get(item.type) ?? 0) + 1);
  }
  return [...counter.entries()].map(([name, count]) => `${name} × ${count}`).join(" / ") || "-";
});

const designAdvice = computed(() => {
  if (fields.some((item) => item.type === "select")) return "包含选择器，建议后续补充选项来源。";
  if (fields.some((item) => item.type === "date-picker")) return "包含日期组件，建议后续补充格式与时区策略。";
  return "当前结构更适合基础增删改查表单。";
});

const filteredRecords = computed(() => {
  const keyword = recordQuery.value.trim().toLowerCase();
  return records.value.filter((item) => !keyword || item.payload.formName.toLowerCase().includes(keyword));
});

const lastSavedLabel = computed(() => {
  const latest = records.value[0];
  return latest ? formatTime(latest.createdAt) : "未保存";
});

async function loadRecords() {
  const result = await getAutoCodeRegistryApi();
  records.value = result.List.filter(
    (item): item is AutoCodeRegistryRecord & { payload: FormDraftPayload } => item.payload?.kind === "form-designer",
  );
}

async function saveDraft() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "表单草稿不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  await saveAutoCodeRegistryApi({
    kind: "form-designer",
    formName: formName.value.trim() || "untitled-form",
    submitLabel: submitLabel.value.trim() || "保存",
    fields: fields.map((item) => ({ ...item })),
  });
  message.value = `表单 ${formName.value.trim() || "untitled-form"} 已保存`;
  await loadRecords();
  saving.value = false;
}

function hydrate(payload: FormDraftPayload) {
  formName.value = payload.formName;
  submitLabel.value = payload.submitLabel;
  fields.splice(0, fields.length, ...payload.fields.map((item) => ({ ...item })));
  message.value = `已回填草稿 ${payload.formName}`;
}

function addField() {
  fields.push({
    label: "新字段",
    name: `field${fields.length + 1}`,
    type: "input",
    placeholder: "",
  });
}

function appendCommonFields() {
  const common: FormField[] = [
    { label: "创建时间", name: "createdAt", type: "date-picker", placeholder: "" },
    { label: "备注", name: "remark", type: "input", placeholder: "请输入备注" },
  ];
  for (const item of common) {
    if (!fields.some((field) => field.name === item.name)) {
      fields.push({ ...item });
    }
  }
  message.value = "已补充常用字段。";
}

function removeField(index: number) {
  if (fields.length === 1) {
    error.value = "至少保留一个字段。";
    return;
  }
  fields.splice(index, 1);
}

async function copyCode() {
  try {
    await navigator.clipboard.writeText(preview.value);
    message.value = "模板已复制到剪贴板";
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", preview.value);
    message.value = "已切换为手动复制模板";
  }
}

async function copyRecord(payload: FormDraftPayload) {
  const summary = `${payload.formName} / ${payload.submitLabel} / ${payload.fields.length} fields`;
  try {
    await navigator.clipboard.writeText(summary);
    message.value = `已复制 ${payload.formName} 摘要`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", summary);
    message.value = `已切换为手动复制 ${payload.formName} 摘要`;
  }
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(() => {
  void loadRecords();
});
</script>
