<template>
  <div class="page-stack">
    <div class="summary-grid">
      <div class="summary-card">
        <span class="summary-label">交付包总数</span>
        <strong>{{ packageStats.total }}</strong>
        <small>已登记的交付包定义</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">模板分类数</span>
        <strong>{{ packageStats.kindCount }}</strong>
        <small>package / plugin / template 分类覆盖</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">待补清单包</span>
        <strong>{{ packageStats.incompleteCount }}</strong>
        <small>清单项少于 3 条，建议继续拆解</small>
      </div>
      <div class="summary-card">
        <span class="summary-label">当前预览包</span>
        <strong>{{ selectedPackage?.name || draft.name }}</strong>
        <small>{{ selectedPackage ? "列表中选中的交付包" : "表单草稿预览中" }}</small>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <div class="section-head">
          <div>
            <h3 class="title">代码包管理</h3>
            <p class="subtitle">维护交付包定义、输出目录、交付清单与后续动作建议。</p>
          </div>
          <span class="status-chip">{{ activeStatus }}</span>
        </div>

        <div class="toolbar-grid">
          <div class="field">
            <label>包名</label>
            <input v-model="draft.name" @keydown.enter="savePackage" />
          </div>
          <div class="field">
            <label>模板类型</label>
            <select v-model="draft.kind">
              <option value="package">package</option>
              <option value="plugin">plugin</option>
              <option value="template">template</option>
            </select>
          </div>
          <div class="field">
            <label>输出目录</label>
            <input v-model="draft.output" placeholder="/modules/feature" @keydown.enter="savePackage" />
          </div>
        </div>

        <div class="field">
          <label>内容清单</label>
          <textarea
            v-model="draft.summary"
            rows="6"
            placeholder="列出需要包含的菜单、接口、表单、脚本、测试、运行说明"
          />
        </div>

        <div class="analysis-grid">
          <div class="sub-card">
            <h4>交付清单拆解</h4>
            <ul class="checklist">
              <li v-for="entry in draftChecklist" :key="entry.raw">
                <span class="check-kind">{{ entry.category }}</span>
                <span>{{ entry.raw }}</span>
              </li>
              <li v-if="draftChecklist.length === 0" class="empty-line">请输入交付清单后自动拆解。</li>
            </ul>
          </div>

          <div class="sub-card">
            <h4>推荐后续动作</h4>
            <ul class="action-list">
              <li v-for="item in actionRecommendations" :key="item">{{ item }}</li>
            </ul>
          </div>
        </div>

        <div class="sub-card fixed-roadmap">
          <h4>固定后续项</h4>
          <div class="tag-list">
            <span class="tag">Web 全量一致验收关闭后执行</span>
            <span class="tag">Rust + Tauri 2 桌面端重构</span>
            <span class="tag">复用当前 Rust 业务核心</span>
            <span class="tag">不得回退旧 Gin / Web 架构</span>
          </div>
        </div>

        <div class="row button-row">
          <button class="btn primary" :disabled="saving" @click="savePackage">
            {{ saving ? "保存中..." : "保存包定义" }}
          </button>
          <button class="btn" type="button" @click="useTemplate(selectedPackage)">载入当前详情</button>
          <button class="btn" type="button" @click="resetDraft">恢复默认草稿</button>
        </div>
      </div>

      <div class="card">
        <div class="section-head">
          <div>
            <h3 class="title">交付包工作台</h3>
            <p class="subtitle">按关键字与分类筛选，查看交付摘要与详情预览。</p>
          </div>
          <button class="btn" type="button" :disabled="loading" @click="load">
            {{ loading ? "刷新中..." : "刷新列表" }}
          </button>
        </div>

        <div class="toolbar-grid filter-grid">
          <div class="field">
            <label>关键字</label>
            <input
              v-model="filters.keyword"
              placeholder="包名 / 输出目录 / 清单"
              @keydown.enter="load"
            />
          </div>
          <div class="field">
            <label>分类筛选</label>
            <select v-model="filters.kind">
              <option value="all">全部</option>
              <option value="package">package</option>
              <option value="plugin">plugin</option>
              <option value="template">template</option>
            </select>
          </div>
        </div>

        <div class="tag-list quick-filter">
          <button
            v-for="item in kindSummary"
            :key="item.kind"
            class="tag quick-tag"
            type="button"
            @click="filters.kind = item.kind"
          >
            {{ item.kind }} · {{ item.count }}
          </button>
          <button class="tag quick-tag" type="button" @click="filters.kind = 'all'">重置分类</button>
        </div>

        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>包名</th>
                <th>类型</th>
                <th>输出目录</th>
                <th>清单项</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in filteredPackages"
                :key="item.ID"
                :class="{ selected: selectedPackage?.ID === item.ID }"
              >
                <td>
                  <div class="name-cell">
                    <strong>{{ item.name }}</strong>
                    <small>{{ packageHealth(item) }}</small>
                  </div>
                </td>
                <td>{{ item.kind }}</td>
                <td>{{ item.output }}</td>
                <td>{{ parseChecklist(item.summary).length }}</td>
                <td>
                  <div class="row table-actions">
                    <button class="btn small" type="button" @click="selectPackage(item)">详情</button>
                    <button class="btn small" type="button" @click="useTemplate(item)">载入</button>
                  </div>
                </td>
              </tr>
              <tr v-if="filteredPackages.length === 0">
                <td colspan="5" class="empty-cell">暂无匹配包定义，请调整筛选条件或新增交付包。</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="analysis-grid">
          <div class="sub-card">
            <h4>详情预览</h4>
            <div v-if="detailTarget" class="detail-grid">
              <div>
                <span class="detail-label">包名</span>
                <strong>{{ detailTarget.name }}</strong>
              </div>
              <div>
                <span class="detail-label">类型</span>
                <strong>{{ detailTarget.kind }}</strong>
              </div>
              <div>
                <span class="detail-label">输出目录</span>
                <strong>{{ detailTarget.output }}</strong>
              </div>
              <div>
                <span class="detail-label">交付评级</span>
                <strong>{{ packageHealth(detailTarget) }}</strong>
              </div>
            </div>
            <p v-else class="empty-line">请选择交付包查看详情。</p>

            <ul v-if="detailTarget" class="checklist">
              <li v-for="entry in parseChecklist(detailTarget.summary)" :key="`${detailTarget.ID}-${entry.raw}`">
                <span class="check-kind">{{ entry.category }}</span>
                <span>{{ entry.raw }}</span>
              </li>
            </ul>
          </div>

          <div class="sub-card">
            <h4>预览摘要</h4>
            <pre class="code-block">{{ manifestPreview }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getPackageListApi, savePackageApi } from "../../api/admin";
