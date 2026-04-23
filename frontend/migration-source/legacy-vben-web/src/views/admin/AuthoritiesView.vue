<template>
  <div class="page-shell">
    <section class="card">
      <div class="hero-row">
        <div>
          <h3 class="title">角色工作台</h3>
          <p class="subtitle">
            聚合查看角色层级、默认入口、风险提示与子角色分布，并支持直接维护角色草稿。
          </p>
        </div>
        <div class="hero-actions">
          <button class="btn primary" :disabled="loading" @click="loadAuthorities">
            {{ loading ? "刷新中..." : "刷新角色" }}
          </button>
          <button class="btn ghost" :disabled="!rootRoles.length" @click="selectRoot">
            查看根角色
          </button>
          <button class="btn ghost" :disabled="!selectedRole" @click="openCreateChild">
            新增子角色
          </button>
          <button class="btn ghost" @click="openCreate">
            新增角色
          </button>
          <button class="btn ghost" :disabled="!selectedRole" @click="copySelectedSummary">
            复制摘要
          </button>
        </div>
      </div>

      <div v-if="message" class="feedback-banner">{{ message }}</div>

      <div class="summary-grid">
        <article class="summary-card">
          <span class="summary-label">角色总数</span>
          <strong class="summary-value">{{ dashboard.total }}</strong>
          <span class="summary-note">含根角色 {{ dashboard.rootCount }} 个</span>
        </article>
        <article class="summary-card success">
          <span class="summary-label">层级深度</span>
          <strong class="summary-value">{{ dashboard.maxLevel }}</strong>
          <span class="summary-note">叶子角色 {{ dashboard.leafCount }} 个</span>
        </article>
        <article class="summary-card warning">
          <span class="summary-label">入口待复核</span>
          <strong class="summary-value">{{ dashboard.routeReviewCount }}</strong>
          <span class="summary-note">缺省或重复入口需确认</span>
        </article>
        <article class="summary-card danger">
          <span class="summary-label">高风险角色</span>
          <strong class="summary-value">{{ dashboard.highRiskCount }}</strong>
          <span class="summary-note">建议优先处理角色配置</span>
        </article>
      </div>

      <div class="toolbar-grid">
        <div class="field">
          <label>关键词</label>
          <input
            v-model.trim="filters.keyword"
            placeholder="按角色ID / 名称 / 路由搜索"
            @keyup.enter="applyFilters"
          />
        </div>
        <div class="field">
          <label>层级视图</label>
          <select v-model="filters.scope">
            <option value="">全部</option>
            <option value="root">仅根角色</option>
            <option value="branch">有子角色</option>
            <option value="leaf">叶子角色</option>
          </select>
        </div>
        <div class="field">
          <label>入口状态</label>
          <select v-model="filters.routeState">
            <option value="">全部</option>
            <option value="stable">稳定</option>
            <option value="review">待复核</option>
            <option value="missing">缺失</option>
          </select>
        </div>
        <div class="field">
          <label>风险等级</label>
          <select v-model="filters.risk">
            <option value="">全部</option>
            <option value="high">高风险</option>
            <option value="medium">中风险</option>
            <option value="low">低风险</option>
          </select>
        </div>
      </div>

      <div class="row wrap">
        <button class="btn primary" @click="applyFilters">应用筛选</button>
        <button class="btn ghost" @click="clearFilters">清空筛选</button>
        <button class="btn ghost" :disabled="!highestRiskRole" @click="focusHighRisk">
          聚焦高风险角色
        </button>
        <span class="hint">当前命中 {{ filteredRows.length }} / {{ workspaceRows.length }}</span>
      </div>

      <div class="spotlight-grid">
        <article class="spotlight-card">
          <div class="spotlight-header">
            <h4>根角色视图</h4>
            <span>{{ rootRoles.length }} 个根角色</span>
          </div>
          <div v-if="rootRoles.length" class="tag-list">
            <button
              v-for="item in rootRoles"
              :key="item.authorityId"
              class="tag-button"
              :class="{ active: selectedRole?.authorityId === item.authorityId }"
              @click="selectRole(item)"
            >
              {{ item.authorityName }} · {{ item.descendantCount }} 个下级
            </button>
          </div>
          <p v-else class="empty-text">当前没有根角色数据。</p>
        </article>

        <article class="spotlight-card">
          <div class="spotlight-header">
            <h4>默认入口分析</h4>
            <span>{{ duplicatedRoutes.length }} 个重复入口</span>
          </div>
          <ul v-if="routeInsights.length" class="rank-list">
            <li v-for="item in routeInsights" :key="item.label" class="rank-item">
              <span>{{ item.label }}</span>
              <span class="rank-meta">{{ item.value }}</span>
            </li>
          </ul>
          <p v-else class="empty-text">当前没有可分析的入口数据。</p>
        </article>
      </div>
    </section>

    <section class="workspace-grid">
      <article class="card">
        <div class="table-header">
          <div>
            <h3 class="title small">角色台账</h3>
            <p class="subtitle small">点击行查看角色详情、风险说明与子角色概览。</p>
          </div>
          <span class="hint">已选 {{ selectedRole?.authorityName || "未选择" }}</span>
        </div>

        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>角色ID</th>
                <th>角色名称</th>
                <th>层级</th>
                <th>默认路由</th>
                <th>父角色</th>
                <th>子角色</th>
                <th>风险</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody v-if="filteredRows.length">
              <tr
                v-for="item in filteredRows"
                :key="item.authorityId"
                :class="{
                  selected: selectedRole?.authorityId === item.authorityId,
                  warning: item.riskLevel === 'medium',
                  danger: item.riskLevel === 'high',
                }"
                @click="selectRole(item)"
              >
                <td>{{ item.authorityId }}</td>
                <td>{{ item.authorityName }}</td>
                <td>{{ item.hierarchyLabel }}</td>
                <td>{{ item.defaultRouter || "-" }}</td>
                <td>{{ item.parentName }}</td>
                <td>{{ item.childCount }}</td>
                <td>
                  <span :class="['badge', badgeTone(item.riskLevel)]">
                    {{ resolveRiskLabel(item.riskLevel) }}
                  </span>
                </td>
                <td>
                  <div class="inline-actions">
                    <button class="btn ghost small" @click.stop="selectRole(item)">详情</button>
                    <button class="btn ghost small" @click.stop="copyRoleSummary(item)">
                      复制
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
            <tbody v-else>
              <tr>
                <td colspan="8" class="empty-cell">当前筛选条件下没有命中的角色。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </article>

      <article class="card detail-card">
        <div class="table-header">
          <div>
            <h3 class="title small">{{ form.ID ? "编辑角色" : "新增角色" }}</h3>
            <p class="subtitle small">
              支持查看角色摘要、风险提示、子角色结构，并在当前面板内直接保存。
            </p>
          </div>
          <button
            class="btn ghost small"
            :disabled="!selectedRole || !hasFormChanges"
            @click="resetFormToSelected"
          >
            重置草稿
          </button>
        </div>

        <div class="toolbar-grid">
          <div class="field">
            <label>角色ID</label>
            <input v-model.number="form.authorityId" type="number" min="1" />
          </div>
          <div class="field">
            <label>角色名称</label>
            <input v-model.trim="form.authorityName" placeholder="请输入角色名称" />
          </div>
          <div class="field">
            <label>默认路由</label>
            <input v-model.trim="form.defaultRouter" placeholder="例如 dashboard / systemTools" />
          </div>
          <div class="field">
            <label>父角色</label>
            <select v-model.number="form.parentId">
              <option :value="0">根节点</option>
              <option
                v-for="item in parentOptions"
                :key="item.authorityId"
                :value="item.authorityId"
              >
                {{ item.authorityName }}（{{ item.hierarchyLabel }}）
              </option>
            </select>
          </div>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="loading" @click="save">
            {{ loading ? "保存中..." : "保存角色" }}
          </button>
          <button class="btn ghost" :disabled="!selectedRole" @click="loadSelectedIntoForm">
            载入当前详情
          </button>
          <span class="hint">变更字段 {{ changedFields.length }} 项</span>
        </div>

        <div class="detail-grid">
          <div class="detail-item">
            <span>当前角色</span>
            <strong>{{ selectedRole?.authorityName || "新建角色" }}</strong>
          </div>
          <div class="detail-item">
            <span>层级位置</span>
            <strong>{{ selectedRole?.hierarchyLabel || previewHierarchyLabel }}</strong>
          </div>
          <div class="detail-item">
            <span>默认入口状态</span>
            <strong>{{ routePreviewLabel }}</strong>
          </div>
          <div class="detail-item">
            <span>风险判断</span>
            <strong>{{ resolveRiskLabel(previewRiskLevel) }}</strong>
          </div>
        </div>

        <div class="detail-note">
          <h4>角色摘要</h4>
          <ul>
            <li>{{ roleSummaryLine }}</li>
            <li>默认入口分析：{{ routePreviewNote }}</li>
            <li>父子关系：{{ relationSummary }}</li>
          </ul>
        </div>

        <div class="detail-note">
          <h4>风险提示</h4>
          <ul v-if="previewRiskNotes.length">
            <li v-for="item in previewRiskNotes" :key="item">{{ item }}</li>
          </ul>
          <p v-else class="empty-text">当前角色配置未发现明显风险，可直接保存或继续补充下级角色。</p>
        </div>

        <div class="detail-note">
          <h4>子角色概览</h4>
          <div v-if="selectedRole?.children.length" class="tag-list">
            <button
              v-for="item in selectedRole.children"
              :key="item.authorityId"
              class="tag-button"
              @click="selectRoleById(item.authorityId)"
            >
              {{ item.authorityName }} · {{ item.defaultRouter || "待补入口" }}
            </button>
          </div>
          <p v-else class="empty-text">当前角色暂无直接子角色。</p>
        </div>

        <div class="detail-note">
          <h4>草稿差异</h4>
          <ul v-if="changedFields.length">
            <li v-for="item in changedFields" :key="item">{{ item }}</li>
          </ul>
          <p v-else class="empty-text">当前草稿与已选角色一致。</p>
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getAuthorityListApi, saveAuthorityApi } from "../../api/admin";
import type { AuthorityInfo } from "../../types";

