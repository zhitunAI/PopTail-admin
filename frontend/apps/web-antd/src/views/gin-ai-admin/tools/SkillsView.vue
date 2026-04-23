<script setup lang="ts">
import type { SkillAssetRecord, SkillRecord, SkillSummary, SkillToolDefinition } from "#/types/gin-ai-admin";

import { computed, onMounted, reactive, ref } from "vue";

import { useAccess } from "@vben/access";
import { Page } from "@vben/common-ui";
import { IconifyIcon } from "@vben/icons";
import { useAccessStore } from "@vben/stores";

import { Alert, Button } from "ant-design-vue";

import {
  createSkillAssetApi,
  deleteSkillApi,
  getGlobalConstraintApi,
  getSkillAssetApi,
  getSkillDetailApi,
  getSkillListApi,
  getSkillToolsApi,
  saveGlobalConstraintApi,
  saveSkillApi,
  saveSkillAssetApi,
} from "#/api/gin-ai-admin/admin";

const assetKinds = ["script", "resource", "reference", "template"] as const;
const assetKindLabels = {
  script: "脚本",
  resource: "资源",
  reference: "参考",
  template: "模板",
} as const;

type SkillsButtonAction =
  | "copy"
  | "create"
  | "createAsset"
  | "delete"
  | "export"
  | "refresh"
  | "save";

const SKILLS_BUTTON_ACCESS_CODES: Record<SkillsButtonAction, string[]> = {
  copy: [
    "复制",
    "复制技能摘要",
    "复制约束",
    "复制内容",
    "btn:复制",
    "btn:复制技能摘要",
    "btn:复制约束",
    "btn:复制内容",
    "btn:skills:copy",
    "btn:/system/tools/skills:copy",
    "btn:/systemTools/skills:copy",
  ],
  create: [
    "新增技能",
    "新增",
    "创建技能",
    "btn:新增技能",
    "btn:新增",
    "btn:创建技能",
    "btn:skills:create",
    "btn:/system/tools/skills:create",
    "btn:/systemTools/skills:create",
  ],
  createAsset: [
    "创建文件",
    "创建脚本",
    "创建资源",
    "创建参考",
    "创建模板",
    "btn:创建文件",
    "btn:创建脚本",
    "btn:创建资源",
    "btn:创建参考",
    "btn:创建模板",
    "btn:skills:createAsset",
    "btn:/system/tools/skills:createAsset",
    "btn:/systemTools/skills:createAsset",
  ],
  delete: [
    "删除",
    "删除技能",
    "btn:删除",
    "btn:删除技能",
    "btn:skills:delete",
    "btn:/system/tools/skills:delete",
    "btn:/systemTools/skills:delete",
  ],
  export: [
    "导出",
    "导出技能",
    "导出内容",
    "打包",
    "btn:导出",
    "btn:导出技能",
    "btn:导出内容",
    "btn:打包",
    "btn:skills:export",
    "btn:/system/tools/skills:export",
    "btn:/systemTools/skills:export",
    "btn:skills:package",
    "btn:/system/tools/skills:package",
    "btn:/systemTools/skills:package",
  ],
  refresh: [
    "刷新",
    "刷新技能",
    "查询",
    "btn:刷新",
    "btn:刷新技能",
    "btn:查询",
    "btn:skills:refresh",
    "btn:/system/tools/skills:refresh",
    "btn:/systemTools/skills:refresh",
  ],
  save: [
    "保存技能",
    "保存配置",
    "保存文件",
    "保存内容",
    "保存约束",
    "编辑",
    "btn:保存技能",
    "btn:保存配置",
    "btn:保存文件",
    "btn:保存内容",
    "btn:保存约束",
    "btn:编辑",
    "btn:skills:save",
    "btn:/system/tools/skills:save",
    "btn:/systemTools/skills:save",
    "btn:skills:update",
    "btn:/system/tools/skills:update",
    "btn:/systemTools/skills:update",
  ],
};

