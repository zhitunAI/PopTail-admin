<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">自动代码工作台</h3>
          <p class="subtitle">把业务实体、字段、输出目标与生成策略整合成可落库的蓝图工作流，而不只是表单草稿。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" @click="applyPreset('crud')">CRUD 模板</button>
          <button class="btn ghost" @click="applyPreset('tree')">树结构模板</button>
          <button class="btn ghost" @click="resetDraft">恢复默认</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>字段数</h4>
        <p class="stat-value">{{ draft.fields.length }}</p>
        <p class="subtitle">已定义的实体字段</p>
      </div>
      <div class="stat-card">
        <h4>输出目标</h4>
        <p class="stat-value">{{ outputTargets.length }}</p>
        <p class="subtitle">{{ outputTargets.join(" / ") || "未选择" }}</p>
      </div>
      <div class="stat-card">
        <h4>质量门槛</h4>
        <p class="stat-value">{{ validationIssues.length ? `${validationIssues.length} 项` : "通过" }}</p>
        <p class="subtitle">{{ validationIssues[0] ?? "可以生成并写入蓝图台账" }}</p>
      </div>
      <div class="stat-card">
        <h4>最近生成</h4>
        <p class="stat-value">{{ lastGeneratedAt }}</p>
        <p class="subtitle">{{ generated.fileStem || "-" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">蓝图编辑</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>结构名称</label>
            <input v-model.trim="draft.entityName" placeholder="例如 RiskPolicy" />
          </div>
          <div class="field">
            <label>简称</label>
            <input v-model.trim="draft.alias" placeholder="例如 rp" />
          </div>
          <div class="field">
            <label>数据表</label>
            <input v-model.trim="draft.tableName" placeholder="例如 risk_policies" />
          </div>
          <div class="field">
            <label>包名</label>
            <input v-model.trim="draft.moduleName" placeholder="例如 compliance" />
          </div>
        </div>

        <div class="field">
          <label>业务描述</label>
          <textarea v-model.trim="draft.summary" rows="4" placeholder="描述接口、列表字段、编辑流转和筛选条件" />
        </div>

        <div class="toolbar-grid">
          <div class="field">
            <label>中文名称</label>
            <input v-model.trim="draft.description" placeholder="例如 风控策略" />
          </div>
          <div class="field">
            <label>模板</label>
            <select v-model="draft.templateName">
              <option value="crud">crud</option>
              <option value="tree">tree</option>
              <option value="base">base</option>
            </select>
          </div>
          <div class="field">
            <label>业务库</label>
            <input v-model.trim="draft.businessDb" placeholder="留空则使用默认库" />
          </div>
        </div>

        <div class="section">
          <div class="section-head-line">
            <strong>生成开关</strong>
            <span class="hint">按目标输出与辅助动作拆分</span>
          </div>
          <div class="tag-list">
            <label class="tag"><input v-model="draft.flags.generateWeb" type="checkbox" /> 前端</label>
            <label class="tag"><input v-model="draft.flags.generateServer" type="checkbox" disabled /> 后端</label>
            <label class="tag"><input v-model="draft.flags.autoCreateMenu" type="checkbox" /> 自动菜单</label>
            <label class="tag"><input v-model="draft.flags.autoCreateApi" type="checkbox" /> 自动 API</label>
            <label class="tag"><input v-model="draft.flags.autoMigrate" type="checkbox" /> 同步表结构</label>
            <label class="tag"><input v-model="draft.flags.gvaModel" type="checkbox" /> 基础模型</label>
            <label class="tag"><input v-model="draft.flags.onlyTemplate" type="checkbox" /> 仅模板</label>
            <label class="tag"><input v-model="draft.flags.isTree" type="checkbox" /> 树形结构</label>
          </div>
        </div>

        <div class="section">
          <div class="section-head-line">
            <strong>输出文件</strong>
            <span class="hint">控制台账中将来要补全的产物范围</span>
          </div>
          <div class="tag-list">
            <label class="tag"><input v-model="draft.outputs.api" type="checkbox" /> API</label>
            <label class="tag"><input v-model="draft.outputs.model" type="checkbox" /> Model</label>
            <label class="tag"><input v-model="draft.outputs.service" type="checkbox" /> Service</label>
            <label class="tag"><input v-model="draft.outputs.form" type="checkbox" /> Form</label>
            <label class="tag"><input v-model="draft.outputs.table" type="checkbox" /> Table</label>
          </div>
        </div>

        <div class="section">
          <div class="section-head-line">
            <strong>字段定义</strong>
            <span class="hint">至少保留一个可用字段，支持快速增删与字段推荐</span>
          </div>
          <div class="stack">
            <div v-for="(field, index) in draft.fields" :key="`${field.name}-${index}`" class="field-row">
              <input v-model.trim="field.name" placeholder="字段名" />
              <input v-model.trim="field.label" placeholder="中文说明" />
              <select v-model="field.type">
                <option value="string">string</option>
                <option value="int">int</option>
                <option value="bool">bool</option>
                <option value="time">time</option>
              </select>
              <button class="btn ghost" @click="removeField(index)">删除字段</button>
            </div>
          </div>
          <div class="row wrap top-gap">
            <button class="btn ghost" @click="addField">新增字段</button>
            <button class="btn ghost" @click="appendRecommendedFields">补常用字段</button>
          </div>
        </div>

        <div v-if="validationIssues.length" class="state-banner error">
          <strong>生成前请先修正：</strong>
          <ul>
            <li v-for="issue in validationIssues" :key="issue">{{ issue }}</li>
          </ul>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="validationIssues.length > 0" @click="generateBlueprint">
            生成蓝图
          </button>
          <button class="btn ghost" :disabled="validationIssues.length > 0 || saving" @click="saveBlueprint">
            {{ saving ? "写入中..." : "写入台账" }}
          </button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">输出摘要</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>列表路由</td><td>{{ generated.routes.list }}</td></tr>
              <tr><td>编辑路由</td><td>{{ generated.routes.form }}</td></tr>
              <tr><td>API Group</td><td>{{ generated.apiGroup }}</td></tr>
              <tr><td>文件前缀</td><td>{{ generated.fileStem }}</td></tr>
              <tr><td>预估文件数</td><td>{{ generated.files.length }}</td></tr>
              <tr><td>风险判断</td><td>{{ generationAdvice }}</td></tr>
            </tbody>
          </table>
        </div>

        <div class="section top-gap">
          <div class="section-head-line">
            <strong>产物清单</strong>
            <span class="hint">根据当前输出选项自动整理</span>
          </div>
          <div class="tag-list">
            <span v-for="item in generated.files" :key="item" class="tag">{{ item }}</span>
            <span v-if="!generated.files.length" class="tag">暂无输出文件</span>
          </div>
        </div>

        <div class="section top-gap">
          <div class="section-head-line">
            <strong>推荐后续动作</strong>
          </div>
          <ul class="bullet-list">
            <li v-for="item in recommendations" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">蓝图代码</h3>
        <p class="subtitle">可直接继续进入自动代码管理、MCP 生成或插件清单打包流程。</p>
        <pre class="code-block">{{ generated.preview }}</pre>
      </div>

      <div class="card">
        <h3 class="title">字段检查</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>字段</th>
                <th>说明</th>
                <th>类型</th>
                <th>状态</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="field in draft.fields" :key="`${field.name}-${field.label}`">
                <td>{{ field.name || "-" }}</td>
                <td>{{ field.label || "-" }}</td>
                <td>{{ field.type }}</td>
                <td>{{ field.name && field.label ? "可生成" : "待补全" }}</td>
              </tr>
              <tr v-if="!draft.fields.length">
                <td colspan="4">暂无字段，请先新增字段。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { saveAutoCodeRegistryApi } from "../../api/admin";

interface BlueprintDraft {
  entityName: string;
  alias: string;
  tableName: string;
  moduleName: string;
  description: string;
  templateName: string;
  businessDb: string;
  summary: string;
  outputs: Record<string, boolean>;
  flags: {
    generateWeb: boolean;
    generateServer: boolean;
    autoCreateMenu: boolean;
    autoCreateApi: boolean;
    autoMigrate: boolean;
    gvaModel: boolean;
    onlyTemplate: boolean;
    isTree: boolean;
  };
  fields: Array<{ name: string; label: string; type: string }>;
}

const defaultDraft: BlueprintDraft = {
  entityName: "RiskPolicy",
  alias: "rp",
  tableName: "risk_policies",
  moduleName: "compliance",
  description: "风控策略",
  templateName: "crud",
  businessDb: "",
  summary: "需要列表、搜索、启停和详情编辑页，支持按规则等级筛选。",
  outputs: {
    api: true,
    model: true,
    service: true,
    form: true,
    table: true,
  },
  flags: {
    generateWeb: true,
    generateServer: true,
    autoCreateMenu: true,
    autoCreateApi: true,
    autoMigrate: true,
    gvaModel: true,
    onlyTemplate: false,
    isTree: false,
  },
  fields: [
    { name: "policyCode", label: "策略编号", type: "string" },
    { name: "riskLevel", label: "风险等级", type: "int" },
  ],
};

const draft = reactive<BlueprintDraft>(JSON.parse(JSON.stringify(defaultDraft)));
const message = ref("");
const error = ref("");
const saving = ref(false);
const lastGeneratedAt = ref("未生成");

const generated = reactive({
  routes: {
    list: "/system/tools/autocode",
    form: "/system/tools/autocode/edit",
  },
  apiGroup: "compliance",
  fileStem: "riskPolicy",
  preview: "",
  files: [] as string[],
});

const outputTargets = computed(() =>
  Object.entries(draft.outputs)
    .filter(([, enabled]) => enabled)
    .map(([key]) => key.toUpperCase()),
);

const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!draft.entityName.trim()) issues.push("结构名称不能为空。");
  if (!draft.alias.trim()) issues.push("简称不能为空。");
  if (!draft.tableName.trim()) issues.push("数据表不能为空。");
  if (!draft.moduleName.trim()) issues.push("包名不能为空。");
  if (!draft.description.trim()) issues.push("中文名称不能为空。");
  if (!draft.summary.trim()) issues.push("业务描述不能为空。");
  if (!outputTargets.value.length) issues.push("至少需要选择一个输出目标。");
  if (!draft.fields.length) issues.push("至少需要一个字段定义。");
  if (draft.fields.some((field) => !field.name.trim() || !field.label.trim())) {
    issues.push("字段名与中文说明必须完整填写。");
  }
  return issues;
});

