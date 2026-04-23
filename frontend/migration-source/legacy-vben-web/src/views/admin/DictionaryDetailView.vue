<template>
  <div class="stack">
    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">字典详情工作台</h3>
          <p class="subtitle">
            围绕层级节点的检索、层级分析、父子关系建议与保存维护，集中处理当前字典详情树。
          </p>
        </div>
        <div class="row">
          <button class="btn ghost" @click="back">返回字典列表</button>
          <button class="btn ghost" :disabled="loading" @click="load">刷新</button>
          <button class="btn ghost" :disabled="!selectedNode" @click="copyNodeSummary">复制节点摘要</button>
        </div>
      </div>
      <p v-if="statusMessage" class="subtitle">{{ statusMessage }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>节点总数</h4>
        <p class="stat-value">{{ filteredRows.length }}</p>
        <p class="subtitle">筛选结果 / 全量 {{ flatRows.length }}</p>
      </div>
      <div class="stat-card">
        <h4>启用节点</h4>
        <p class="stat-value">{{ enabledCount }}</p>
        <p class="subtitle">停用 {{ flatRows.length - enabledCount }} 个</p>
      </div>
      <div class="stat-card">
        <h4>最大层级</h4>
        <p class="stat-value">{{ maxLevel }}</p>
        <p class="subtitle">根节点 {{ rootCount }} 个 / 叶子 {{ leafCount }} 个</p>
      </div>
      <div class="stat-card">
        <h4>当前字典</h4>
        <p class="stat-value">{{ dictionaryId }}</p>
        <p class="subtitle">{{ selectedNode?.label || "未选中节点" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">筛选与层级分析</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>标签 / 字典值 / 扩展值</label>
            <input
              v-model.trim="filters.keyword"
              placeholder="输入节点标签、值或扩展值"
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
          <div class="field">
            <label>层级</label>
            <select v-model="filters.level">
              <option value="">全部层级</option>
              <option v-for="level in levelOptions" :key="level" :value="String(level)">
                第 {{ level }} 层
              </option>
            </select>
          </div>
        </div>
        <div class="row">
          <button class="btn primary" :disabled="loading" @click="applyFilters">查询</button>
          <button class="btn ghost" :disabled="loading" @click="resetFilters">重置</button>
          <button class="btn ghost" @click="openRootDraft">新增根节点</button>
        </div>
        <div class="tag-list">
          <button
            v-for="card in levelCards"
            :key="card.level"
            class="tag action-tag"
            :class="{ active: filters.level === String(card.level) }"
            @click="pickLevel(card.level)"
          >
            第 {{ card.level }} 层 · {{ card.count }}
          </button>
        </div>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>当前建议</td><td>{{ currentAdvice }}</td></tr>
              <tr><td>层级热点</td><td>{{ hotLevelSummary }}</td></tr>
              <tr><td>最近操作</td><td>{{ recentActions[0] || "暂无操作记录" }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">节点概览</h3>
        <div v-if="selectedNodeMeta" class="data-table">
          <table>
            <tbody>
              <tr><td>节点标签</td><td>{{ selectedNodeMeta.label }}</td></tr>
              <tr><td>路径摘要</td><td>{{ selectedNodeMeta.path }}</td></tr>
              <tr><td>子节点数</td><td>{{ selectedNodeMeta.childCount }}</td></tr>
              <tr><td>节点状态</td><td>{{ selectedNodeMeta.status ? "启用" : "停用" }}</td></tr>
              <tr><td>节点建议</td><td>{{ selectedAdvice }}</td></tr>
              <tr><td>扩展值</td><td>{{ selectedNodeMeta.extend || "暂无扩展值" }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">请选择左侧节点，查看路径、层级与维护建议。</p>
      </div>
    </div>

    <div class="card">
      <div class="row space-between wrap">
        <div>
          <h3 class="title">树形节点列表</h3>
          <p class="subtitle">点击节点可查看详情并直接编辑；支持从父节点继续补充子项。</p>
        </div>
        <div class="row">
          <button
            v-if="filteredRows.length"
            class="btn ghost"
            @click="selectNode(filteredRows[0])"
          >
            查看首个命中
          </button>
        </div>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>展示值</th>
              <th>字典值</th>
              <th>扩展值</th>
              <th>路径</th>
              <th>状态</th>
              <th>排序</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in filteredRows" :key="item.ID">
              <td>{{ item.ID }}</td>
              <td>{{ indent(item.level) }}{{ item.label }}</td>
              <td>{{ item.value }}</td>
              <td>{{ item.extend || "-" }}</td>
              <td>{{ item.path }}</td>
              <td>{{ item.status ? "启用" : "停用" }}</td>
              <td>{{ item.sort }}</td>
              <td>
                <div class="row">
                  <button class="btn ghost" @click="selectNode(item)">查看</button>
                  <button class="btn ghost" @click="prepareChild(item)">新增子项</button>
                </div>
              </td>
            </tr>
            <tr v-if="!filteredRows.length">
              <td colspan="8">暂无符合条件的节点，请调整筛选条件后重试。</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">{{ form.ID ? "编辑节点" : "新增节点" }}</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>展示值</label>
            <input v-model.trim="form.label" placeholder="输入节点展示值" />
          </div>
          <div class="field">
            <label>字典值</label>
            <input v-model.trim="form.value" placeholder="输入节点字典值" />
          </div>
          <div class="field">
            <label>扩展值</label>
            <input v-model.trim="form.extend" placeholder="可填写颜色、编码或补充值" />
          </div>
          <div class="field">
            <label>层级</label>
            <input v-model.number="form.level" type="number" min="1" />
          </div>
          <div class="field">
            <label>排序</label>
            <input v-model.number="form.sort" type="number" min="1" />
          </div>
          <div class="field">
            <label>状态</label>
            <select v-model="form.status">
              <option :value="true">启用</option>
              <option :value="false">停用</option>
            </select>
          </div>
        </div>
        <p class="subtitle">当前父节点：{{ parentLabel }}</p>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>表单校验</td><td>{{ formValidation }}</td></tr>
              <tr><td>节点建议</td><td>{{ formSuggestion }}</td></tr>
              <tr><td>层级分析</td><td>{{ formLevelHint }}</td></tr>
            </tbody>
          </table>
        </div>
        <div class="row">
          <button class="btn primary" :disabled="loading || !formValid" @click="save">保存节点</button>
          <button class="btn ghost" @click="openRootDraft">新增根节点</button>
          <button class="btn ghost" @click="resetForm">重置</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">最近维护提示</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>#</th>
                <th>内容</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(item, index) in recentActions" :key="item">
                <td>{{ index + 1 }}</td>
                <td>{{ item }}</td>
              </tr>
              <tr v-if="!recentActions.length">
                <td colspan="2">暂无操作记录</td>
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
import { useRoute, useRouter } from "vue-router";
import { getDictionaryDetailTreeApi, saveDictionaryDetailApi } from "../../api/admin";
import type { DictionaryDetailInfo } from "../../types";

type StatusFilter = "" | "enabled" | "disabled";
type FlatNode = DictionaryDetailInfo & {
  path: string;
  childCount: number;
  isLeaf: boolean;
};

const route = useRoute();
const router = useRouter();
const rows = ref<DictionaryDetailInfo[]>([]);
const filteredRows = ref<FlatNode[]>([]);
const selectedNode = ref<FlatNode | null>(null);
const statusMessage = ref("");
const loading = ref(false);
const recentActions = ref<string[]>([]);

const filters = reactive<{
  keyword: string;
  status: StatusFilter;
  level: string;
}>({
  keyword: "",
  status: "",
  level: "",
});

const form = reactive({
  ID: undefined as number | undefined,
  sysDictionaryID: 1,
  label: "",
  value: "",
  extend: "",
  level: 1,
  status: true,
  sort: 1,
  parentID: undefined as number | null | undefined,
});

const dictionaryId = computed(() => Number(route.params.id || 1));
const flatRows = computed<FlatNode[]>(() => flatten(rows.value));
const enabledCount = computed(() => flatRows.value.filter((item) => item.status).length);
const maxLevel = computed(() => flatRows.value.reduce((max, item) => Math.max(max, item.level), 0));
const rootCount = computed(() => flatRows.value.filter((item) => !item.parentID).length);
const leafCount = computed(() => flatRows.value.filter((item) => item.isLeaf).length);
const levelOptions = computed(() => Array.from(new Set(flatRows.value.map((item) => item.level))).sort((a, b) => a - b));
const levelCards = computed(() =>
  levelOptions.value.map((level) => ({
    level,
    count: flatRows.value.filter((item) => item.level === level).length,
  })),
);
const selectedNodeMeta = computed(() => {
  if (!selectedNode.value) {
    return null;
  }
  return selectedNode.value;
});
const currentAdvice = computed(() => {
  if (!flatRows.value.length) {
    return "当前字典尚无节点，建议先建立根节点与基础排序。";
  }
  if (!enabledCount.value) {
    return "所有节点均处于停用态，建议优先恢复线上仍需展示的节点。";
  }
  if (maxLevel.value >= 4) {
    return "当前层级较深，建议检查是否存在过深树结构影响前端选择体验。";
  }
  return "优先检查扩展值为空或子节点较多的节点，确认映射关系与展示语义。";
});
const hotLevelSummary = computed(() => {
  const hottest = levelCards.value.slice().sort((a, b) => b.count - a.count)[0];
  return hottest ? `第 ${hottest.level} 层节点最多，共 ${hottest.count} 个。` : "暂无层级分布信息";
});
const selectedAdvice = computed(() => {
  if (!selectedNode.value) {
    return "请选择节点查看建议";
  }
  if (!selectedNode.value.status) {
    return "该节点处于停用态，建议确认其子节点是否也应同步停用。";
  }
  if (selectedNode.value.isLeaf && !selectedNode.value.extend) {
    return "叶子节点暂无扩展值，若被前端组件引用，建议补充颜色、编码或别名。";
  }
  if (!selectedNode.value.isLeaf) {
    return `该节点包含 ${selectedNode.value.childCount} 个子节点，建议确认排序与层级是否合理。`;
  }
  return "该节点状态正常，可继续检查相邻排序与父子命名一致性。";
});
const formValid = computed(() => Boolean(form.label.trim() && form.value.trim() && form.level >= 1));
const formValidation = computed(() => {
  if (!form.label.trim()) {
    return "请填写节点展示值。";
  }
  if (!form.value.trim()) {
    return "请填写节点字典值。";
  }
  if (form.level < 1) {
    return "节点层级至少为 1。";
  }
  if (form.parentID && form.level === 1) {
    return "存在父节点时，层级建议大于 1。";
  }
  return "表单可提交。";
});
const formSuggestion = computed(() => {
  if (!form.parentID) {
    return "根节点建议承载大类语义，保持 label 与 value 简洁稳定。";
  }
  const parent = flatRows.value.find((item) => item.ID === form.parentID);
  if (!parent) {
    return "当前父节点未命中，建议重新选择父节点后再保存。";
  }
  return `建议与父节点 ${parent.label} 保持同一命名域，并确认排序是否连续。`;
});
const formLevelHint = computed(() => {
  if (!form.parentID) {
    return "当前为根节点维护，适合建立一级目录。";
  }
  const parent = flatRows.value.find((item) => item.ID === form.parentID);
  if (!parent) {
    return "当前父节点不存在于已加载树中。";
  }
  return `父节点位于第 ${parent.level} 层，当前节点建议处于第 ${parent.level + 1} 层。`;
});
const parentLabel = computed(() => {
  if (!form.parentID) {
    return "根节点";
  }
  return flatRows.value.find((item) => item.ID === form.parentID)?.label || `父节点 ${form.parentID}`;
});

async function load() {
  loading.value = true;
  try {
    rows.value = await getDictionaryDetailTreeApi(dictionaryId.value);
    form.sysDictionaryID = dictionaryId.value;
    applyFilters();
    if (selectedNode.value) {
      const current = flatRows.value.find((item) => item.ID === selectedNode.value?.ID) ?? null;
      if (current) {
        selectNode(current, false);
      } else if (filteredRows.value[0]) {
        selectNode(filteredRows.value[0], false);
      } else {
        hydrateRootDraft(false);
      }
    } else if (filteredRows.value[0]) {
      selectNode(filteredRows.value[0], false);
    } else {
      hydrateRootDraft(false);
    }
    statusMessage.value = `已加载字典 ${dictionaryId.value} 的 ${flatRows.value.length} 个节点。`;
  } finally {
    loading.value = false;
  }
}

async function back() {
  await router.push({ name: "dictionaries" });
}

function flatten(nodes: DictionaryDetailInfo[], trail: string[] = []): FlatNode[] {
  return nodes.flatMap((node) => {
    const nextTrail = [...trail, node.label];
    const childRows = flatten(node.children ?? [], nextTrail);
    return [
      {
        ...node,
        path: nextTrail.join(" / "),
        childCount: node.children?.length ?? 0,
        isLeaf: !(node.children?.length ?? 0),
      },
      ...childRows,
    ];
  });
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  filteredRows.value = flatRows.value.filter((item) => {
    const matchesKeyword =
      !keyword ||
      item.label.toLowerCase().includes(keyword) ||
      item.value.toLowerCase().includes(keyword) ||
      item.extend.toLowerCase().includes(keyword);
    const matchesStatus =
      !filters.status ||
      (filters.status === "enabled" ? item.status : !item.status);
    const matchesLevel = !filters.level || String(item.level) === filters.level;
    return matchesKeyword && matchesStatus && matchesLevel;
  });
  statusMessage.value = `当前命中 ${filteredRows.value.length} 个节点。`;
}

function resetFilters() {
  filters.keyword = "";
  filters.status = "";
  filters.level = "";
  filteredRows.value = [...flatRows.value];
  statusMessage.value = "已重置节点筛选条件。";
}

function pickLevel(level: number) {
  filters.level = String(level);
  applyFilters();
}

function selectNode(node: FlatNode, record = true) {
  selectedNode.value = node;
  hydrateForm(node);
  if (record) {
    noteAction(`已查看节点 ${node.path}`);
  }
}

function indent(level: number) {
  return `${"　".repeat(Math.max(0, level - 1))}`;
}

function hydrateForm(node: FlatNode) {
  form.ID = node.ID;
  form.sysDictionaryID = dictionaryId.value;
  form.label = node.label;
  form.value = node.value;
  form.extend = node.extend;
  form.level = node.level;
  form.status = node.status;
  form.sort = node.sort;
  form.parentID = node.parentID;
}

function hydrateRootDraft(record = true) {
  selectedNode.value = null;
  form.ID = undefined;
  form.sysDictionaryID = dictionaryId.value;
  form.label = "";
  form.value = "";
  form.extend = "";
  form.level = 1;
  form.status = true;
  form.sort = rootCount.value + 1;
  form.parentID = undefined;
  if (record) {
    noteAction("已切换到新增根节点模式");
  }
}

function openRootDraft() {
  hydrateRootDraft(true);
}

function prepareChild(node: FlatNode) {
  selectedNode.value = node;
  form.ID = undefined;
  form.sysDictionaryID = dictionaryId.value;
  form.label = `${node.label}子项`;
  form.value = `${node.value}_child_${node.childCount + 1}`;
  form.extend = node.extend;
  form.level = node.level + 1;
  form.status = true;
  form.sort = node.childCount + 1;
  form.parentID = node.ID;
  noteAction(`已基于节点 ${node.label} 创建子项草稿`);
  statusMessage.value = `已为 ${node.label} 准备子节点草稿。`;
}

function resetForm() {
  if (selectedNode.value) {
    hydrateForm(selectedNode.value);
    statusMessage.value = "已恢复当前节点内容。";
    return;
  }
  hydrateRootDraft(false);
  statusMessage.value = "已重置为新增根节点模式。";
}

function noteAction(message: string) {
  recentActions.value = [`${new Date().toLocaleString()} · ${message}`, ...recentActions.value].slice(0, 8);
}

async function copyNodeSummary() {
  if (!selectedNode.value) {
    return;
  }
  const text = [
    `字典：${dictionaryId.value}`,
    `节点：${selectedNode.value.label}`,
    `路径：${selectedNode.value.path}`,
    `状态：${selectedNode.value.status ? "启用" : "停用"}`,
    `排序：${selectedNode.value.sort}`,
    `建议：${selectedAdvice.value}`,
  ].join("\n");
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      window.prompt("当前环境不支持自动复制，请手动复制以下内容：", text);
    }
    noteAction(`已复制节点 ${selectedNode.value.label} 摘要`);
    statusMessage.value = "已复制当前节点摘要。";
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
      sysDictionaryID: dictionaryId.value,
      label: form.label.trim(),
      value: form.value.trim(),
      extend: form.extend.trim(),
      level: form.level,
      status: form.status,
      sort: form.sort,
      parentID: form.parentID,
    };
    const saved = await saveDictionaryDetailApi(payload);
    noteAction(`已保存节点 ${saved.label}`);
    statusMessage.value = `节点 ${saved.label} 已保存。`;
    await load();
    const current = flatRows.value.find((item) => item.ID === saved.ID) ?? null;
    if (current) {
      selectNode(current, false);
    }
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>