import type { PackageRecord } from "../../types";

type ChecklistEntry = {
  category: string;
  raw: string;
};

const packages = ref<PackageRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const statusText = ref("等待读取交付包定义。");
const selectedPackageId = ref<number | null>(null);

const defaultDraft = {
  name: "feature-bundle",
  kind: "package",
  output: "/modules/feature-bundle",
  summary:
    "用户列表、详情表单、角色切换、导出模板、测试脚本、运行说明，以及开发完成后的 Tauri 2 桌面端重构。",
};

const draft = reactive({ ...defaultDraft });
const filters = reactive({
  keyword: "",
  kind: "all",
});

const selectedPackage = computed(
  () => packages.value.find((item) => item.ID === selectedPackageId.value) || null,
);

const detailTarget = computed(() => selectedPackage.value);

const packageStats = computed(() => {
  const kinds = new Set(packages.value.map((item) => item.kind));
  return {
    total: packages.value.length,
    kindCount: kinds.size,
    incompleteCount: packages.value.filter((item) => parseChecklist(item.summary).length < 3).length,
  };
});

const kindSummary = computed(() => {
  const counter = new Map<string, number>();
  for (const item of packages.value) {
    counter.set(item.kind, (counter.get(item.kind) || 0) + 1);
  }
  return Array.from(counter.entries())
    .map(([kind, count]) => ({ kind, count }))
    .sort((left, right) => right.count - left.count || left.kind.localeCompare(right.kind));
});

