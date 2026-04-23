<template>
  <div class="stack">
    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap">
        <div>
          <h3 class="title">页面设计工作台</h3>
          <p class="subtitle">基于 auto-code 台账保存草图、生成代码、回填最近设计，并在保存前做真实设计校验。</p>
        </div>
        <div class="tag-list">
          <span class="tag">最近草图 {{ records.length }}</span>
          <span class="tag">当前板块 {{ sectionList.length }}</span>
          <span class="tag">风险提示 {{ designRisks.length }}</span>
          <span class="tag">最近生成 {{ lastGeneratedAt ? formatTime(lastGeneratedAt) : "未生成" }}</span>
        </div>
      </div>

      <div class="toolbar-grid">
        <div class="card">
          <h4 class="title">草图摘要</h4>
          <p class="subtitle">{{ draftSummary }}</p>
        </div>
        <div class="card">
          <h4 class="title">配色建议</h4>
          <p class="subtitle">{{ paletteProfile.advice }}</p>
          <div class="tag-list">
            <span class="tag">主色 {{ paletteProfile.primary }}</span>
            <span class="tag">强调 {{ paletteProfile.accent }}</span>
          </div>
        </div>
        <div class="card">
          <h4 class="title">布局建议</h4>
          <p class="subtitle">{{ layoutProfile.advice }}</p>
        </div>
        <div class="card">
          <h4 class="title">生成状态</h4>
          <p class="subtitle">{{ generationBlocked ? "仍有阻塞项，建议先修正后再保存" : "校验可通过，可直接生成并入库" }}</p>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="title">设计草稿</h3>
      <div class="toolbar-grid">
        <div class="field">
          <label>页面用途</label>
          <select v-model="draft.pageType">
            <option>企业官网</option>
            <option>电商页面</option>
            <option>个人博客</option>
            <option>产品介绍</option>
            <option>活动落地页</option>
          </select>
        </div>
        <div class="field">
          <label>目标人群</label>
          <select v-model="draft.audience">
            <option>企业决策者</option>
            <option>运营团队</option>
            <option>开发团队</option>
            <option>终端消费者</option>
            <option>活动访客</option>
          </select>
        </div>
        <div class="field">
          <label>风格偏好</label>
          <select v-model="draft.style">
            <option>简约</option>
            <option>科技感</option>
            <option>专业</option>
            <option>创意</option>
          </select>
        </div>
        <div class="field">
          <label>布局方式</label>
          <select v-model="draft.layout">
            <option>单栏布局</option>
            <option>双栏布局</option>
            <option>卡片式</option>
            <option>响应式</option>
          </select>
        </div>
        <div class="field">
          <label>配色方案</label>
          <select v-model="draft.palette">
            <option>蓝色系</option>
            <option>绿色系</option>
            <option>暖色调</option>
            <option>黑白灰</option>
          </select>
        </div>
        <div class="field">
          <label>主转化动作</label>
          <input v-model="draft.cta" placeholder="例如 预约演示 / 立即咨询 / 立即购买" />
        </div>
      </div>
      <div class="field">
        <label>主要板块</label>
        <input v-model="draft.sections" placeholder="例如 Banner、亮点、案例、FAQ、联系方式" />
      </div>
      <div class="field">
        <label>详细描述</label>
        <textarea v-model="draft.prompt" rows="4" placeholder="描述主目标、核心卖点、转化动作、信息层级" />
      </div>
      <div class="field">
        <label>设计备注</label>
        <textarea v-model="draft.notes" rows="3" placeholder="记录对留白、插图、表单、导航等额外要求" />
      </div>
      <div class="row" style="justify-content: space-between; align-items: flex-start; gap: 16px; flex-wrap: wrap">
        <div class="row" style="gap: 8px; flex-wrap: wrap">
          <button class="btn ghost" @click="handleGeneratePage">生成草图</button>
          <button class="btn ghost" @click="copyCode">复制代码</button>
          <button class="btn primary" @click="saveDraft">写入台账</button>
          <button class="btn ghost" @click="fillLatest">回填最近草图</button>
          <button class="btn ghost" @click="restoreDefaults">恢复默认草稿</button>
        </div>
        <div class="tag-list">
          <span v-for="item in designRecommendations" :key="item" class="tag">{{ item }}</span>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">生成校验</h3>
        <div class="stack">
          <div
            v-for="item in validationItems"
            :key="item.label"
            class="row"
            style="justify-content: space-between; gap: 12px; align-items: center; border-bottom: 1px solid var(--va-border-color, #ebeef5); padding-bottom: 8px"
          >
            <div>
              <strong>{{ item.label }}</strong>
              <p class="subtitle" style="margin: 4px 0 0">{{ item.detail }}</p>
            </div>
            <span class="tag">{{ item.ok ? "通过" : "待修正" }}</span>
          </div>
        </div>
        <div class="card" style="margin-top: 16px">
          <h4 class="title">设计风险提示</h4>
          <div class="tag-list" v-if="designRisks.length">
            <span v-for="risk in designRisks" :key="risk" class="tag">{{ risk }}</span>
          </div>
          <p v-else class="subtitle">当前草图未发现明显风险，可直接进入预览与保存流程。</p>
        </div>
      </div>

      <div class="card">
        <h3 class="title">最近草图详情</h3>
        <div v-if="selectedRecord" class="stack">
          <div class="tag-list">
            <span class="tag">时间 {{ formatTime(selectedRecord.createdAt) }}</span>
            <span class="tag">用途 {{ selectedRecord.payload.pageType }}</span>
            <span class="tag">风格 {{ selectedRecord.payload.style }}</span>
            <span class="tag">布局 {{ selectedRecord.payload.layout }}</span>
          </div>
          <p class="subtitle">{{ selectedRecord.payload.summary || summarizePayload(selectedRecord.payload) }}</p>
          <div class="row" style="gap: 8px; flex-wrap: wrap">
            <button class="btn ghost" @click="hydrate(selectedRecord.payload)">回填当前草稿</button>
            <button class="btn ghost" @click="copyRecordSummary(selectedRecord.payload)">复制摘要</button>
          </div>
          <div class="card">
            <h4 class="title">板块列表</h4>
            <div class="tag-list">
              <span v-for="item in splitSections(selectedRecord.payload.sections)" :key="item" class="tag">{{ item }}</span>
            </div>
          </div>
          <div class="card">
            <h4 class="title">设计备注</h4>
            <p class="subtitle">{{ selectedRecord.payload.notes || "该草图未记录额外备注。" }}</p>
          </div>
        </div>
        <p v-else class="subtitle">暂无可查看的历史草图。</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">页面预览</h3>
        <div class="preview-frame" :style="previewStyle">
          <div style="padding: 24px">
            <div class="row" style="justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap">
              <div>
                <h2 style="margin: 0 0 8px">{{ draft.pageType }}</h2>
                <p style="margin: 0; opacity: 0.82">{{ draft.prompt }}</p>
              </div>
              <div class="tag-list">
                <span class="tag">{{ draft.style }}</span>
                <span class="tag">{{ draft.layout }}</span>
                <span class="tag">{{ draft.audience }}</span>
              </div>
            </div>
            <div class="card" style="margin-top: 16px; background: rgba(255, 255, 255, 0.12)">
              <h4 class="title" style="color: inherit">设计落点</h4>
              <p class="subtitle" style="color: inherit">{{ draftSummary }}</p>
              <div class="tag-list">
                <span v-for="section in sectionList" :key="section" class="tag">{{ section }}</span>
              </div>
            </div>
            <div class="card" style="margin-top: 16px; background: rgba(255, 255, 255, 0.12)">
              <h4 class="title" style="color: inherit">建议 CTA</h4>
              <p class="subtitle" style="color: inherit">{{ draft.cta || "请补充主转化动作" }}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="title">源代码</h3>
        <pre class="code-block">{{ snippet }}</pre>
      </div>
    </div>

    <div class="card">
      <div class="row" style="justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap">
        <h3 class="title" style="margin: 0">最近草图</h3>
        <div class="row" style="gap: 8px; flex-wrap: wrap">
          <input v-model="recordQuery" placeholder="按用途 / 风格 / 布局 / 板块筛选" />
        </div>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>时间</th>
              <th>用途</th>
              <th>风格</th>
              <th>布局</th>
              <th>板块数</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in filteredRecords"
              :key="item.ID"
            >
              <td>{{ formatTime(item.createdAt) }}</td>
              <td>{{ item.payload.pageType }}</td>
              <td>{{ item.payload.style }}</td>
              <td>{{ item.payload.layout }}</td>
              <td>{{ splitSections(item.payload.sections).length }}</td>
              <td>
                <div class="row" style="gap: 8px; flex-wrap: wrap">
                  <button class="btn ghost" @click="selectRecord(item.ID)">详情</button>
                  <button class="btn ghost" @click="hydrate(item.payload)">回填</button>
                </div>
              </td>
            </tr>
            <tr v-if="!filteredRecords.length">
              <td colspan="6">暂无匹配的草图记录</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getAutoCodeRegistryApi, saveAutoCodeRegistryApi } from "../../api/admin";