const SKILLS_BUTTON_ACCESS_LABELS: Record<SkillsButtonAction, string[]> = {
  copy: ["复制", "复制技能摘要", "复制约束", "复制内容", "copy"],
  create: ["新增技能", "新增", "创建技能", "create"],
  createAsset: ["创建文件", "创建脚本", "创建资源", "创建参考", "创建模板", "createAsset"],
  delete: ["删除", "删除技能", "delete"],
  export: ["导出", "导出技能", "导出内容", "打包", "export", "package"],
  refresh: ["刷新", "刷新技能", "查询", "refresh"],
  save: ["保存技能", "保存配置", "保存文件", "保存内容", "保存约束", "编辑", "save", "update", "edit"],
};

const SKILLS_BUTTON_ACCESS_CODE_PREFIXES = [
  "btn:skills:",
  "btn:/system/tools/skills:",
  "btn:/systemTools/skills:",
];

const skills = ref<SkillSummary[]>([]);
const tools = ref<SkillToolDefinition[]>([]);
const currentSkill = ref<null | SkillRecord>(null);
const currentSkillName = ref("");
const globalConstraint = ref("");
const activeAssetKind = ref<(typeof assetKinds)[number]>("script");
const selectedAssetName = ref("");
const newAssetName = ref("");
const assetContent = ref("");
const skillQuery = ref("");
const message = ref("");
const error = ref("");
const loading = ref(false);
const savingSkill = ref(false);
const savingAsset = ref(false);
const savingConstraint = ref(false);
const accessStore = useAccessStore();
const { hasAccessByCodes } = useAccess();
const draft = reactive({
  name: "",
  description: "",
  allowedTools: "",
  context: "",
  agent: "",
  markdown: "",
  enabled: true,
  tags: "",
});

const filteredSkills = computed(() => {
  const keyword = skillQuery.value.trim().toLowerCase();
  return skills.value.filter(
    (item) =>
      !keyword ||
      item.name.toLowerCase().includes(keyword) ||
      item.description.toLowerCase().includes(keyword) ||
      item.tags.join(",").toLowerCase().includes(keyword),
  );
});

const currentAssets = computed<SkillAssetRecord[]>(() => {
  if (!currentSkill.value) {
    return [];
  }
  switch (activeAssetKind.value) {
    case "reference": {
      return currentSkill.value.references;
    }
    case "resource": {
      return currentSkill.value.resources;
    }
    case "template": {
      return currentSkill.value.templates;
    }
    default: {
      return currentSkill.value.scripts;
    }
  }
});