type ScopeFilter = "" | "root" | "branch" | "leaf";
type RouteState = "stable" | "review" | "missing";
type RouteFilter = "" | RouteState;
type RiskLevel = "low" | "medium" | "high";
type RiskFilter = "" | RiskLevel;

interface AuthorityWorkspaceRow extends AuthorityInfo {
  childCount: number;
  descendantCount: number;
  hierarchyLabel: string;
  level: number;
  parentName: string;
  riskLevel: RiskLevel;
  riskNotes: string[];
  routeState: RouteState;
}

interface InsightRow {
  label: string;
  value: string;
}

const rows = ref<AuthorityInfo[]>([]);
const loading = ref(false);
const message = ref("");
const selectedRole = ref<AuthorityWorkspaceRow | null>(null);

const form = reactive({
  ID: undefined as number | undefined,
  authorityId: 1000,
  authorityName: "",
  defaultRouter: "dashboard",
  parentId: 0,
});

const filters = reactive<{
  keyword: string;
  risk: RiskFilter;
  routeState: RouteFilter;
  scope: ScopeFilter;
}>({
  keyword: "",
  risk: "",
  routeState: "",
  scope: "",
});

const workspaceRows = computed(() => {
  const flat = flattenAuthorities(rows.value);
  const routeCounts = new Map<string, number>();
  const authorityIdCounts = new Map<number, number>();

  for (const item of flat) {
    const routeKey = item.defaultRouter.trim();
    if (routeKey) {
      routeCounts.set(routeKey, (routeCounts.get(routeKey) ?? 0) + 1);
    }
    authorityIdCounts.set(item.authorityId, (authorityIdCounts.get(item.authorityId) ?? 0) + 1);
  }

  return flat.map((item) => {
    const routeKey = item.defaultRouter.trim();
    const riskNotes: string[] = [];
    let routeState: RouteState = "stable";

    if (!routeKey) {
      routeState = "missing";
      riskNotes.push("默认路由为空，登录后可能无法准确落位。");
    } else if ((routeCounts.get(routeKey) ?? 0) > 1) {
      routeState = "review";
      riskNotes.push("默认路由与其他角色重复，需确认是否有意共享入口。");
    }

    if ((authorityIdCounts.get(item.authorityId) ?? 0) > 1) {
      riskNotes.push("角色ID重复，建议调整为唯一标识。");
    }
    if (item.parentId === item.authorityId && item.parentId !== 0) {
      riskNotes.push("父角色指向自身，层级结构需修正。");
    }
    if (item.level >= 3) {
      riskNotes.push("角色层级较深，建议确认授权链路是否仍然清晰。");
    }
    if (item.childCount === 0 && !routeKey) {
      riskNotes.push("既无子角色又无默认入口，角色职责可能不完整。");
    }

    let riskLevel: RiskLevel = "low";
    if (riskNotes.length >= 3 || routeState === "missing") {
      riskLevel = "high";
    } else if (riskNotes.length >= 1 || routeState === "review") {
      riskLevel = "medium";
    }

    return {
      ...item,
      riskLevel,
      riskNotes,
      routeState,
    };
  });
});