const generationAdvice = computed(() => {
  if (validationIssues.value.length) {
    return "请先补全蓝图信息";
  }
  if (draft.fields.length >= 8) {
    return "字段较多，建议生成后进入 AutoCodeAdmin 做二次审查";
  }
  if (draft.flags.isTree || draft.templateName === "tree") {
    return "树结构场景建议同步核对父子字段与菜单层级";
  }
  return "蓝图结构较稳定，可直接入库";
});

const recommendations = computed(() => {
  const items = [
    `先核对 ${generated.routes.list} 与 ${generated.routes.form} 是否符合路由规划。`,
    `确认 API 分组 ${generated.apiGroup} 与包名 ${draft.moduleName} 的边界一致。`,
  ];
  if (!draft.flags.autoCreateMenu) {
    items.push("当前未勾选自动菜单，后续需手工补齐菜单台账。");
  }
  if (!draft.flags.autoCreateApi) {
    items.push("当前未勾选自动 API，生成后需手工维护 API 台账。");
  }
  if (!draft.outputs.form || !draft.outputs.table) {
    items.push("当前页面层产物不完整，建议确认是否只生成服务端骨架。");
  }
  if (!draft.fields.some((field) => field.type === "time")) {
    items.push("如涉及审计或发布记录，建议补充时间字段。");
  }
  return items;
});