const filteredPackages = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();
  return packages.value.filter((item) => {
    const hitKind = filters.kind === "all" || item.kind === filters.kind;
    const hitKeyword =
      keyword.length === 0 ||
      [item.name, item.kind, item.output, item.summary].some((part) =>
        part.toLowerCase().includes(keyword),
      );
    return hitKind && hitKeyword;
  });
});

const draftChecklist = computed(() => parseChecklist(draft.summary));

const actionRecommendations = computed(() => {
  const items = [
    `确认 ${draft.kind} 类型对应的输出目录 ${draft.output} 已纳入交付规范。`,
  ];
  if (!draft.output.includes("/")) {
    items.push("建议补充分层输出目录，避免交付包落在根路径。");
  }
  if (draftChecklist.value.length < 3) {
    items.push("当前清单项偏少，建议至少拆出页面、接口、配置、测试或文档。");
  }
  if (!draftChecklist.value.some((item) => item.category === "测试")) {
    items.push("建议显式加入测试脚本或验收项，便于交付包闭环。");
  }
  if (!draftChecklist.value.some((item) => item.category === "文档")) {
    items.push("建议补充运行说明、部署说明或对接说明。");
  }
  if (draft.kind === "plugin") {
    items.push("插件类交付包建议补充菜单挂载、权限接入与安装回滚说明。");
  }
  if (draft.kind === "template") {
    items.push("模板类交付包建议附带可复制字段与示例配置，便于二次生成。");
  }
  return items;
});

const activeStatus = computed(() => {
  if (saving.value) return "正在保存交付包";
  if (loading.value) return "正在同步包定义";
  return statusText.value;
});

const manifestPreview = computed(() =>
  JSON.stringify(
    {
      packageName: draft.name,
      packageType: draft.kind,
      outputPath: draft.output,
      checklist: draftChecklist.value.map((item, index) => ({
        order: index + 1,
        category: item.category,
        content: item.raw,
      })),
      recommendations: actionRecommendations.value,
    },
    null,
    2,
  ),
);

function parseChecklist(summary: string): ChecklistEntry[] {
  return summary
    .split(/[、,，\n]/)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => ({
      raw: entry,
      category: classifyEntry(entry),
    }));
}

function classifyEntry(entry: string) {
  if (/(接口|api|路由)/i.test(entry)) return "接口";
  if (/(表单|页面|列表|详情|菜单|视图)/i.test(entry)) return "页面";
  if (/(测试|验收|回归)/i.test(entry)) return "测试";
  if (/(文档|说明|readme|手册)/i.test(entry)) return "文档";
  if (/(脚本|任务|流水线|部署)/i.test(entry)) return "脚本";
  if (/(配置|参数|字典|权限)/i.test(entry)) return "配置";
  return "模块";
}

function packageHealth(item: PackageRecord) {
  const count = parseChecklist(item.summary).length;
  if (count >= 6) return "完整度高";
  if (count >= 3) return "可交付";
  return "待补清单";
}

function applyDraft(payload: Pick<PackageRecord, "kind" | "name" | "output" | "summary">) {
  draft.name = payload.name;
  draft.kind = payload.kind;
  draft.output = payload.output;
  draft.summary = payload.summary;
}

function selectPackage(item: PackageRecord) {
  selectedPackageId.value = item.ID;
  statusText.value = `正在查看 ${item.name} 的交付详情。`;
}

function useTemplate(item: PackageRecord | null) {
  if (!item) {
    statusText.value = "当前没有可载入的交付包详情。";
    return;
  }
  applyDraft(item);
  selectedPackageId.value = item.ID;
  statusText.value = `已将 ${item.name} 载入为当前编辑草稿。`;
}

function resetDraft() {
  applyDraft(defaultDraft);
  statusText.value = "已恢复默认交付包草稿。";
}