import type { AutoCodeRegistryRecord } from "../../types";

type PictureDraftPayload = {
  kind: "page-draft";
  pageType: string;
  audience: string;
  style: string;
  layout: string;
  palette: string;
  sections: string;
  prompt: string;
  cta: string;
  notes: string;
  summary: string;
  snippet: string;
};

type ValidationItem = {
  label: string;
  detail: string;
  ok: boolean;
};

type DraftSummaryInput = {
  pageType: string;
  audience: string;
  style: string;
  layout: string;
  cta: string;
};

const defaultDraft = {
  pageType: "企业官网",
  audience: "企业决策者",
  style: "科技感",
  layout: "响应式",
  palette: "蓝色系",
  sections: "Banner、产品能力、客户案例、FAQ、联系表单",
  prompt: "首页突出产品能力、客户成果与预约演示入口，首屏明确价值主张与行动按钮。",
  cta: "预约演示",
  notes: "首屏保留客户徽标区，案例区突出行业结果，FAQ 后接联系表单。",
};

const draft = reactive({ ...defaultDraft });
const snippet = ref("");
const message = ref("");
const recordQuery = ref("");
const lastGeneratedAt = ref<number | null>(null);
const selectedRecordId = ref<number | null>(null);
const records = ref<Array<AutoCodeRegistryRecord & { payload: PictureDraftPayload }>>([]);