function toFileStem(name: string) {
  if (!name) return "entity";
  return name.charAt(0).toLowerCase() + name.slice(1);
}

function buildFiles() {
  return outputTargets.value.map((item) => `${generated.fileStem}.${item.toLowerCase()}.ts`);
}

function generateBlueprint() {
  error.value = "";
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "蓝图信息不完整";
    return;
  }

  generated.routes.list = `/${draft.moduleName}/${draft.alias}/list`;
  generated.routes.form = `/${draft.moduleName}/${draft.alias}/edit`;
  generated.apiGroup = draft.moduleName || "default";
  generated.fileStem = toFileStem(draft.entityName.trim());
  generated.files = buildFiles();
  generated.preview = JSON.stringify(
    {
      entity: draft.entityName.trim(),
      description: draft.description.trim(),
      alias: draft.alias.trim(),
      table: draft.tableName.trim(),
      module: draft.moduleName.trim(),
      template: draft.templateName,
      businessDb: draft.businessDb.trim() || "default",
      summary: draft.summary.trim(),
      flags: draft.flags,
      fields: draft.fields.map((field) => ({
        name: field.name.trim(),
        label: field.label.trim(),
        type: field.type,
      })),
      outputs: outputTargets.value,
      routes: generated.routes,
      files: generated.files,
      recommendations: recommendations.value,
    },
    null,
    2,
  );
  lastGeneratedAt.value = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  message.value = `已生成 ${draft.entityName} 蓝图，可继续写入台账。`;
}