async function load() {
  loading.value = true;
  try {
    const result = await getPackageListApi();
    packages.value = result.List;
    if (!selectedPackageId.value && result.List.length > 0) {
      selectedPackageId.value = result.List[0]?.ID ?? null;
    } else if (
      selectedPackageId.value &&
      !result.List.some((item) => item.ID === selectedPackageId.value)
    ) {
      selectedPackageId.value = result.List[0]?.ID ?? null;
    }
    statusText.value = `已同步 ${result.List.length} 个交付包定义。`;
  } finally {
    loading.value = false;
  }
}

async function savePackage() {
  saving.value = true;
  try {
    await savePackageApi({
      name: draft.name.trim(),
      kind: draft.kind.trim(),
      output: draft.output.trim(),
      summary: draft.summary.trim(),
    });
    await load();
    const matched = packages.value.find(
      (item) => item.name === draft.name.trim() && item.output === draft.output.trim(),
    );
    if (matched) {
      selectedPackageId.value = matched.ID;
    }
    statusText.value = `已保存交付包 ${draft.name.trim()}。`;
  } finally {
    saving.value = false;
  }
}

onMounted(load);
</script>

<style scoped>
.page-stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.summary-grid,
.analysis-grid {
  display: grid;
  gap: 16px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.summary-card,
.sub-card {
  border: 1px solid #e5e7eb;
  border-radius: 14px;
  background: #fff;
  padding: 16px;
}

.summary-card strong {
  display: block;
  margin: 8px 0 4px;
  font-size: 28px;
  color: #111827;
}

.summary-label,
.detail-label {
  display: block;
  color: #6b7280;
  font-size: 12px;
}

.split-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: minmax(0, 1.05fr) minmax(0, 1fr);
}

.card {
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  background: #fff;
  padding: 20px;
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.title {
  margin: 0;
  font-size: 20px;
}

.subtitle {
  margin: 6px 0 0;
  color: #6b7280;
}

.status-chip {
  border-radius: 999px;
  background: #eef2ff;
  color: #4338ca;
  padding: 6px 12px;
  font-size: 12px;
  white-space: nowrap;
}

.toolbar-grid,
.detail-grid {
  display: grid;
  gap: 12px;
}

.toolbar-grid {
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  margin-bottom: 16px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  font-size: 13px;
  color: #374151;
}

.field input,
.field select,
.field textarea {
  width: 100%;
  border: 1px solid #d1d5db;
  border-radius: 10px;
  padding: 10px 12px;
  font: inherit;
}

.analysis-grid {
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  margin-top: 16px;
}

.sub-card h4 {
  margin: 0 0 12px;
  font-size: 16px;
}

.checklist,
.action-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checklist li,
.action-list li {
  color: #1f2937;
}

.check-kind {
  display: inline-block;
  min-width: 42px;
  margin-right: 8px;
  color: #4f46e5;
  font-size: 12px;
}

.fixed-roadmap {
  margin-top: 16px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag {
  border: 0;
  border-radius: 999px;
  background: #f3f4f6;
  color: #374151;
  padding: 6px 10px;
  font-size: 12px;
}

.quick-filter {
  margin-bottom: 16px;
}

.quick-tag {
  cursor: pointer;
}

.row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.button-row {
  margin-top: 16px;
}

.btn {
  border: 1px solid #d1d5db;
  border-radius: 10px;
  background: #fff;
  padding: 9px 14px;
  cursor: pointer;
}

.btn.primary {
  border-color: #4f46e5;
  background: #4f46e5;
  color: #fff;
}

.btn.small {
  padding: 6px 10px;
  font-size: 12px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.data-table {
  overflow: auto;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  padding: 12px;
  border-bottom: 1px solid #e5e7eb;
  text-align: left;
  vertical-align: top;
}

tbody tr.selected {
  background: #eef2ff;
}

.name-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.name-cell small {
  color: #6b7280;
}

.table-actions {
  gap: 8px;
}

.code-block {
  margin: 0;
  padding: 14px;
  border-radius: 12px;
  background: #111827;
  color: #e5e7eb;
  overflow: auto;
}

.empty-line,
.empty-cell {
  color: #6b7280;
}

@media (max-width: 1100px) {
  .split-grid {
    grid-template-columns: 1fr;
  }
}
</style>