const rootRoles = computed(() => workspaceRows.value.filter((item) => item.parentId === 0));

const parentOptions = computed(() =>
  workspaceRows.value.filter((item) => item.authorityId !== form.authorityId),
);

const duplicatedRoutes = computed(() => {
  const counts = new Map<string, number>();
  for (const item of workspaceRows.value) {
    const route = item.defaultRouter.trim();
    if (!route) continue;
    counts.set(route, (counts.get(route) ?? 0) + 1);
  }
  return [...counts.entries()]
    .filter(([, count]) => count > 1)
    .map(([route, count]) => ({ count, route }))
    .sort((left, right) => right.count - left.count);
});

const dashboard = computed(() => {
  const total = workspaceRows.value.length;
  const rootCount = rootRoles.value.length;
  const leafCount = workspaceRows.value.filter((item) => item.childCount === 0).length;
  const maxLevel = total
    ? Math.max(...workspaceRows.value.map((item) => item.level)) + 1
    : 0;
  const routeReviewCount = workspaceRows.value.filter(
    (item) => item.routeState === "missing" || item.routeState === "review",
  ).length;
  const highRiskCount = workspaceRows.value.filter((item) => item.riskLevel === "high").length;

  return {
    highRiskCount,
    leafCount,
    maxLevel,
    rootCount,
    routeReviewCount,
    total,
  };
});