const paletteCatalog: Record<string, { background: string; primary: string; accent: string; advice: string }> = {
  蓝色系: {
    background: "linear-gradient(135deg, #0f172a, #1d4ed8)",
    primary: "#1d4ed8",
    accent: "#93c5fd",
    advice: "适合强调专业、稳定与企业可信度，适合搭配数据、案例与产品亮点。",
  },
  绿色系: {
    background: "linear-gradient(135deg, #052e16, #16a34a)",
    primary: "#16a34a",
    accent: "#86efac",
    advice: "适合效率、增长、流程与可持续主题，建议搭配操作步骤和成效数据。",
  },
  暖色调: {
    background: "linear-gradient(135deg, #7c2d12, #fb923c)",
    primary: "#f97316",
    accent: "#fdba74",
    advice: "适合活动页和强转化场景，但要控制大面积高饱和背景避免阅读疲劳。",
  },
  黑白灰: {
    background: "linear-gradient(135deg, #111827, #6b7280)",
    primary: "#111827",
    accent: "#d1d5db",
    advice: "适合品牌展示和作品集，建议用明确强调色引导 CTA，避免信息层级过平。",
  },
};

const layoutCatalog: Record<string, { advice: string }> = {
  单栏布局: { advice: "适合叙事型页面与清晰信息流，但板块过多时要加强锚点与节奏。" },
  双栏布局: { advice: "适合卖点与表单并置、说明与案例对照，需保证左右信息密度平衡。" },
  卡片式: { advice: "适合模块化功能展示与多内容并列，建议统一卡片高度和层级。" },
  响应式: { advice: "适合通用业务页和多终端访问，需优先保证移动端 CTA 与表单可见。" },
};

function splitSections(value: string) {
  const unique = new Set(
    value
      .split(/[、,，\n]/)
      .map((item) => item.trim())
      .filter(Boolean),
  );
  return [...unique];
}

function summarizePayload(payload: DraftSummaryInput) {
  return `${payload.pageType}面向${payload.audience}，采用${payload.style}风格与${payload.layout}结构，主转化动作是“${payload.cta || "待补充"}”。`;
}

const sectionList = computed(() => splitSections(draft.sections));
const paletteProfile = computed(() => paletteCatalog[draft.palette] ?? paletteCatalog["蓝色系"]);
const layoutProfile = computed(() => layoutCatalog[draft.layout] ?? layoutCatalog["响应式"]);

