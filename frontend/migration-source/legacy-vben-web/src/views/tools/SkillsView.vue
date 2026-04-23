<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">技能工作台</h3>
          <p class="subtitle">恢复技能索引、技能详情、文件资产和全局约束维护，并补充筛选、质量提示与摘要联动。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="loadAll">
            {{ loading ? "刷新中..." : "刷新技能" }}
          </button>
          <button class="btn ghost" :disabled="!currentSkillName" @click="copySkillSummary">复制技能摘要</button>
          <button class="btn primary" @click="startCreateSkill">新增技能</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

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
            <textarea v-model="globalConstraint" rows="6" />
          </div>
          <div class="row wrap">
            <button class="btn ghost" :disabled="loading" @click="loadAll">刷新</button>
            <button class="btn primary" :disabled="savingConstraint" @click="saveConstraint">
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
                      <button class="btn ghost" @click.stop="removeSkill(item.name)">删除</button>
                    </div>
                  </td>
                </tr>
                <tr v-if="!filteredSkills.length">
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
            <textarea v-model="draft.markdown" rows="12" />
          </div>

          <div v-if="validationIssues.length" class="state-banner error">
            <strong>保存前请修正：</strong>
            <ul>
              <li v-for="item in validationIssues" :key="item">{{ item }}</li>
            </ul>
          </div>

          <div class="row wrap">
            <label class="tag"><input v-model="draft.enabled" type="checkbox" /> 启用</label>
            <button class="btn primary" :disabled="savingSkill || validationIssues.length > 0" @click="saveSkill">
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
            <button class="btn ghost" :disabled="!currentSkillName" @click="createAsset">创建文件</button>
            <button class="btn primary" :disabled="!currentSkillName || !selectedAssetName || savingAsset" @click="saveAsset">
              {{ savingAsset ? "保存中..." : "保存文件" }}
            </button>
            <button class="btn ghost" :disabled="!assetContent" @click="copyAssetContent">复制内容</button>
          </div>
          <div class="field">
            <label>内容</label>
            <textarea v-model="assetContent" rows="12" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
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
} from "../../api/admin";
import type { SkillAssetRecord, SkillRecord, SkillSummary, SkillToolDefinition } from "../../types";

const assetKinds = ["script", "resource", "reference", "template"] as const;
const assetKindLabels = {
  script: "脚本",
  resource: "资源",
  reference: "参考",
  template: "模板",
} as const;

const skills = ref<SkillSummary[]>([]);
const tools = ref<SkillToolDefinition[]>([]);
const currentSkill = ref<SkillRecord | null>(null);
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
    case "resource":
      return currentSkill.value.resources;
    case "reference":
      return currentSkill.value.references;
    case "template":
      return currentSkill.value.templates;
    default:
      return currentSkill.value.scripts;
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

async function loadAll() {
  loading.value = true;
  error.value = "";
  try {
    const [skillRows, toolRows, constraint] = await Promise.all([
      getSkillListApi(),
      getSkillToolsApi(),
      getGlobalConstraintApi(),
    ]);
    skills.value = skillRows.sort((a, b) => b.updatedAt - a.updatedAt);
    tools.value = toolRows;
    globalConstraint.value = constraint.content;
    if (!currentSkillName.value && skills.value[0]) {
      await selectSkill(skills.value[0].name);
    }
    message.value = "已刷新技能索引、工具定义与全局约束。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "刷新技能失败";
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
  if (validationIssues.value.length) {
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
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存技能失败";
  } finally {
    savingSkill.value = false;
  }
}

async function removeSkill(name: string) {
  await deleteSkillApi(name);
  if (currentSkillName.value === name) {
    startCreateSkill();
  }
  message.value = `技能 ${name} 已删除`;
  await loadAll();
}

async function saveConstraint() {
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

async function copyAssetContent() {
  if (!assetContent.value) return;
  await copyText(assetContent.value, "已复制当前资产内容。");
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