const routeInsights = computed<InsightRow[]>(() => {
  const missing = workspaceRows.value.filter((item) => item.routeState === "missing").length;
  const review = workspaceRows.value.filter((item) => item.routeState === "review").length;
  const stable = workspaceRows.value.filter((item) => item.routeState === "stable").length;
  const duplicateText = duplicatedRoutes.value.length
    ? duplicatedRoutes.value
        .slice(0, 3)
        .map((item) => `${item.route} ×${item.count}`)
        .join("，")
    : "暂无重复入口";

  return [
    { label: "稳定入口", value: `${stable} 个` },
    { label: "待复核入口", value: `${review} 个` },
    { label: "缺失入口", value: `${missing} 个` },
    { label: "重复入口", value: duplicateText },
  ];
});

const filteredRows = ref<AuthorityWorkspaceRow[]>([]);

const highestRiskRole = computed(
  () => workspaceRows.value.find((item) => item.riskLevel === "high") ?? null,
);

const selectedParent = computed(
  () => workspaceRows.value.find((item) => item.authorityId === form.parentId) ?? null,
);

const hasFormChanges = computed(() => changedFields.value.length > 0);

const changedFields = computed(() => {
  if (!selectedRole.value && form.ID === undefined) {
    const draftNotes: string[] = [];
    if (form.authorityName.trim()) {
      draftNotes.push(`角色名称：${form.authorityName.trim()}`);
    }
    if (form.defaultRouter.trim() !== "dashboard") {
      draftNotes.push(`默认路由：${form.defaultRouter.trim() || "未填写"}`);
    }
    if (form.parentId !== 0) {
      draftNotes.push(`父角色：${selectedParent.value?.authorityName || form.parentId}`);
    }
    return draftNotes;
  }

  const baseline = selectedRole.value
    ? {
        ID: selectedRole.value.ID,
        authorityId: selectedRole.value.authorityId,
        authorityName: selectedRole.value.authorityName,
        defaultRouter: selectedRole.value.defaultRouter,
        parentId: selectedRole.value.parentId,
      }
    : {
        ID: undefined,
        authorityId: undefined,
        authorityName: "",
        defaultRouter: "dashboard",
        parentId: 0,
      };

  const result: string[] = [];
  if (baseline.authorityId !== form.authorityId) {
    result.push(`角色ID：${baseline.authorityId ?? "-"} → ${form.authorityId}`);
  }
  if (baseline.authorityName !== form.authorityName) {
    result.push(`角色名称：${baseline.authorityName || "-"} → ${form.authorityName || "-"}`);
  }
  if (baseline.defaultRouter !== form.defaultRouter) {
    result.push(`默认路由：${baseline.defaultRouter || "-"} → ${form.defaultRouter || "-"}`);
  }
  if (baseline.parentId !== form.parentId) {
    result.push(
      `父角色：${resolveAuthorityName(baseline.parentId)} → ${resolveAuthorityName(form.parentId)}`,
    );
  }
  return result;
});