const draftSummary = computed(() => summarizePayload(draft));

const validationItems = computed<ValidationItem[]>(() => [
  {
    label: "页面用途",
    detail: draft.pageType.trim().length >= 2 ? `当前用途：${draft.pageType}` : "页面用途至少填写 2 个字。",
    ok: draft.pageType.trim().length >= 2,
  },
  {
    label: "板块完整度",
    detail:
      sectionList.value.length >= 3
        ? `已配置 ${sectionList.value.length} 个板块。`
        : "建议至少提供 3 个板块，避免生成结果过于空泛。",
    ok: sectionList.value.length >= 3,
  },
  {
    label: "描述清晰度",
    detail:
      draft.prompt.trim().length >= 16
        ? "详细描述长度充足，可支持生成页面骨架。"
        : "详细描述建议不少于 16 个字，并说明目标与层级。",
    ok: draft.prompt.trim().length >= 16,
  },
  {
    label: "转化动作",
    detail: draft.cta.trim() ? `当前主 CTA：${draft.cta}` : "请填写主转化动作，方便生成首屏按钮与表单文案。",
    ok: draft.cta.trim().length > 0,
  },
  {
    label: "布局适配",
    detail:
      draft.layout !== "双栏布局" || sectionList.value.length >= 4
        ? "当前布局与内容量基本匹配。"
        : "双栏布局建议至少 4 个板块，避免左右信息过少。",
    ok: draft.layout !== "双栏布局" || sectionList.value.length >= 4,
  },
]);

const generationBlocked = computed(() => validationItems.value.some((item) => !item.ok));

const designRisks = computed(() => {
  const risks: string[] = [];
  if (draft.layout === "单栏布局" && sectionList.value.length >= 6) {
    risks.push("单栏板块较多，首屏到页尾可能过长");
  }
  if (draft.palette === "暖色调" && draft.style === "专业") {
    risks.push("专业风格搭配暖色调时需控制饱和度");
  }
  if (!/(联系|咨询|购买|报名|试用|预约)/.test(`${draft.prompt}${draft.cta}`)) {
    risks.push("描述里缺少明确转化动作");
  }
  if ((draft.pageType === "企业官网" || draft.pageType === "产品介绍") && !/(案例|FAQ|联系)/.test(draft.sections)) {
    risks.push("企业/产品页建议补充案例、FAQ 或联系板块");
  }
  if (draft.palette === "黑白灰" && draft.style === "创意") {
    risks.push("创意风格使用黑白灰时需额外强调视觉焦点");
  }
  return risks;
});

const designRecommendations = computed(() => {
  const suggestions = [
    `首屏文案面向${draft.audience}`,
    `强调色建议使用 ${paletteProfile.value.accent}`,
    layoutProfile.value.advice,
  ];
  if (!sectionList.value.includes("FAQ")) {
    suggestions.push("可补充 FAQ 降低转化阻力");
  }
  if (!sectionList.value.some((item) => /案例|客户|评价/.test(item))) {
    suggestions.push("建议增加案例/评价增强可信度");
  }
  return suggestions.slice(0, 4);
});

const previewStyle = computed(() => ({
  background: paletteProfile.value.background,
  color: "white",
  minHeight: "360px",
}));

const filteredRecords = computed(() => {
  const keyword = recordQuery.value.trim().toLowerCase();
  if (!keyword) {
    return records.value;
  }
  return records.value.filter((item) =>
    [item.payload.pageType, item.payload.style, item.payload.layout, item.payload.sections, item.payload.audience]
      .join(" ")
      .toLowerCase()
      .includes(keyword),
  );
});

const selectedRecord = computed(() =>
  records.value.find((item) => item.ID === selectedRecordId.value) ?? records.value[0] ?? null,
);