const enabledCount = computed(() => skills.value.filter((item) => item.enabled).length);
const tagCount = computed(() =>
  draft.tags
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean).length,
);
const assetCount = computed(() => {
  if (!currentSkill.value) return 0;
  return (
    currentSkill.value.scripts.length +
    currentSkill.value.resources.length +
    currentSkill.value.references.length +
    currentSkill.value.templates.length
  );
});
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!draft.name.trim()) issues.push("技能名称不能为空。");
  if (!draft.description.trim()) issues.push("技能描述不能为空。");
  if (!draft.markdown.trim()) issues.push("技能 Markdown 不能为空。");
  if (!draft.allowedTools.trim()) issues.push("建议至少填写 Allowed Tools。");
  return issues;
});
const skillAdvice = computed(() => {
  if (!currentSkill.value) return "新技能草稿，建议先补齐描述、工具和 Markdown 结构。";
  if (!currentSkill.value.enabled) return "当前技能处于停用态，建议确认是否仍需保留。";
  if (!assetCount.value) return "当前技能没有配套资产，建议补充脚本、参考或模板。";
  return "技能结构较完整，可继续校验资产内容与约束语义。";
});
const currentSkillSaveAction = computed<"create" | "save">(() => (currentSkill.value ? "save" : "create"));
const skillsButtonAccessCandidates = new Set<string>(Object.values(SKILLS_BUTTON_ACCESS_CODES).flat());
const skillsButtonAccessLabels = Object.values(SKILLS_BUTTON_ACCESS_LABELS).flat();
const hasSkillsButtonAccessEnvelope = computed(() =>
  accessStore.accessCodes.some(
    (code) =>
      skillsButtonAccessCandidates.has(code) ||
      SKILLS_BUTTON_ACCESS_CODE_PREFIXES.some((prefix) => code.startsWith(prefix)) ||
      skillsButtonAccessLabels.some(
        (label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`),
      ),
  ),
);
const canSaveSkillDraft = computed(() => canUseSkillsAction(currentSkillSaveAction.value));

function hasFlexibleSkillsButtonAccess(action: SkillsButtonAction) {
  if (hasAccessByCodes(SKILLS_BUTTON_ACCESS_CODES[action])) {
    return true;
  }
  const labels = SKILLS_BUTTON_ACCESS_LABELS[action];
  return accessStore.accessCodes.some((code) =>
    labels.some((label) => code === label || code === `btn:${label}` || code.endsWith(`:${label}`)),
  );
}

function canUseSkillsAction(action: SkillsButtonAction) {
  return !hasSkillsButtonAccessEnvelope.value || hasFlexibleSkillsButtonAccess(action);
}

function denySkillsAction(action: SkillsButtonAction, label = SKILLS_BUTTON_ACCESS_LABELS[action][0]) {
  if (canUseSkillsAction(action)) {
    return false;
  }
  error.value = `无按钮权限，当前账号不能执行「${label}」操作。`;
  return true;
}

async function loadAll(showMessage = false) {
  if (showMessage && denySkillsAction("refresh", "刷新技能")) {
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const [skillRows, toolRows, constraint] = await Promise.all([
      getSkillListApi(),
      getSkillToolsApi(),
      getGlobalConstraintApi(),
    ]);
    skills.value = skillRows.toSorted((a, b) => b.updatedAt - a.updatedAt);
    tools.value = toolRows;
    globalConstraint.value = constraint.content;
    if (!currentSkillName.value && skills.value[0]) {
      await selectSkill(skills.value[0].name);
    }
    message.value = "已刷新技能索引、工具定义与全局约束。";
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "刷新技能失败";
  } finally {
    loading.value = false;
  }
}

async function selectSkill(name: string) {
  currentSkill.value = await getSkillDetailApi(name);
  currentSkillName.value = name;
  draft.name = currentSkill.value.name;
  draft.description = currentSkill.value.description;
  draft.allowedTools = currentSkill.value.allowedTools;
  draft.context = currentSkill.value.context;
  draft.agent = currentSkill.value.agent;
  draft.markdown = currentSkill.value.markdown;
  draft.enabled = currentSkill.value.enabled;
  draft.tags = currentSkill.value.tags.join(", ");
  selectedAssetName.value = currentAssets.value[0]?.name || "";
  await loadAssetContent();
}

function startCreateSkill() {
  currentSkill.value = null;
  currentSkillName.value = "";
  draft.name = "";
  draft.description = "";
  draft.allowedTools = "";
  draft.context = "";
  draft.agent = "";
  draft.markdown = "## Instructions\n- 描述使用场景\n";
  draft.enabled = true;
  draft.tags = "";
  selectedAssetName.value = "";
  assetContent.value = "";
  message.value = "已打开新技能草稿。";
}

async function saveSkill() {
  if (
    denySkillsAction(
      currentSkillSaveAction.value,
      currentSkillSaveAction.value === "create" ? "新增技能" : "保存技能",
    )
  ) {
    return;
  }
  if (validationIssues.value.length > 0) {
    error.value = validationIssues.value[0] ?? "技能信息不完整";
    return;
  }
  savingSkill.value = true;
  error.value = "";
  try {
    const saved = await saveSkillApi({
      name: draft.name.trim(),
      description: draft.description.trim(),
      allowedTools: draft.allowedTools.trim(),
      context: draft.context.trim(),
      agent: draft.agent.trim(),
      markdown: draft.markdown,
      enabled: draft.enabled,
      tags: draft.tags.split(",").map((item) => item.trim()).filter(Boolean),
    });
    currentSkill.value = saved;
    currentSkillName.value = saved.name;
    message.value = `技能 ${saved.name} 已保存`;
    await loadAll();
  } catch (error_) {
    error.value = error_ instanceof Error ? error_.message : "保存技能失败";
  } finally {
    savingSkill.value = false;
  }
}

async function removeSkill(name: string) {
  if (denySkillsAction("delete", "删除技能")) {
    return;
  }
  await deleteSkillApi(name);
  if (currentSkillName.value === name) {
    startCreateSkill();
  }
  message.value = `技能 ${name} 已删除`;
  await loadAll();
}

async function saveConstraint() {
  if (denySkillsAction("save", "保存约束")) {
    return;
  }
  savingConstraint.value = true;
  try {
    const saved = await saveGlobalConstraintApi(globalConstraint.value);
    globalConstraint.value = saved.content;
    message.value = "全局约束已保存";
  } finally {
    savingConstraint.value = false;
  }
}

async function createAsset() {
  if (denySkillsAction("createAsset", "创建文件")) {
    return;
  }
  if (!currentSkillName.value || !newAssetName.value.trim()) {
    message.value = "请先选择技能并填写文件名";
    return;
  }
  await createSkillAssetApi(activeAssetKind.value, {
    skillName: currentSkillName.value,
    name: newAssetName.value.trim(),
  });
  message.value = `${assetKindLabels[activeAssetKind.value]} ${newAssetName.value.trim()} 已创建`;
  await selectSkill(currentSkillName.value);
  selectedAssetName.value = newAssetName.value.trim();
  newAssetName.value = "";
  await loadAssetContent();
}

async function switchAssetKind(kind: (typeof assetKinds)[number]) {
  activeAssetKind.value = kind;
  selectedAssetName.value = currentAssets.value[0]?.name || "";
  await loadAssetContent();
}

async function loadAssetContent() {
  if (!currentSkillName.value || !selectedAssetName.value) {
    assetContent.value = "";
    return;
  }
  const file = await getSkillAssetApi(activeAssetKind.value, {
    skillName: currentSkillName.value,
    name: selectedAssetName.value,
  });
  assetContent.value = file.content;
}

async function saveAsset() {
  if (denySkillsAction("save", "保存文件")) {
    return;
  }
  if (!currentSkillName.value || !selectedAssetName.value) {
    message.value = "请先选择文件";
    return;
  }
  savingAsset.value = true;
  try {
    await saveSkillAssetApi(activeAssetKind.value, {
      skillName: currentSkillName.value,
      name: selectedAssetName.value,
      content: assetContent.value,
    });
    message.value = `${assetKindLabels[activeAssetKind.value]} ${selectedAssetName.value} 已保存`;
    await selectSkill(currentSkillName.value);
  } finally {
    savingAsset.value = false;
  }
}

async function copySkillSummary() {
  if (denySkillsAction("copy", "复制技能摘要")) {
    return;
  }
  if (!currentSkill.value) return;
  const text = [
    `技能：${currentSkill.value.name}`,
    `描述：${currentSkill.value.description}`,
    `工具：${currentSkill.value.allowedTools}`,
    `标签：${currentSkill.value.tags.join(", ") || "-"}`,
    `状态：${currentSkill.value.enabled ? "启用" : "停用"}`,
  ].join("\n");
  await copyText(text, `已复制技能 ${currentSkill.value.name} 摘要。`);
}

async function copyGlobalConstraint() {
  if (denySkillsAction("copy", "复制约束")) {
    return;
  }
  if (!globalConstraint.value.trim()) return;
  await copyText(globalConstraint.value, "已复制全局约束。");
}

async function copyAssetContent() {
  if (denySkillsAction("copy", "复制内容")) {
    return;
  }
  if (!assetContent.value) return;
  await copyText(assetContent.value, "已复制当前资产内容。");
}

function exportSkillBundle() {
  if (denySkillsAction("export", "导出技能")) {
    return;
  }
  if (!currentSkill.value || typeof window === "undefined") {
    return;
  }
  const content = JSON.stringify(
    {
      exportedAt: new Date().toISOString(),
      globalConstraint: globalConstraint.value,
      skill: currentSkill.value,
    },
    null,
    2,
  );
  const blob = new Blob([content], { type: "application/json;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `${currentSkill.value.name}-skill.json`;
  link.click();
  window.URL.revokeObjectURL(url);
  message.value = `已导出技能 ${currentSkill.value.name}。`;
}

function exportAssetContent() {
  if (denySkillsAction("export", "导出内容")) {
    return;
  }
  if (!assetContent.value || !selectedAssetName.value || typeof window === "undefined") {
    return;
  }
  const blob = new Blob([assetContent.value], { type: "text/plain;charset=utf-8" });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = selectedAssetName.value;
  link.click();
  window.URL.revokeObjectURL(url);
  message.value = `已导出资产 ${selectedAssetName.value}。`;
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
  void loadAll();
});
</script>

<template>
  <Page>
    <template #title>
      <div class="flex items-center gap-2">
        <IconifyIcon class="text-lg" icon="lucide:brain-circuit" />
        <span>技能管理</span>
      </div>
    </template>
    <template #extra>
      <div class="flex flex-wrap items-center gap-2">
        <Button :loading="loading" :disabled="!canUseSkillsAction('refresh')" @click="loadAll(true)">刷新技能</Button>
        <Button :disabled="!currentSkillName || !canUseSkillsAction('copy')" @click="copySkillSummary">复制技能摘要</Button>
        <Button :disabled="!currentSkill || !canUseSkillsAction('export')" @click="exportSkillBundle">导出技能</Button>
        <Button v-if="canUseSkillsAction('create')" type="primary" @click="startCreateSkill">新增技能</Button>
      </div>
    </template>

    <Alert v-if="message" class="mb-4" show-icon type="info" :message="message" />
    <Alert v-if="error" class="mb-4" show-icon type="error" :message="error" />

    <div class="muted-grid">
      <div class="stat-card">
        <h4>技能总数</h4>
        <p class="stat-value">{{ skills.length }}</p>
        <p class="subtitle">启用 {{ enabledCount }} / 停用 {{ skills.length - enabledCount }}</p>
      </div>
      <div class="stat-card">
        <h4>工具支持</h4>
        <p class="stat-value">{{ tools.length }}</p>
        <p class="subtitle">当前技能工具定义数量</p>
      </div>
      <div class="stat-card">
        <h4>当前技能</h4>
        <p class="stat-value">{{ currentSkillName || "-" }}</p>
        <p class="subtitle">{{ currentSkill ? skillAdvice : "请选择或新建技能" }}</p>
      </div>
      <div class="stat-card">
        <h4>当前资产</h4>
        <p class="stat-value">{{ currentAssets.length }}</p>
        <p class="subtitle">{{ assetKindLabels[activeAssetKind] }} / 总资产 {{ assetCount }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="stack">
        <div class="card">
          <h3 class="title">全局约束</h3>
          <div class="field">
            <label>约束内容</label>
            <textarea v-model="globalConstraint" rows="6"></textarea>
          </div>
          <div class="row wrap">
            <button class="btn ghost" :disabled="loading || !canUseSkillsAction('refresh')" @click="loadAll(true)">刷新</button>
            <button class="btn ghost" :disabled="!globalConstraint.trim() || !canUseSkillsAction('copy')" @click="copyGlobalConstraint">复制约束</button>
            <button class="btn primary" :disabled="savingConstraint || !canUseSkillsAction('save')" @click="saveConstraint">
              {{ savingConstraint ? "保存中..." : "保存约束" }}
            </button>
          </div>
          <div class="tag-list top-gap">
            <span v-for="tool in tools" :key="tool.key" class="tag">{{ tool.label }} · {{ tool.summary }}</span>
          </div>
        </div>

        <div class="card">
          <div class="row between wrap">
            <h3 class="title">技能索引</h3>
            <input v-model.trim="skillQuery" placeholder="按名称/描述/标签筛选" />
          </div>
          <div class="data-table">
            <table>
              <thead>
                <tr>
                  <th>技能</th>
                  <th>标签</th>
                  <th>状态</th>
                  <th>更新时间</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="item in filteredSkills"
                  :key="item.name"
                  :class="{ selected: currentSkillName === item.name }"
                  @click="selectSkill(item.name)"
                >
                  <td>{{ item.name }}</td>
                  <td>{{ item.tags.join(", ") || "-" }}</td>
                  <td>{{ item.enabled ? "启用" : "停用" }}</td>
                  <td>{{ formatTime(item.updatedAt) }}</td>
                  <td>
                    <div class="row wrap">
                      <button class="btn ghost" @click.stop="selectSkill(item.name)">打开</button>
                      <button v-if="canUseSkillsAction('delete')" class="btn ghost" @click.stop="removeSkill(item.name)">删除</button>
                    </div>
                  </td>
                </tr>
                <tr v-if="filteredSkills.length === 0">
                  <td colspan="5">暂无匹配技能</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div class="stack">
        <div class="card">
          <h3 class="title">技能配置</h3>
          <div class="toolbar-grid">
            <div class="field">
              <label>Name</label>
              <input v-model.trim="draft.name" placeholder="例如 parity-restore" />
            </div>
            <div class="field">
              <label>标签</label>
              <input v-model.trim="draft.tags" placeholder="使用逗号分隔" />
            </div>
          </div>
          <div class="toolbar-grid">
            <div class="field">
              <label>Description</label>
              <input v-model.trim="draft.description" />
            </div>
            <div class="field">
              <label>Allowed Tools</label>
              <input v-model.trim="draft.allowedTools" />
            </div>
            <div class="field">
              <label>Context</label>
              <input v-model.trim="draft.context" />
            </div>
            <div class="field">
              <label>Agent</label>
              <input v-model.trim="draft.agent" />
            </div>
          </div>
          <div class="field">
            <label>Markdown</label>
            <textarea v-model="draft.markdown" rows="12"></textarea>
          </div>

          <div v-if="validationIssues.length > 0" class="state-banner error">
            <strong>保存前请修正：</strong>
            <ul>
              <li v-for="item in validationIssues" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div class="row wrap">
            <label class="tag"><input v-model="draft.enabled" type="checkbox" /> 启用</label>
            <button class="btn primary" :disabled="savingSkill || validationIssues.length > 0 || !canSaveSkillDraft" @click="saveSkill">
              {{ savingSkill ? "保存中..." : "保存技能" }}
            </button>
          </div>

          <div class="data-table top-gap">
            <table>
              <tbody>
                <tr><td>标签数</td><td>{{ tagCount }}</td></tr>
                <tr><td>资产总数</td><td>{{ assetCount }}</td></tr>
                <tr><td>质量判断</td><td>{{ skillAdvice }}</td></tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="card">
          <h3 class="title">文件资产</h3>
          <div class="row wrap">
            <button
              v-for="kind in assetKinds"
              :key="kind"
              class="btn ghost"
              :class="{ active: activeAssetKind === kind }"
              @click="switchAssetKind(kind)"
            >
              {{ assetKindLabels[kind] }}
            </button>
          </div>
          <div class="toolbar-grid">
            <div class="field">
              <label>新文件名</label>
              <input v-model.trim="newAssetName" placeholder="例如 check-gap.sh" />
            </div>
            <div class="field">
              <label>当前文件</label>
              <select v-model="selectedAssetName" @change="loadAssetContent">
                <option v-for="file in currentAssets" :key="file.name" :value="file.name">{{ file.name }}</option>
              </select>
            </div>
          </div>
          <div class="row wrap">
            <button class="btn ghost" :disabled="!currentSkillName || !canUseSkillsAction('createAsset')" @click="createAsset">创建文件</button>
            <button class="btn primary" :disabled="!currentSkillName || !selectedAssetName || savingAsset || !canUseSkillsAction('save')" @click="saveAsset">
              {{ savingAsset ? "保存中..." : "保存文件" }}
            </button>
            <button class="btn ghost" :disabled="!assetContent || !canUseSkillsAction('copy')" @click="copyAssetContent">复制内容</button>
            <button class="btn ghost" :disabled="!assetContent || !selectedAssetName || !canUseSkillsAction('export')" @click="exportAssetContent">导出内容</button>
          </div>
          <div class="field">
            <label>内容</label>
            <textarea v-model="assetContent" rows="12"></textarea>
          </div>
        </div>
      </div>
    </div>
</Page>
</template>