const previewRouteState = computed<RouteState>(() => {
  const route = form.defaultRouter.trim();
  if (!route) {
    return "missing";
  }
  const duplicateCount = workspaceRows.value.filter(
    (item) => item.defaultRouter.trim() === route && item.authorityId !== selectedRole.value?.authorityId,
  ).length;
  return duplicateCount > 0 ? "review" : "stable";
});

const previewRiskNotes = computed(() => {
  const notes: string[] = [];
  if (!form.authorityName.trim()) {
    notes.push("角色名称为空，建议填写可辨识的职责名称。");
  }
  if (form.authorityId <= 0) {
    notes.push("角色ID需为正整数。");
  }
  if (previewRouteState.value === "missing") {
    notes.push("默认路由为空，角色登录后可能无法落到稳定入口。");
  }
  if (previewRouteState.value === "review") {
    notes.push("默认路由与其他角色重复，建议确认入口共享是否符合预期。");
  }
  if (form.parentId === form.authorityId && form.parentId !== 0) {
    notes.push("父角色不能指向自身。");
  }
  const parentLevel = selectedParent.value?.level ?? -1;
  if (parentLevel >= 2) {
    notes.push("当前父角色层级较深，建议确认是否仍需继续下沉。");
  }
  return notes;
});

const previewRiskLevel = computed<RiskLevel>(() => {
  if (previewRiskNotes.value.length >= 3 || previewRouteState.value === "missing") {
    return "high";
  }
  if (previewRiskNotes.value.length > 0 || previewRouteState.value === "review") {
    return "medium";
  }
  return "low";
});

const previewHierarchyLabel = computed(() => {
  const parent = selectedParent.value;
  const level = parent ? parent.level + 1 : 0;
  return `${level + 1} 级角色`;
});

const routePreviewLabel = computed(() => {
  switch (previewRouteState.value) {
    case "missing":
      return "缺失默认入口";
    case "review":
      return "默认入口待复核";
    default:
      return "默认入口稳定";
  }
});

