<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">导出模板</h3>
          <p class="subtitle">围绕模板检索、预览、推荐与分类摘要，集中查看当前可运营的导出模板资产。</p>
        </div>
        <div class="row">
          <button class="btn ghost" :disabled="loading" @click="load">刷新</button>
          <button class="btn ghost" :disabled="!activeRow" @click="copyActiveTemplateId">复制当前模板标识</button>
        </div>
      </div>
      <p v-if="statusMessage" class="subtitle">{{ statusMessage }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>模板总数</h4>
        <p class="stat-value">{{ filteredRows.length }}</p>
        <p class="subtitle">当前筛选结果 / 全量 {{ sourceRows.length }}</p>
      </div>
      <div class="stat-card">
        <h4>分类数量</h4>
        <p class="stat-value">{{ categoryCards.length }}</p>
        <p class="subtitle">按模板标识前缀自动归类</p>
      </div>
      <div class="stat-card">
        <h4>最近推荐</h4>
        <p class="stat-value">{{ recommendedTemplates.length }}</p>
        <p class="subtitle">{{ recommendedTemplates[0]?.name || "暂无推荐模板" }}</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeRow?.template_id || "-" }}</p>
        <p class="subtitle">{{ activeRow?.name || "未选择模板" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">模板筛选</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>模板名 / 说明</label>
            <input
              v-model.trim="filters.keyword"
              placeholder="输入名称、说明或关键字"
              @keyup.enter="applyFilters"
            />
          </div>
          <div class="field">
            <label>模板标识 / 编号</label>
            <input
              v-model.trim="filters.templateId"
              placeholder="输入 template_id 或局部编号"
              @keyup.enter="applyFilters"
            />
          </div>
          <div class="field">
            <label>模板分类</label>
            <select v-model="filters.category">
              <option value="">全部分类</option>
              <option v-for="item in categoryCards" :key="item.key" :value="item.key">
                {{ item.label }}（{{ item.count }}）
              </option>
            </select>
          </div>
        </div>
        <div class="row">
          <button class="btn primary" :disabled="loading" @click="applyFilters">查询</button>
          <button class="btn ghost" :disabled="loading" @click="reset">重置</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">分类摘要</h3>
        <div class="tag-list">
          <button
            v-for="item in categoryCards"
            :key="item.key"
            class="tag action-tag"
            :class="{ active: filters.category === item.key }"
            @click="pickCategory(item.key)"
          >
            {{ item.label }} · {{ item.count }}
          </button>
        </div>
        <p class="subtitle">
          {{
            filters.category
              ? `已锁定分类：${activeCategoryLabel}，可继续结合编号和名称精筛。`
              : "未选择分类时展示全部模板，可点击分类标签快速筛选。"
          }}
        </p>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">模板列表</h3>
          <p class="subtitle">支持按名称、编号、分类联合过滤，点击任意模板查看详情与推荐关系。</p>
        </div>
        <div class="row">
          <button
            v-if="recommendedTemplates.length"
            class="btn ghost"
            @click="inspect(recommendedTemplates[0])"
          >
            查看首个推荐
          </button>
        </div>
      </div>

      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>分类</th>
              <th>模板名称</th>
              <th>模板标识</th>
              <th>说明</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in filteredRows" :key="item.ID">
              <td>{{ item.ID }}</td>
              <td>{{ resolveCategory(item).label }}</td>
              <td>{{ item.name }}</td>
              <td>{{ item.template_id }}</td>
              <td>{{ item.desc }}</td>
              <td>
                <div class="row">
                  <button class="btn ghost" @click="inspect(item)">详情</button>
                  <button class="btn ghost" @click="copyTemplateId(item)">复制标识</button>
                </div>
              </td>
            </tr>
            <tr v-if="!filteredRows.length">
              <td colspan="6">暂无符合条件的模板，请调整筛选条件后重试。</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">模板详情</h3>
        <div v-if="activeRow" class="data-table">
          <table>
            <tbody>
              <tr><td>模板名称</td><td>{{ activeRow.name }}</td></tr>
              <tr><td>模板标识</td><td>{{ activeRow.template_id }}</td></tr>
              <tr><td>模板分类</td><td>{{ resolveCategory(activeRow).label }}</td></tr>
              <tr><td>序号</td><td>{{ activeRow.ID }}</td></tr>
              <tr><td>运营摘要</td><td>{{ activeSummary }}</td></tr>
              <tr><td>说明</td><td>{{ activeRow.desc || "暂无补充说明" }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">请选择左侧模板，查看分类、说明与推荐摘要。</p>
      </div>

      <div class="card">
        <h3 class="title">最近模板推荐</h3>
        <p class="subtitle">按模板 ID 倒序推荐最近维护的模板，便于运营侧快速复用。</p>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>模板</th>
                <th>标识</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in recommendedTemplates" :key="item.ID">
                <td>{{ item.ID }}</td>
                <td>{{ item.name }}</td>
                <td>{{ item.template_id }}</td>
                <td>
                  <button class="btn ghost" @click="inspect(item)">查看</button>
                </td>
              </tr>
              <tr v-if="!recommendedTemplates.length">
                <td colspan="4">暂无推荐模板</td>
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
import { getExportTemplatesApi } from "../../api/admin";
import type { ExportTemplateInfo } from "../../types";

type TemplateCategory = {
  key: string;
  label: string;
  count: number;
};

const sourceRows = ref<ExportTemplateInfo[]>([]);
const filteredRows = ref<ExportTemplateInfo[]>([]);
const activeRow = ref<ExportTemplateInfo | null>(null);
const loading = ref(false);
const statusMessage = ref("");
const filters = reactive({
  keyword: "",
  templateId: "",
  category: "",
});

async function load() {
  loading.value = true;
  try {
    const result = await getExportTemplatesApi();
    const sortedRows = [...result.List].sort((left, right) => right.ID - left.ID);
    sourceRows.value = sortedRows;
    filteredRows.value = sortedRows;
    if (!activeRow.value && sortedRows.length) {
      activeRow.value = sortedRows[0];
    } else if (activeRow.value) {
      activeRow.value = sortedRows.find((item) => item.ID === activeRow.value?.ID) ?? sortedRows[0] ?? null;
    }
    if (filters.keyword || filters.templateId || filters.category) {
      applyFilters();
      return;
    }
    statusMessage.value = `已加载 ${sortedRows.length} 条模板记录`;
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : "导出模板加载失败";
  } finally {
    loading.value = false;
  }
}

function resolveCategory(template: ExportTemplateInfo) {
  const normalized = template.template_id.trim();
  if (!normalized) {
    return { key: "unknown", label: "未分类" };
  }
  const prefix = normalized.includes("_")
    ? normalized.split("_")[0]
    : normalized.includes("-")
      ? normalized.split("-")[0]
      : normalized.replace(/[0-9]+$/g, "");
  const key = (prefix || normalized).toLowerCase();
  return {
    key,
    label: prefix ? prefix.toUpperCase() : "未分类",
  };
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  const templateId = filters.templateId.trim().toLowerCase();
  filteredRows.value = sourceRows.value.filter((item) => {
    const category = resolveCategory(item);
    const matchesKeyword =
      !keyword ||
      item.name.toLowerCase().includes(keyword) ||
      item.desc.toLowerCase().includes(keyword) ||
      item.template_id.toLowerCase().includes(keyword);
    const matchesTemplateId = !templateId || item.template_id.toLowerCase().includes(templateId);
    const matchesCategory = !filters.category || category.key === filters.category;
    return matchesKeyword && matchesTemplateId && matchesCategory;
  });
  if (filteredRows.value.length && !filteredRows.value.some((item) => item.ID === activeRow.value?.ID)) {
    activeRow.value = filteredRows.value[0];
  }
  statusMessage.value = filteredRows.value.length
    ? `筛选得到 ${filteredRows.value.length} 条模板`
    : "未匹配到模板，请调整筛选条件";
}

function reset() {
  filters.keyword = "";
  filters.templateId = "";
  filters.category = "";
  filteredRows.value = sourceRows.value;
  activeRow.value = filteredRows.value[0] ?? null;
  statusMessage.value = "已恢复默认模板视图";
}

function inspect(item: ExportTemplateInfo) {
  activeRow.value = item;
  statusMessage.value = `已查看模板：${item.name}`;
}

function pickCategory(categoryKey: string) {
  filters.category = filters.category === categoryKey ? "" : categoryKey;
  applyFilters();
}

async function copyTemplateId(item: ExportTemplateInfo) {
  try {
    await navigator.clipboard.writeText(item.template_id);
    statusMessage.value = `已复制模板标识：${item.template_id}`;
  } catch (error) {
    window.prompt("当前环境无法直接写入剪贴板，请手动复制模板标识：", item.template_id);
    statusMessage.value = error instanceof Error ? error.message : "剪贴板写入失败，已切换为手动复制";
  }
}

async function copyActiveTemplateId() {
  if (!activeRow.value) {
    return;
  }
  await copyTemplateId(activeRow.value);
}

const categoryCards = computed<TemplateCategory[]>(() => {
  const categoryMap = new Map<string, TemplateCategory>();
  for (const item of sourceRows.value) {
    const category = resolveCategory(item);
    const current = categoryMap.get(category.key);
    if (current) {
      current.count += 1;
    } else {
      categoryMap.set(category.key, { ...category, count: 1 });
    }
  }
  return Array.from(categoryMap.values()).sort((left, right) => right.count - left.count);
});

const recommendedTemplates = computed(() => {
  const candidates = filters.category
    ? sourceRows.value.filter((item) => resolveCategory(item).key === filters.category)
    : sourceRows.value;
  return candidates.slice(0, 5);
});

const activeCategoryLabel = computed(() => {
  if (!filters.category) {
    return "全部分类";
  }
  return categoryCards.value.find((item) => item.key === filters.category)?.label || filters.category;
});

const activeSummary = computed(() => {
  if (!activeRow.value) {
    return "-";
  }
  const activeCategoryKey = resolveCategory(activeRow.value).key;
  const relatedCount = sourceRows.value.filter((item) => resolveCategory(item).key === activeCategoryKey).length;
  return `同分类模板 ${relatedCount} 个，适合用于 ${resolveCategory(activeRow.value).label} 场景快速导出。`;
});

onMounted(load);
</script>