function buildSnippet() {
  const layoutClassMap: Record<string, string> = {
    单栏布局: "page-shell single-column",
    双栏布局: "page-shell two-column",
    卡片式: "page-shell card-grid",
    响应式: "page-shell responsive-shell",
  };
  const sectionBlocks = sectionList.value
    .map(
      (item, index) => `    <section class="content-block section-${index + 1}">
      <h2>${item}</h2>
      <p>${draft.pageType} - ${item} 面向 ${draft.audience} 展示重点内容。</p>
    </section>`,
    )
    .join("\n");

  return `<template>
  <main class="${layoutClassMap[draft.layout] ?? layoutClassMap["响应式"]}">
    <section class="hero">
      <span class="hero-kicker">${draft.pageType}</span>
      <h1>${draft.prompt}</h1>
      <p>${draftSummary.value}</p>
      <button class="primary-action">${draft.cta || "立即行动"}</button>
    </section>
${sectionBlocks}
  </main>
</template>

<style scoped>
.page-shell {
  min-height: 100vh;
  padding: 48px 24px;
  background: ${paletteProfile.value.background};
  color: white;
}
.two-column .content-block {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.card-grid .content-block {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 20px;
}
.primary-action {
  margin-top: 20px;
  padding: 10px 20px;
  border: none;
  border-radius: 999px;
  background: ${paletteProfile.value.accent};
  color: #0f172a;
}
</style>`;
}

async function copyText(value: string, fallbackLabel: string) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value);
      message.value = `${fallbackLabel}已复制到剪贴板`;
      return;
    }
  } catch {
    // continue with fallback
  }
  window.prompt(`当前环境无法直接写入剪贴板，请手动复制${fallbackLabel}`, value);
  message.value = `${fallbackLabel}已提供手动复制降级方案`;
}

function generatePage(showMessage = true) {
  if (generationBlocked.value) {
    message.value = "仍有未通过的生成校验，请先修正后再生成";
    return;
  }
  snippet.value = buildSnippet();
  lastGeneratedAt.value = Date.now();
  if (showMessage) {
    message.value = `${draft.pageType} 草图已生成，可继续复制代码或写入台账`;
  }
}

function handleGeneratePage() {
  generatePage(true);
}

async function loadRecords() {
  const result = await getAutoCodeRegistryApi();
  records.value = result.List.filter(
    (item): item is AutoCodeRegistryRecord & { payload: PictureDraftPayload } =>
      item.payload?.kind === "page-draft",
  ).sort((left, right) => right.createdAt - left.createdAt);
  if (!selectedRecordId.value && records.value.length > 0) {
    selectedRecordId.value = records.value[0].ID;
  }
}

async function saveDraft() {
  generatePage(false);
  if (generationBlocked.value || !snippet.value) {
    message.value = "保存前请先完成生成校验";
    return;
  }
  await saveAutoCodeRegistryApi({
    kind: "page-draft",
    pageType: draft.pageType.trim(),
    audience: draft.audience,
    style: draft.style,
    layout: draft.layout,
    palette: draft.palette,
    sections: draft.sections.trim(),
    prompt: draft.prompt.trim(),
    cta: draft.cta.trim(),
    notes: draft.notes.trim(),
    summary: draftSummary.value,
    snippet: snippet.value,
  });
  message.value = `${draft.pageType} 草图已写入台账`;
  await loadRecords();
}

function hydrate(payload: Partial<PictureDraftPayload>) {
  draft.pageType = payload.pageType || defaultDraft.pageType;
  draft.audience = payload.audience || defaultDraft.audience;
  draft.style = payload.style || defaultDraft.style;
  draft.layout = payload.layout || defaultDraft.layout;
  draft.palette = payload.palette || defaultDraft.palette;
  draft.sections = payload.sections || defaultDraft.sections;
  draft.prompt = payload.prompt || defaultDraft.prompt;
  draft.cta = payload.cta || defaultDraft.cta;
  draft.notes = payload.notes || "";
  snippet.value = payload.snippet || buildSnippet();
  message.value = `${draft.pageType} 草图已回填`;
}

function restoreDefaults() {
  hydrate(defaultDraft);
  generatePage(false);
  message.value = "已恢复默认设计草稿";
}

function fillLatest() {
  const latest = records.value[0];
  if (!latest) {
    message.value = "暂无历史草图可回填";
    return;
  }
  selectedRecordId.value = latest.ID;
  hydrate(latest.payload);
}

function selectRecord(id: number) {
  selectedRecordId.value = id;
}

async function copyCode() {
  if (!snippet.value) {
    generatePage(false);
  }
  if (!snippet.value) {
    return;
  }
  await copyText(snippet.value, "代码");
}

async function copyRecordSummary(payload: PictureDraftPayload) {
  await copyText(payload.summary || summarizePayload(payload), "草图摘要");
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(async () => {
  generatePage(false);
  await loadRecords();
});
</script>