const routePreviewNote = computed(() => {
  switch (previewRouteState.value) {
    case "missing":
      return "当前未配置默认路由，建议补齐常用入口。";
    case "review":
      return "当前路由与其他角色重叠，建议核查角色分流是否清晰。";
    default:
      return "当前默认路由可单独定位该角色的进入页面。";
  }
});

const relationSummary = computed(() => {
  const parentName = resolveAuthorityName(form.parentId);
  if (form.parentId === 0) {
    return "当前草稿将作为根角色直接挂载。";
  }
  return `当前草稿将挂在父角色 ${parentName} 下。`;
});

const roleSummaryLine = computed(() => {
  return `角色 ${form.authorityName.trim() || "未命名角色"}（#${form.authorityId || "-"}) 的预期入口为 ${
    form.defaultRouter.trim() || "待补充"
  }。`;
});

function flattenAuthorities(
  items: AuthorityInfo[],
  level = 0,
  parentName = "根节点",
): AuthorityWorkspaceRow[] {
  return items.flatMap((item) => {
    const current: AuthorityWorkspaceRow = {
      ...item,
      childCount: item.children.length,
      descendantCount: countDescendants(item),
      hierarchyLabel: `${level + 1} 级角色`,
      level,
      parentName,
      riskLevel: "low",
      riskNotes: [],
      routeState: "stable",
    };
    return [
      current,
      ...flattenAuthorities(item.children, level + 1, item.authorityName),
    ];
  });
}

function countDescendants(item: AuthorityInfo): number {
  return item.children.reduce((total, child) => total + 1 + countDescendants(child), 0);
}

function resolveAuthorityName(authorityId: number) {
  if (authorityId === 0) {
    return "根节点";
  }
  return workspaceRows.value.find((item) => item.authorityId === authorityId)?.authorityName ?? `#${authorityId}`;
}

function badgeTone(level: RiskLevel) {
  if (level === "high") return "danger";
  if (level === "medium") return "warning";
  return "success";
}

function resolveRiskLabel(level: RiskLevel) {
  if (level === "high") return "高风险";
  if (level === "medium") return "待复核";
  return "稳定";
}

function hydrateForm(authority: AuthorityInfo) {
  form.ID = authority.ID || undefined;
  form.authorityId = authority.authorityId;
  form.authorityName = authority.authorityName;
  form.defaultRouter = authority.defaultRouter;
  form.parentId = authority.parentId;
}

function selectRole(authority: AuthorityWorkspaceRow) {
  selectedRole.value = authority;
  hydrateForm(authority);
  message.value = `已切换到角色 ${authority.authorityName}`;
}

function selectRoleById(authorityId: number) {
  const target = workspaceRows.value.find((item) => item.authorityId === authorityId);
  if (target) {
    selectRole(target);
  }
}

function selectRoot() {
  const root = rootRoles.value[0];
  if (root) {
    selectRole(root);
  }
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  filteredRows.value = workspaceRows.value.filter((item) => {
    const keywordMatched =
      !keyword ||
      `${item.authorityId}`.includes(keyword) ||
      item.authorityName.toLowerCase().includes(keyword) ||
      item.defaultRouter.toLowerCase().includes(keyword) ||
      item.parentName.toLowerCase().includes(keyword);

    const scopeMatched =
      !filters.scope ||
      (filters.scope === "root" && item.parentId === 0) ||
      (filters.scope === "branch" && item.childCount > 0) ||
      (filters.scope === "leaf" && item.childCount === 0);

    const routeMatched = !filters.routeState || item.routeState === filters.routeState;
    const riskMatched = !filters.risk || item.riskLevel === filters.risk;

    return keywordMatched && scopeMatched && routeMatched && riskMatched;
  });

  if (filteredRows.value.length && !filteredRows.value.some((item) => item.authorityId === selectedRole.value?.authorityId)) {
    selectedRole.value = filteredRows.value[0];
    hydrateForm(filteredRows.value[0]);
  }

  message.value = `筛选完成，命中 ${filteredRows.value.length} 个角色`;
}

