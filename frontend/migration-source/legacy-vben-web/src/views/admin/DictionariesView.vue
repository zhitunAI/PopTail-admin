<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">字典工作台</h3>
          <p class="subtitle">
            围绕字典主表的检索、状态巡检、摘要联动与新增维护，集中处理字典资产。
          </p>
        </div>
        <div class="row">
          <button class="btn ghost" :disabled="loading" @click="load">刷新</button>
          <button class="btn ghost" :disabled="!activeDictionary" @click="copyDictionarySummary">
            复制当前摘要
          </button>
        </div>
      </div>
      <p v-if="statusMessage" class="subtitle">{{ statusMessage }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>字典总数</h4>
        <p class="stat-value">{{ filteredRows.length }}</p>
        <p class="subtitle">筛选结果 / 全量 {{ sourceRows.length }}</p>
      </div>
      <div class="stat-card">
        <h4>启用字典</h4>
        <p class="stat-value">{{ enabledCount }}</p>
        <p class="subtitle">停用 {{ sourceRows.length - enabledCount }} 条</p>
      </div>
      <div class="stat-card">
        <h4>类型分组</h4>
        <p class="stat-value">{{ typeBuckets.length }}</p>
        <p class="subtitle">{{ typeBuckets[0]?.label || "暂无类型分组" }}</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeDictionary?.type || "-" }}</p>
        <p class="subtitle">{{ activeDictionary?.name || "未选择字典" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">筛选与状态</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>名称 / 说明</label>
            <input
              v-model.trim="filters.keyword"
              placeholder="输入名称、说明关键字"
              @keyup.enter="applyFilters"
            />
          </div>
          <div class="field">
            <label>类型</label>
            <input
              v-model.trim="filters.type"
              placeholder="输入类型标识"
              @keyup.enter="applyFilters"
            />
          </div>
          <div class="field">
            <label>状态</label>
            <select v-model="filters.status">
              <option value="">全部状态</option>
              <option value="enabled">启用</option>
              <option value="disabled">停用</option>
            </select>
          </div>
        </div>
        <div class="row">
          <button class="btn primary" :disabled="loading" @click="applyFilters">查询</button>
          <button class="btn ghost" :disabled="loading" @click="reset">重置</button>
          <button class="btn ghost" :disabled="loading" @click="openCreate">新增字典</button>
        </div>
        <div class="tag-list">
          <button
            v-for="bucket in typeBuckets"
            :key="bucket.key"
            class="tag action-tag"
            :class="{ active: filters.type === bucket.sample }"
            @click="pickTypeBucket(bucket.sample)"
          >
            {{ bucket.label }} · {{ bucket.count }}
          </button>
        </div>
        <p class="subtitle">
          {{
            filters.status
              ? `当前仅查看${filters.status === "enabled" ? "启用" : "停用"}字典。`
              : "可按名称、类型和状态组合筛选，快速定位需维护字典。"
          }}
        </p>
      </div>

      <div class="card">
        <h3 class="title">当前建议</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>工作台建议</td><td>{{ currentAdvice }}</td></tr>
              <tr><td>最近操作</td><td>{{ recentActions[0] || "暂无操作记录" }}</td></tr>
              <tr><td>状态提示</td><td>{{ statusHint }}</td></tr>
              <tr><td>待关注类型</td><td>{{ attentionBucket }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">字典台账</h3>
          <p class="subtitle">选中条目后可直接查看摘要、编辑主表信息或跳转到详情树继续维护。</p>
        </div>
        <div class="row">
          <button
            v-if="filteredRows.length"
            class="btn ghost"
            @click="inspect(filteredRows[0])"
          >
            查看首条命中
          </button>
        </div>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>名称</th>
              <th>类型</th>
              <th>状态</th>
              <th>分组</th>
              <th>说明</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in filteredRows" :key="item.ID">
              <td>{{ item.ID }}</td>
              <td>{{ item.name }}</td>
              <td>{{ item.type }}</td>
              <td>{{ item.status ? "启用" : "停用" }}</td>
              <td>{{ resolveTypeBucket(item.type).label }}</td>
              <td>{{ item.desc || "暂无说明" }}</td>
              <td>
                <div class="row">
                  <button class="btn ghost" @click="inspect(item)">概览</button>
                  <button class="btn ghost" @click="openDetail(item.ID)">详情</button>
                </div>
              </td>
            </tr>
            <tr v-if="!filteredRows.length">
              <td colspan="7">暂无符合条件的字典，请调整筛选条件后重试。</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">字典概览</h3>
        <div v-if="activeDictionary" class="data-table">
          <table>
            <tbody>
              <tr><td>字典名称</td><td>{{ activeDictionary.name }}</td></tr>
              <tr><td>类型标识</td><td>{{ activeDictionary.type }}</td></tr>
              <tr><td>状态</td><td>{{ activeDictionary.status ? "启用" : "停用" }}</td></tr>
              <tr><td>类型分组</td><td>{{ resolveTypeBucket(activeDictionary.type).label }}</td></tr>
              <tr><td>运营摘要</td><td>{{ activeSummary }}</td></tr>
              <tr><td>维护建议</td><td>{{ activeRecommendation }}</td></tr>
              <tr><td>说明</td><td>{{ activeDictionary.desc || "暂无说明" }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">请选择左侧字典，查看摘要、分组和后续维护建议。</p>
      </div>

      <div class="card">
        <h3 class="title">{{ form.ID ? "编辑字典" : "新增字典" }}</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>名称</label>
            <input v-model.trim="form.name" placeholder="输入字典名称" />
          </div>
          <div class="field">
            <label>类型</label>
            <input v-model.trim="form.type" placeholder="例如 sys_region" />
          </div>
          <div class="field">
            <label>状态</label>
            <select v-model="form.status">
              <option :value="true">启用</option>
              <option :value="false">停用</option>
            </select>
          </div>
        </div>
        <div class="field">
          <label>说明</label>
          <textarea v-model.trim="form.desc" rows="4" placeholder="填写字典用途、适用范围或维护备注" />
        </div>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>表单校验</td><td>{{ formValidation }}</td></tr>
              <tr><td>命名建议</td><td>{{ formSuggestion }}</td></tr>
              <tr><td>最近操作提示</td><td>{{ recentActions[0] || "暂无操作记录" }}</td></tr>
            </tbody>
          </table>
        </div>
        <div class="row">
          <button class="btn primary" :disabled="loading || !formValid" @click="save">保存字典</button>
          <button class="btn ghost" @click="resetForm">重置表单</button>
          <button
            class="btn ghost"
            :disabled="!activeDictionary || !activeDictionary.ID"
            @click="openDetail(activeDictionary!.ID)"
          >
            去维护详情
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { getDictionaryListApi, saveDictionaryApi } from "../../api/admin";
import type { DictionaryInfo } from "../../types";

type StatusFilter = "" | "enabled" | "disabled";
type TypeBucket = {
  key: string;
  label: string;
  count: number;
  sample: string;
};

const sourceRows = ref<DictionaryInfo[]>([]);
const filteredRows = ref<DictionaryInfo[]>([]);
const activeDictionary = ref<DictionaryInfo | null>(null);
const loading = ref(false);
const statusMessage = ref("");
const recentActions = ref<string[]>([]);
const router = useRouter();

const filters = reactive<{
  keyword: string;
  type: string;
  status: StatusFilter;
}>({
  keyword: "",
  type: "",
  status: "",
});

const form = reactive({
  ID: undefined as number | undefined,
  name: "",
  type: "",
  status: true,
  desc: "",
});

const enabledCount = computed(() => sourceRows.value.filter((item) => item.status).length);
const typeBuckets = computed<TypeBucket[]>(() => {
  const store = new Map<string, TypeBucket>();
  sourceRows.value.forEach((item) => {
    const bucketKey = getTypeBucketKey(item.type);
    const current = store.get(bucketKey);
    if (current) {
      current.count += 1;
    } else {
      store.set(bucketKey, {
        key: bucketKey,
        label: `类型组 ${bucketKey}`,
        count: 1,
        sample: item.type,
      });
    }
  });
  return Array.from(store.values()).sort((a, b) => b.count - a.count || a.label.localeCompare(b.label));
});
const currentAdvice = computed(() => {
  if (!sourceRows.value.length) {
    return "当前尚无字典，建议先新增基础字典主表。";
  }
  if (!enabledCount.value) {
    return "当前所有字典均处于停用态，建议优先恢复核心字典。";
  }
  if (filteredRows.value.length !== sourceRows.value.length) {
    return `当前已聚焦 ${filteredRows.value.length} 条字典，可继续进入详情树维护具体节点。`;
  }
  return "优先检查说明为空或停用的字典，补齐维护意图后再进入详情层。";
});
const statusHint = computed(() => {
  if (filters.status === "enabled") {
    return "仅查看启用字典，适合巡检线上仍生效的资产。";
  }
  if (filters.status === "disabled") {
    return "当前聚焦停用字典，建议确认是否还能被引用。";
  }
  return "未限制状态，可结合名称或类型继续收窄范围。";
});
const attentionBucket = computed(() => {
  const disabled = sourceRows.value.filter((item) => !item.status);
  if (!disabled.length) {
    return "暂无明显停用风险分组";
  }
  const first = disabled[0];
  return `${resolveTypeBucket(first.type).label} 中存在停用字典 ${first.name}`;
});
const activeSummary = computed(() => {
  if (!activeDictionary.value) {
    return "未选择字典";
  }
  return `${activeDictionary.value.name}（${activeDictionary.value.type}）当前处于${
    activeDictionary.value.status ? "启用" : "停用"
  }状态，归属 ${resolveTypeBucket(activeDictionary.value.type).label}。`;
});
const activeRecommendation = computed(() => {
  if (!activeDictionary.value) {
    return "请选择字典查看建议";
  }
  if (!activeDictionary.value.desc) {
    return "该字典说明为空，建议补充用途、前端展示语义或维护边界。";
  }
  if (!activeDictionary.value.status) {
    return "该字典当前停用，建议进入详情页确认节点是否也应同步冻结。";
  }
  return "建议进入详情页检查节点层级与扩展值是否仍符合现行流程。";
});
const formValid = computed(() => Boolean(form.name.trim() && form.type.trim()));
const formValidation = computed(() => {
  if (!form.name.trim()) {
    return "请填写字典名称。";
  }
  if (!form.type.trim()) {
    return "请填写类型标识。";
  }
  if (form.type.trim().length < 3) {
    return "类型标识建议至少 3 个字符，便于后续识别。";
  }
  return "表单可提交。";
});
const formSuggestion = computed(() => {
  if (!form.type.trim()) {
    return "类型标识建议使用 domain_scene 形式，便于主表与详情树联动。";
  }
  if (sourceRows.value.some((item) => item.type === form.type.trim() && item.ID !== form.ID)) {
    return "已有同类型标识，请确认是否为同一字典主表。";
  }
  return `当前类型将归入 ${resolveTypeBucket(form.type.trim()).label}。`;
});

async function load() {
  loading.value = true;
  try {
    const result = await getDictionaryListApi();
    sourceRows.value = result.List;
    applyFilters();
    if (activeDictionary.value) {
      const current = sourceRows.value.find((item) => item.ID === activeDictionary.value?.ID) ?? null;
      if (current) {
        inspect(current, false);
      } else {
        activeDictionary.value = null;
        resetForm();
      }
    } else if (filteredRows.value[0]) {
      inspect(filteredRows.value[0], false);
    }
    statusMessage.value = `已加载 ${sourceRows.value.length} 条字典主表记录。`;
  } finally {
    loading.value = false;
  }
}

async function openDetail(id: number) {
  await router.push({ name: "dictionaryDetail", params: { id } });
}

function inspect(item: DictionaryInfo, record = true) {
  activeDictionary.value = item;
  hydrateForm(item);
  if (record) {
    noteAction(`已查看字典 ${item.name}（${item.type}）`);
  }
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  const type = filters.type.trim().toLowerCase();
  filteredRows.value = sourceRows.value.filter((item) => {
    const matchesKeyword =
      !keyword ||
      item.name.toLowerCase().includes(keyword) ||
      item.desc.toLowerCase().includes(keyword);
    const matchesType = !type || item.type.toLowerCase().includes(type);
    const matchesStatus =
      !filters.status ||
      (filters.status === "enabled" ? item.status : !item.status);
    return matchesKeyword && matchesType && matchesStatus;
  });
  statusMessage.value = `当前命中 ${filteredRows.value.length} 条字典。`;
}

function reset() {
  filters.keyword = "";
  filters.type = "";
  filters.status = "";
  filteredRows.value = [...sourceRows.value];
  statusMessage.value = "已重置筛选条件。";
}

function hydrateForm(item: DictionaryInfo) {
  form.ID = item.ID;
  form.name = item.name;
  form.type = item.type;
  form.status = item.status;
  form.desc = item.desc;
}

function resetForm() {
  if (activeDictionary.value) {
    hydrateForm(activeDictionary.value);
    statusMessage.value = "已恢复当前选中字典内容。";
    return;
  }
  form.ID = undefined;
  form.name = "";
  form.type = "";
  form.status = true;
  form.desc = "";
  statusMessage.value = "已清空表单，可直接新增字典。";
}

function openCreate() {
  activeDictionary.value = null;
  form.ID = undefined;
  form.name = "";
  form.type = "";
  form.status = true;
  form.desc = "";
  noteAction("已打开新增字典表单");
  statusMessage.value = "已切换到新增字典模式。";
}

function pickTypeBucket(sample: string) {
  filters.type = sample;
  applyFilters();
}

function resolveTypeBucket(type: string) {
  const key = getTypeBucketKey(type);
  return typeBuckets.value.find((item) => item.key === key) ?? {
    key,
    label: `类型组 ${key}`,
    count: 0,
    sample: type,
  };
}

function getTypeBucketKey(type: string) {
  const normalized = type.trim();
  if (!normalized) {
    return "未命名";
  }
  return normalized.split(/[_-]/)[0] || normalized;
}

function noteAction(message: string) {
  recentActions.value = [`${new Date().toLocaleString()} · ${message}`, ...recentActions.value].slice(0, 6);
}

async function copyDictionarySummary() {
  if (!activeDictionary.value) {
    return;
  }
  const text = [
    `字典：${activeDictionary.value.name}`,
    `类型：${activeDictionary.value.type}`,
    `状态：${activeDictionary.value.status ? "启用" : "停用"}`,
    `分组：${resolveTypeBucket(activeDictionary.value.type).label}`,
    `建议：${activeRecommendation.value}`,
  ].join("\n");
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      window.prompt("当前环境不支持自动复制，请手动复制以下内容：", text);
    }
    noteAction(`已复制字典 ${activeDictionary.value.name} 摘要`);
    statusMessage.value = "已复制当前字典摘要。";
  } catch {
    window.prompt("复制失败，请手动复制以下内容：", text);
  }
}

async function save() {
  if (!formValid.value) {
    statusMessage.value = formValidation.value;
    return;
  }
  loading.value = true;
  try {
    const payload = {
      ID: form.ID,
      name: form.name.trim(),
      type: form.type.trim(),
      status: form.status,
      desc: form.desc.trim(),
    };
    const saved = await saveDictionaryApi(payload);
    noteAction(`已保存字典 ${saved.name}`);
    statusMessage.value = `字典 ${saved.name} 已保存。`;
    await load();
    const current = sourceRows.value.find((item) => item.ID === saved.ID) ?? null;
    if (current) {
      inspect(current, false);
    }
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>
