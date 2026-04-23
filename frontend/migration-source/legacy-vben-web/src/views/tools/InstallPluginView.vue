<template>
  <div class="stack">
    <div class="card">
      <div class="section-head">
        <div>
          <h3 class="title">插件安装工作台</h3>
          <p class="subtitle">
            基于当前安装台账接口完成插件登记、安装清单解析、风险提示与最近安装回看，帮助发布与联调流程落账。
          </p>
        </div>
        <button class="btn ghost" :disabled="loading" @click="loadRecords">
          {{ loading ? "刷新中..." : "刷新台账" }}
        </button>
      </div>

      <div class="summary-grid">
        <article class="metric-card">
          <span class="metric-label">安装总数</span>
          <strong class="metric-value">{{ installMetrics.total }}</strong>
          <small class="metric-note">当前插件安装登记总量</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">工作区安装</span>
          <strong class="metric-value">{{ installMetrics.workspace }}</strong>
          <small class="metric-note">适合本地验证与联调回归</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">高风险安装</span>
          <strong class="metric-value">{{ installMetrics.highRisk }}</strong>
          <small class="metric-note">混合部署、依赖较多或状态异常需复核</small>
        </article>
        <article class="metric-card">
          <span class="metric-label">当前筛选结果</span>
          <strong class="metric-value">{{ filteredRecords.length }}</strong>
          <small class="metric-note">{{ statusText }}</small>
        </article>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <div class="section-head">
          <div>
            <h3 class="title">登记安装</h3>
            <p class="subtitle">上传插件包名称、填写清单摘要，并在登记前查看拆解结果与安装风险。</p>
          </div>
        </div>

        <div class="toolbar-grid">
          <div class="field">
            <label>插件包</label>
            <input :key="fileInputKey" type="file" accept=".zip" @change="onPickFile" />
            <small class="hint">{{ draft.fileName || "请选择 zip 插件包" }}</small>
          </div>
          <div class="field">
            <label>插件类型</label>
            <select v-model="draft.kind">
              <option value="full">full</option>
              <option value="server">server</option>
              <option value="web">web</option>
            </select>
          </div>
          <div class="field">
            <label>安装环境</label>
            <select v-model="draft.target">
              <option value="workspace">workspace</option>
              <option value="docker">docker</option>
              <option value="hybrid">hybrid</option>
            </select>
          </div>
        </div>

        <div class="preset-panel">
          <div class="section-title-row">
            <strong>常用清单模板</strong>
            <span class="hint">用于快速填充清单摘要，再按本次安装调整</span>
          </div>
          <div class="tag-list">
            <button class="tag action-tag" type="button" @click="applyManifestPreset('ops')">
              运维插件
            </button>
            <button class="tag action-tag" type="button" @click="applyManifestPreset('content')">
              内容治理
            </button>
            <button class="tag action-tag" type="button" @click="applyManifestPreset('ui')">
              前端扩展
            </button>
            <button class="tag action-tag" type="button" @click="applyManifestPreset('reset')">
              清空清单
            </button>
          </div>
        </div>

        <div class="field">
          <label>资源摘要</label>
          <textarea
            v-model="draft.manifest"
            rows="7"
            placeholder="填写菜单、接口、字典、依赖、配置或页面说明，支持换行、逗号、分号分隔"
          />
        </div>

        <div class="panel-grid">
          <section class="subcard">
            <div class="section-title-row">
              <strong>清单解析</strong>
              <span class="hint">本地按关键字拆解菜单、接口、字典、依赖与配置</span>
            </div>
            <div class="manifest-groups">
              <div class="manifest-group">
                <span class="detail-label">菜单 / 页面</span>
                <p>{{ draftBreakdown.menus.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">接口</span>
                <p>{{ draftBreakdown.apis.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">字典 / 配置</span>
                <p>{{ draftBreakdown.dictionaries.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">依赖 / 脚本</span>
                <p>{{ draftBreakdown.dependencies.join("、") || "未识别" }}</p>
              </div>
            </div>
          </section>

          <section class="subcard">
            <div class="section-title-row">
              <strong>安装风险提示</strong>
              <span class="hint">登记前先看环境、依赖与范围是否合理</span>
            </div>
            <div class="risk-header">
              <span class="status-pill" :class="riskTone(draftRiskLevel)">
                {{ draftRiskLevel }}风险
              </span>
              <small class="hint">{{ draftAdvice }}</small>
            </div>
            <ul class="risk-list">
              <li v-for="item in draftRiskNotes" :key="item">{{ item }}</li>
            </ul>
          </section>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="submitting || !canInstall" @click="installPlugin">
            {{ submitting ? "登记中..." : "登记安装" }}
          </button>
          <button class="btn ghost" :disabled="!selectedRecord" @click="loadSelectedIntoDraft">
            载入当前记录
          </button>
          <button class="btn ghost" @click="resetDraft">重置草稿</button>
        </div>
      </div>

      <div class="card">
        <div class="section-head">
          <div>
            <h3 class="title">安装台账</h3>
            <p class="subtitle">按插件名、状态、目标环境和类型筛选，并聚焦最近安装与待复核记录。</p>
          </div>
        </div>

        <div class="toolbar-grid">
          <div class="field">
            <label>关键词</label>
            <input
              v-model.trim="filters.keyword"
              placeholder="搜索插件名或摘要"
              @keydown.enter="applyFilters"
            />
          </div>
          <div class="field">
            <label>状态</label>
            <select v-model="filters.status">
              <option value="all">全部状态</option>
              <option v-for="item in statusOptions" :key="item" :value="item">{{ item }}</option>
            </select>
          </div>
          <div class="field">
            <label>环境</label>
            <select v-model="filters.target">
              <option value="all">全部环境</option>
              <option v-for="item in targetOptions" :key="item" :value="item">{{ item }}</option>
            </select>
          </div>
          <div class="field">
            <label>类型</label>
            <select v-model="filters.kind">
              <option value="all">全部类型</option>
              <option v-for="item in kindOptions" :key="item" :value="item">{{ item }}</option>
            </select>
          </div>
        </div>

        <div class="row wrap">
          <button class="btn ghost" @click="applyFilters">应用筛选</button>
          <button class="btn ghost" @click="resetFilters">重置筛选</button>
          <button class="btn ghost" :disabled="!selectedRecord" @click="copyDetailSummary">
            复制安装摘要
          </button>
        </div>

        <div class="panel-grid">
          <section class="subcard">
            <div class="section-title-row">
              <strong>最近安装</strong>
              <span class="hint">优先回看最近登记的插件</span>
            </div>
            <div v-if="recentRecords.length" class="stack-list">
              <button
                v-for="item in recentRecords"
                :key="item.ID"
                class="list-chip"
                type="button"
                @click="selectRecord(item)"
              >
                <span>{{ item.name }}</span>
                <small>{{ item.target }} / {{ item.kind }}</small>
              </button>
            </div>
            <p v-else class="empty-hint">暂无最近安装记录。</p>
          </section>

          <section class="subcard">
            <div class="section-title-row">
              <strong>待复核安装</strong>
              <span class="hint">高风险或混合环境建议优先复核</span>
            </div>
            <div v-if="riskyRecords.length" class="stack-list">
              <button
                v-for="item in riskyRecords"
                :key="item.ID"
                class="list-chip warning"
                type="button"
                @click="selectRecord(item)"
              >
                <span>{{ item.name }}</span>
                <small>{{ summarizeRisk(item) }}</small>
              </button>
            </div>
            <p v-else class="empty-hint">当前没有明显需要升级复核的安装记录。</p>
          </section>
        </div>

        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>插件名</th>
                <th>类型</th>
                <th>环境</th>
                <th>状态</th>
                <th>风险</th>
                <th>时间</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in filteredRecords"
                :key="item.ID"
                :class="{ selected: selectedRecord?.ID === item.ID }"
                @click="selectRecord(item)"
              >
                <td>{{ item.name }}</td>
                <td>{{ item.kind }}</td>
                <td>{{ item.target }}</td>
                <td>
                  <span class="status-pill" :class="statusTone(item.status)">
                    {{ item.status }}
                  </span>
                </td>
                <td>{{ summarizeRisk(item) }}</td>
                <td>{{ formatTime(item.createdAt) }}</td>
              </tr>
              <tr v-if="!filteredRecords.length">
                <td colspan="6">暂无匹配的安装记录，可调整筛选条件或先登记新的插件安装。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="section-head">
        <div>
          <h3 class="title">详情预览</h3>
          <p class="subtitle">查看当前记录的清单拆解、安装建议与复核摘要。</p>
        </div>
      </div>

      <template v-if="selectedRecord">
        <div class="detail-grid">
          <div>
            <span class="detail-label">插件名</span>
            <strong>{{ selectedRecord.name }}</strong>
          </div>
          <div>
            <span class="detail-label">插件类型</span>
            <strong>{{ selectedRecord.kind }}</strong>
          </div>
          <div>
            <span class="detail-label">安装环境</span>
            <strong>{{ selectedRecord.target }}</strong>
          </div>
          <div>
            <span class="detail-label">登记状态</span>
            <strong>{{ selectedRecord.status }}</strong>
          </div>
          <div>
            <span class="detail-label">登记时间</span>
            <strong>{{ formatTime(selectedRecord.createdAt) }}</strong>
          </div>
          <div>
            <span class="detail-label">安装结论</span>
            <strong>{{ selectedAdvice }}</strong>
          </div>
        </div>

        <div class="panel-grid">
          <section class="subcard">
            <div class="section-title-row">
              <strong>清单拆解</strong>
              <span class="hint">按当前登记摘要解析出的组件范围</span>
            </div>
            <div class="manifest-groups">
              <div class="manifest-group">
                <span class="detail-label">菜单 / 页面</span>
                <p>{{ selectedBreakdown.menus.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">接口</span>
                <p>{{ selectedBreakdown.apis.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">字典 / 配置</span>
                <p>{{ selectedBreakdown.dictionaries.join("、") || "未识别" }}</p>
              </div>
              <div class="manifest-group">
                <span class="detail-label">依赖 / 脚本</span>
                <p>{{ selectedBreakdown.dependencies.join("、") || "未识别" }}</p>
              </div>
            </div>
          </section>

          <section class="subcard">
            <div class="section-title-row">
              <strong>风险提示</strong>
              <span class="hint">用于判断是否需要继续解包、注入或人工复核</span>
            </div>
            <div class="risk-header">
              <span class="status-pill" :class="riskTone(selectedRiskLevel)">
                {{ selectedRiskLevel }}风险
              </span>
              <small class="hint">{{ selectedAdvice }}</small>
            </div>
            <ul class="risk-list">
              <li v-for="item in selectedRiskNotes" :key="item">{{ item }}</li>
            </ul>
          </section>
        </div>
      </template>
      <p v-else class="empty-hint">请选择一条安装记录查看详情，或先登记新的插件安装。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getPluginInstallListApi, savePluginInstallApi } from "../../api/admin";
import type { PluginInstallRecord } from "../../types";

type ManifestBreakdown = {
  apis: string[];
  dependencies: string[];
  dictionaries: string[];
  menus: string[];
  others: string[];
};

type RiskLevel = "低" | "中" | "高";

const records = ref<PluginInstallRecord[]>([]);
const selectedRecord = ref<PluginInstallRecord | null>(null);
const loading = ref(false);
const submitting = ref(false);
const statusText = ref("等待加载插件安装台账。");
const fileInputKey = ref(0);
const filters = reactive({
  keyword: "",
  status: "all",
  target: "all",
  kind: "all",
});
const draft = reactive({
  fileName: "",
  kind: "full",
  target: "workspace",
  manifest: "",
});

const statusOptions = computed(() =>
  Array.from(new Set(records.value.map((item) => item.status).filter(Boolean))),
);
const targetOptions = computed(() =>
  Array.from(new Set(records.value.map((item) => item.target).filter(Boolean))),
);
const kindOptions = computed(() =>
  Array.from(new Set(records.value.map((item) => item.kind).filter(Boolean))),
);

const filteredRecords = computed(() => {
  const keyword = filters.keyword.trim().toLowerCase();
  return records.value.filter((item) => {
    if (
      keyword &&
      !`${item.name} ${item.manifest} ${item.kind} ${item.target}`.toLowerCase().includes(keyword)
    ) {
      return false;
    }
    if (filters.status !== "all" && item.status !== filters.status) {
      return false;
    }
    if (filters.target !== "all" && item.target !== filters.target) {
      return false;
    }
    if (filters.kind !== "all" && item.kind !== filters.kind) {
      return false;
    }
    return true;
  });
});

const recentRecords = computed(() => records.value.slice(0, 5));
const riskyRecords = computed(() =>
  records.value.filter((item) => evaluateRisk(item).level === "高").slice(0, 5),
);

const draftBreakdown = computed(() => analyzeManifest(draft.manifest));
const draftRisk = computed(() =>
  evaluateRisk({
    kind: draft.kind,
    manifest: draft.manifest,
    status: draft.fileName ? "已准备" : "待选择插件包",
    target: draft.target,
  }),
);
const draftRiskLevel = computed(() => draftRisk.value.level);
const draftRiskNotes = computed(() => draftRisk.value.notes);
const draftAdvice = computed(() => draftRisk.value.advice);
const canInstall = computed(() => Boolean(draft.fileName.trim()));

const selectedBreakdown = computed(() => analyzeManifest(selectedRecord.value?.manifest ?? ""));
const selectedRisk = computed(() =>
  selectedRecord.value
    ? evaluateRisk(selectedRecord.value)
    : { advice: "", level: "低" as RiskLevel, notes: [] as string[] }
);
const selectedRiskLevel = computed(() => selectedRisk.value.level);
const selectedRiskNotes = computed(() => selectedRisk.value.notes);
const selectedAdvice = computed(() => selectedRisk.value.advice);

const installMetrics = computed(() => ({
  total: records.value.length,
  workspace: records.value.filter((item) => item.target === "workspace").length,
  highRisk: records.value.filter((item) => evaluateRisk(item).level === "高").length,
}));

function splitManifest(text: string) {
  return text
    .split(/[\n,，;；|]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function analyzeManifest(text: string): ManifestBreakdown {
  const entries = splitManifest(text);
  const breakdown: ManifestBreakdown = {
    apis: [],
    dependencies: [],
    dictionaries: [],
    menus: [],
    others: [],
  };

  entries.forEach((entry) => {
    const normalized = entry.toLowerCase();
    if (
      normalized.includes("menu") ||
      normalized.includes("页面") ||
      normalized.includes("路由") ||
      normalized.includes("view")
    ) {
      breakdown.menus.push(entry);
      return;
    }
    if (
      normalized.includes("api") ||
      normalized.includes("接口") ||
      normalized.includes("route") ||
      normalized.includes("endpoint")
    ) {
      breakdown.apis.push(entry);
      return;
    }
    if (
      normalized.includes("dict") ||
      normalized.includes("字典") ||
      normalized.includes("配置") ||
      normalized.includes("config")
    ) {
      breakdown.dictionaries.push(entry);
      return;
    }
    if (
      normalized.includes("依赖") ||
      normalized.includes("script") ||
      normalized.includes("脚本") ||
      normalized.includes("redis") ||
      normalized.includes("sql") ||
      normalized.includes("docker")
    ) {
      breakdown.dependencies.push(entry);
      return;
    }
    breakdown.others.push(entry);
  });

  return breakdown;
}

function evaluateRisk(item: Pick<PluginInstallRecord, "kind" | "manifest" | "status" | "target">) {
  const notes: string[] = [];
  const breakdown = analyzeManifest(item.manifest);
  const dependencyCount = breakdown.dependencies.length;
  const resourceCount =
    breakdown.menus.length +
    breakdown.apis.length +
    breakdown.dictionaries.length +
    breakdown.others.length;

  if (item.target === "hybrid") {
    notes.push("当前为 hybrid 安装，需同时核对本地资源与容器环境挂载。");
  } else if (item.target === "docker") {
    notes.push("当前为 docker 安装，建议确认镜像内依赖与挂载目录已同步。");
  } else {
    notes.push("当前为 workspace 安装，适合先做资源注入与回归验证。");
  }

  if (item.kind === "full") {
    notes.push("full 插件通常同时覆盖前后端，建议复查页面入口与服务注册。");
  }
  if (dependencyCount > 0) {
    notes.push(`检测到 ${dependencyCount} 项依赖/脚本说明，安装前需确认外部服务已就绪。`);
  }
  if (resourceCount === 0) {
    notes.push("摘要未识别到清单项，建议补充菜单、接口、配置等范围说明。");
  }
  if (!item.status.includes("已")) {
    notes.push("当前状态未显示已落账，建议先确认登记流程是否完成。");
  } else {
    notes.push("当前已完成登记，可继续进入解包、注入与启用检查。");
  }

  let level: RiskLevel = "低";
  if (item.target === "hybrid" || dependencyCount >= 2 || resourceCount >= 6) {
    level = "高";
  } else if (item.target === "docker" || item.kind === "full" || dependencyCount === 1 || resourceCount >= 3) {
    level = "中";
  }

  const advice =
    level === "高"
      ? "建议先做安装前置核查，再安排联调与回滚预案。"
      : level === "中"
        ? "建议登记后补做环境核查与重点资源验收。"
        : "当前范围较小，可直接进入本地验证与启用流程。";

  return { advice, level, notes };
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleString("zh-CN");
}

function statusTone(status: string) {
  if (status.includes("失败") || status.includes("异常")) {
    return "tone-danger";
  }
  if (status.includes("登记") || status.includes("处理中")) {
    return "tone-warning";
  }
  return "tone-success";
}

function riskTone(level: RiskLevel) {
  if (level === "高") {
    return "tone-danger";
  }
  if (level === "中") {
    return "tone-warning";
  }
  return "tone-success";
}

function summarizeRisk(record: PluginInstallRecord) {
  const risk = evaluateRisk(record);
  return `${risk.level}风险 / ${risk.advice}`;
}

function buildDetailSummary(record: PluginInstallRecord) {
  const risk = evaluateRisk(record);
  return [
    `插件：${record.name}`,
    `类型：${record.kind}`,
    `环境：${record.target}`,
    `状态：${record.status}`,
    `时间：${formatTime(record.createdAt)}`,
    `结论：${risk.advice}`,
  ].join("\n");
}

async function copyText(text: string, fallbackTitle: string) {
  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return true;
  }
  if (typeof window !== "undefined") {
    window.prompt(fallbackTitle, text);
    return false;
  }
  return false;
}

function selectRecord(record: PluginInstallRecord) {
  selectedRecord.value = record;
  statusText.value = `已选中插件 ${record.name}，可查看清单拆解与安装风险。`;
}

function applyFilters() {
  statusText.value = `筛选命中 ${filteredRecords.value.length} 条安装记录。`;
  if (!filteredRecords.value.length) {
    return;
  }
  if (!selectedRecord.value || !filteredRecords.value.some((item) => item.ID === selectedRecord.value?.ID)) {
    selectedRecord.value = filteredRecords.value[0] ?? null;
  }
}

function resetFilters() {
  filters.keyword = "";
  filters.status = "all";
  filters.target = "all";
  filters.kind = "all";
  statusText.value = "已重置筛选条件。";
  if (records.value[0]) {
    selectedRecord.value = records.value[0];
  }
}

function onPickFile(event: Event) {
  const input = event.target as HTMLInputElement;
  draft.fileName = input.files?.[0]?.name ?? "";
  statusText.value = draft.fileName
    ? `已选择插件包 ${draft.fileName}，可继续完善安装摘要。`
    : "尚未选择插件包。";
}

function applyManifestPreset(kind: "ops" | "content" | "ui" | "reset") {
  if (kind === "reset") {
    draft.manifest = "";
    statusText.value = "已清空安装摘要模板。";
    return;
  }

  if (kind === "ops") {
    draft.kind = "server";
    draft.target = "docker";
    draft.manifest = "菜单: 运维中心, 接口: /tool/plugin-install/save, 配置: 插件启用开关, 依赖: redis, script: 安装后巡检";
    statusText.value = "已套用运维插件模板。";
    return;
  }

  if (kind === "content") {
    draft.kind = "full";
    draft.target = "hybrid";
    draft.manifest = "页面: 内容审核工作台, 接口: /content/review/list, 字典: 审核状态, 依赖: sql, docker: 容器映射";
    statusText.value = "已套用内容治理模板。";
    return;
  }

  draft.kind = "web";
  draft.target = "workspace";
  draft.manifest = "页面: 插件概览页, menu: 扩展中心, config: 前端开关, script: 构建后验证";
  statusText.value = "已套用前端扩展模板。";
}

async function loadRecords() {
  loading.value = true;
  try {
    const result = await getPluginInstallListApi();
    records.value = [...result.List].sort((a, b) => b.ID - a.ID);
    if (!records.value.length) {
      selectedRecord.value = null;
      statusText.value = "当前暂无插件安装记录，可先登记新的插件包。";
      return;
    }
    if (!selectedRecord.value || !records.value.some((item) => item.ID === selectedRecord.value?.ID)) {
      selectedRecord.value = records.value[0];
    } else {
      selectedRecord.value = records.value.find((item) => item.ID === selectedRecord.value?.ID) ?? records.value[0];
    }
    statusText.value = `已加载 ${records.value.length} 条插件安装记录。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "加载插件安装记录失败";
  } finally {
    loading.value = false;
  }
}

async function installPlugin() {
  if (!canInstall.value) {
    statusText.value = "请先选择插件包。";
    return;
  }
  submitting.value = true;
  try {
    const record = await savePluginInstallApi({
      fileName: draft.fileName,
      kind: draft.kind,
      target: draft.target,
      manifest: draft.manifest,
    });
    await loadRecords();
    selectedRecord.value = record;
    statusText.value = `已登记插件 ${record.name}，可继续执行解包、资源注入与启用流程。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "插件安装登记失败";
  } finally {
    submitting.value = false;
  }
}

function loadSelectedIntoDraft() {
  if (!selectedRecord.value) {
    return;
  }
  draft.fileName = `${selectedRecord.value.name}.zip`;
  draft.kind = selectedRecord.value.kind;
  draft.target = selectedRecord.value.target;
  draft.manifest = selectedRecord.value.manifest;
  statusText.value = `已将 ${selectedRecord.value.name} 载入为当前安装草稿。`;
}

function resetDraft() {
  draft.fileName = "";
  draft.kind = "full";
  draft.target = "workspace";
  draft.manifest = "";
  fileInputKey.value += 1;
  statusText.value = "已重置安装草稿。";
}

async function copyDetailSummary() {
  if (!selectedRecord.value) {
    statusText.value = "请先选择一条安装记录。";
    return;
  }
  const copied = await copyText(buildDetailSummary(selectedRecord.value), "复制安装摘要");
  statusText.value = copied
    ? `已复制 ${selectedRecord.value.name} 的安装摘要。`
    : `当前环境不支持自动复制，已提供 ${selectedRecord.value.name} 的手动复制内容。`;
}

onMounted(loadRecords);
</script>

<style scoped>
.section-head {
  align-items: flex-start;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.summary-grid,
.panel-grid,
.detail-grid {
  display: grid;
  gap: 12px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-top: 16px;
}

.metric-card,
.subcard {
  background: color-mix(in srgb, var(--el-bg-color) 92%, var(--el-color-primary) 8%);
  border: 1px solid var(--el-border-color-light);
  border-radius: 12px;
  padding: 14px;
}

.metric-label,
.detail-label,
.hint {
  color: var(--el-text-color-secondary);
}

.metric-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.metric-value {
  font-size: 24px;
  line-height: 1.1;
}

.metric-note {
  color: var(--el-text-color-secondary);
}

.preset-panel,
.manifest-groups,
.stack-list,
.risk-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.preset-panel {
  margin-bottom: 12px;
}

.section-title-row {
  align-items: center;
  display: flex;
  gap: 8px;
  justify-content: space-between;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag {
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color);
  border-radius: 999px;
  color: var(--el-text-color-regular);
  padding: 6px 12px;
}

.action-tag {
  cursor: pointer;
}

.manifest-group p,
.empty-hint {
  color: var(--el-text-color-regular);
  margin: 6px 0 0;
}

.risk-header {
  align-items: center;
  display: flex;
  gap: 8px;
}

.risk-list {
  margin: 0;
  padding-left: 18px;
}

.stack-list {
  max-height: 220px;
  overflow: auto;
}

.list-chip {
  align-items: flex-start;
  background: var(--el-fill-color-blank);
  border: 1px solid var(--el-border-color);
  border-radius: 10px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  text-align: left;
}

.list-chip.warning {
  border-color: color-mix(in srgb, var(--el-color-danger) 35%, var(--el-border-color) 65%);
}

.status-pill {
  border-radius: 999px;
  display: inline-flex;
  padding: 4px 10px;
}

.tone-success {
  background: color-mix(in srgb, var(--el-color-success) 14%, white 86%);
  color: var(--el-color-success);
}

.tone-warning {
  background: color-mix(in srgb, var(--el-color-warning) 18%, white 82%);
  color: var(--el-color-warning-dark-2);
}

.tone-danger {
  background: color-mix(in srgb, var(--el-color-danger) 14%, white 86%);
  color: var(--el-color-danger);
}

.detail-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 12px;
}

.detail-grid > div {
  background: var(--el-fill-color-light);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
}

.selected {
  background: color-mix(in srgb, var(--el-color-primary) 8%, transparent);
}
</style>