function clearFilters() {
  filters.keyword = "";
  filters.scope = "";
  filters.routeState = "";
  filters.risk = "";
  filteredRows.value = [...workspaceRows.value];
  message.value = "已清空筛选条件";
}

function focusHighRisk() {
  if (!highestRiskRole.value) {
    return;
  }
  filters.risk = "high";
  applyFilters();
  selectRole(highestRiskRole.value);
}

function openCreate() {
  selectedRole.value = null;
  form.ID = undefined;
  form.authorityId = workspaceRows.value.length
    ? Math.max(...workspaceRows.value.map((item) => item.authorityId)) + 1
    : 1000;
  form.authorityName = "新角色";
  form.defaultRouter = "dashboard";
  form.parentId = 0;
  message.value = "已创建新角色草稿";
}

function openCreateChild() {
  if (!selectedRole.value) {
    openCreate();
    return;
  }
  form.ID = undefined;
  form.authorityId = workspaceRows.value.length
    ? Math.max(...workspaceRows.value.map((item) => item.authorityId)) + 1
    : 1000;
  form.authorityName = `${selectedRole.value.authorityName}子角色`;
  form.defaultRouter = selectedRole.value.defaultRouter || "dashboard";
  form.parentId = selectedRole.value.authorityId;
  message.value = `已基于 ${selectedRole.value.authorityName} 创建子角色草稿`;
}

function loadSelectedIntoForm() {
  if (!selectedRole.value) {
    return;
  }
  hydrateForm(selectedRole.value);
  message.value = `已载入角色 ${selectedRole.value.authorityName} 的当前配置`;
}

function resetFormToSelected() {
  if (selectedRole.value) {
    hydrateForm(selectedRole.value);
    message.value = `已重置为角色 ${selectedRole.value.authorityName} 的最新配置`;
    return;
  }
  openCreate();
}

async function copyText(text: string, successMessage: string) {
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      message.value = successMessage;
      return;
    }
  } catch {
    // fall through
  }

  if (typeof window !== "undefined") {
    window.prompt("当前环境无法直接写入剪贴板，请手动复制以下内容：", text);
    message.value = "已打开复制提示，可手动复制角色摘要";
  }
}

function buildRoleSummary(authority: AuthorityWorkspaceRow) {
  return [
    `角色：${authority.authorityName} (#${authority.authorityId})`,
    `层级：${authority.hierarchyLabel}`,
    `父角色：${authority.parentName}`,
    `默认入口：${authority.defaultRouter || "未配置"}`,
    `子角色：${authority.childCount} 个，累计下级 ${authority.descendantCount} 个`,
    `风险：${resolveRiskLabel(authority.riskLevel)}`,
    `提示：${authority.riskNotes.join("；") || "暂无明显风险"}`,
  ].join("\n");
}

async function copyRoleSummary(authority: AuthorityWorkspaceRow) {
  await copyText(buildRoleSummary(authority), `已复制角色 ${authority.authorityName} 摘要`);
}

async function copySelectedSummary() {
  if (!selectedRole.value) {
    return;
  }
  await copyRoleSummary(selectedRole.value);
}

async function loadAuthorities() {
  loading.value = true;
  try {
    rows.value = await getAuthorityListApi();
    filteredRows.value = workspaceRows.value;

    if (selectedRole.value) {
      const refreshed = workspaceRows.value.find(
        (item) => item.authorityId === selectedRole.value?.authorityId,
      );
      if (refreshed) {
        selectedRole.value = refreshed;
        hydrateForm(refreshed);
      } else if (filteredRows.value.length) {
        selectRole(filteredRows.value[0]);
      }
    } else if (filteredRows.value.length) {
      selectRole(filteredRows.value[0]);
    }

    message.value = `角色数据已刷新，共 ${workspaceRows.value.length} 个角色`;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "获取角色列表失败";
  } finally {
    loading.value = false;
  }
}