async function saveBlueprint() {
  generateBlueprint();
  if (!generated.preview || validationIssues.value.length) {
    return;
  }

  saving.value = true;
  error.value = "";
  try {
    await saveAutoCodeRegistryApi(JSON.parse(generated.preview));
    message.value = `已写入蓝图台账：${draft.entityName}`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "写入蓝图台账失败";
  } finally {
    saving.value = false;
  }
}

function addField() {
  draft.fields.push({ name: "", label: "", type: "string" });
}

function removeField(index: number) {
  if (draft.fields.length === 1) {
    error.value = "至少保留一个字段。";
    return;
  }
  draft.fields.splice(index, 1);
}

function appendRecommendedFields() {
  const recommended = [
    { name: "createdAt", label: "创建时间", type: "time" },
    { name: "updatedAt", label: "更新时间", type: "time" },
    { name: "enabled", label: "启用状态", type: "bool" },
  ];
  for (const item of recommended) {
    if (!draft.fields.some((field) => field.name === item.name)) {
      draft.fields.push({ ...item });
    }
  }
  message.value = "已补充常用字段建议。";
}

function applyPreset(kind: "crud" | "tree") {
  if (kind === "tree") {
    draft.entityName = "RegionNode";
    draft.alias = "region";
    draft.tableName = "region_nodes";
    draft.moduleName = "geo";
    draft.description = "区域节点";
    draft.templateName = "tree";
    draft.summary = "需要树形列表、层级新增、节点排序和启停控制。";
    draft.flags.isTree = true;
    draft.fields = [
      { name: "parentId", label: "父节点", type: "int" },
      { name: "nodeName", label: "节点名称", type: "string" },
      { name: "sortOrder", label: "排序值", type: "int" },
    ];
    message.value = "已应用树结构模板。";
  } else {
    Object.assign(draft, JSON.parse(JSON.stringify(defaultDraft)));
    message.value = "已应用 CRUD 模板。";
  }
  generateBlueprint();
}

function resetDraft() {
  Object.assign(draft, JSON.parse(JSON.stringify(defaultDraft)));
  error.value = "";
  message.value = "已恢复默认蓝图草稿。";
  generateBlueprint();
}

generateBlueprint();
</script>