async function save() {
  if (!form.authorityName.trim()) {
    message.value = "角色名称不能为空";
    return;
  }
  if (form.authorityId <= 0) {
    message.value = "角色ID必须为正整数";
    return;
  }
  if (form.parentId === form.authorityId && form.parentId !== 0) {
    message.value = "父角色不能指向自身";
    return;
  }

  loading.value = true;
  try {
    const saved = await saveAuthorityApi({
      ID: form.ID,
      authorityId: form.authorityId,
      authorityName: form.authorityName.trim(),
      defaultRouter: form.defaultRouter.trim(),
      parentId: form.parentId,
    });
    await loadAuthorities();
    const current = workspaceRows.value.find((item) => item.authorityId === saved.authorityId);
    if (current) {
      selectedRole.value = current;
      hydrateForm(current);
    }
    message.value = `角色 ${saved.authorityName} 已保存`;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "保存角色失败";
  } finally {
    loading.value = false;
  }
}

onMounted(loadAuthorities);
</script>

<style scoped>
.page-shell {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-row,
.table-header,
.spotlight-header,
.row,
.hero-actions,
.inline-actions {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.hero-actions,
.inline-actions,
.row.wrap {
  flex-wrap: wrap;
  justify-content: flex-start;
}

.summary-grid,
.spotlight-grid,
.workspace-grid,
.toolbar-grid,
.detail-grid {
  display: grid;
  gap: 12px;
}

.summary-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.spotlight-grid,
.workspace-grid {
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.toolbar-grid {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.detail-grid {
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}

.summary-card,
.spotlight-card,
.detail-item {
  background: var(--vb-color-bg-card, #fff);
  border: 1px solid var(--vb-color-border, #e5e7eb);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
}

.summary-card.success {
  border-color: rgba(34, 197, 94, 0.35);
}

.summary-card.warning {
  border-color: rgba(245, 158, 11, 0.4);
}

.summary-card.danger {
  border-color: rgba(239, 68, 68, 0.35);
}

.summary-label,
.detail-item span,
.hint,
.rank-meta,
.empty-text {
  color: var(--vb-color-text-2, #6b7280);
  font-size: 12px;
}

.summary-value {
  font-size: 28px;
  line-height: 1;
}

.summary-note {
  color: var(--vb-color-text-3, #9ca3af);
  font-size: 12px;
}

.feedback-banner {
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 10px;
  color: var(--vb-color-text-1, #111827);
  padding: 10px 12px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-button,
.link-btn {
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.22);
  border-radius: 999px;
  color: inherit;
  cursor: pointer;
  padding: 6px 10px;
}

.tag-button.active {
  background: rgba(59, 130, 246, 0.16);
}

.rank-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.rank-item {
  align-items: center;
  display: flex;
  gap: 8px;
  justify-content: space-between;
}

.data-table table {
  width: 100%;
}

.data-table tr {
  cursor: pointer;
}

.data-table tr.selected {
  background: rgba(59, 130, 246, 0.08);
}

.data-table tr.warning {
  background: rgba(245, 158, 11, 0.06);
}

.data-table tr.danger {
  background: rgba(239, 68, 68, 0.06);
}

.empty-cell,
.empty-panel {
  color: var(--vb-color-text-2, #6b7280);
  padding: 16px 8px;
  text-align: center;
}

.detail-note {
  border: 1px solid var(--vb-color-border, #e5e7eb);
  border-radius: 12px;
  margin-top: 12px;
  padding: 14px;
}

.detail-note h4 {
  margin: 0 0 8px;
}

.detail-note ul {
  margin: 0;
  padding-left: 18px;
}

.badge {
  border-radius: 999px;
  display: inline-flex;
  font-size: 12px;
  padding: 4px 10px;
}

.badge.success {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.badge.warning {
  background: rgba(245, 158, 11, 0.14);
  color: #b45309;
}

.badge.danger {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field input,
.field select {
  min-width: 0;
}
</style>
