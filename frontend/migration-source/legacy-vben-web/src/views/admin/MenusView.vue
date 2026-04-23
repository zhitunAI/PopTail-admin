<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">菜单工作台</h3>
          <p class="subtitle">展示菜单树、路由映射、显隐状态与层级分布，支持新增、编辑、筛选和详情联动。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="showTopLevel">只看一级菜单</button>
          <button class="btn ghost" :disabled="loading" @click="load">恢复全部</button>
          <button class="btn primary" @click="openCreate">新增菜单</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>菜单总数</h4>
        <p class="stat-value">{{ sourceRows.length }}</p>
        <p class="subtitle">当前菜单台账总量</p>
      </div>
      <div class="stat-card">
        <h4>一级菜单</h4>
        <p class="stat-value">{{ topLevelCount }}</p>
        <p class="subtitle">可直接挂在导航栏的入口</p>
      </div>
      <div class="stat-card">
        <h4>隐藏菜单</h4>
        <p class="stat-value">{{ hiddenCount }}</p>
        <p class="subtitle">需确认是否仍应保留入口</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeMenu?.meta.title || "-" }}</p>
        <p class="subtitle">{{ activeMenu ? menuHealth(activeMenu) : "请选择菜单查看详情" }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <div class="row between wrap">
          <h3 class="title">菜单列表</h3>
          <div class="toolbar-grid compact">
            <div class="field">
              <label>关键词</label>
              <input v-model.trim="filters.keyword" placeholder="标题 / 路由名 / 路径" @keydown.enter="applyFilters" />
            </div>
            <div class="field">
              <label>显隐</label>
              <select v-model="filters.visibility">
                <option value="all">全部</option>
                <option value="visible">仅显示可见菜单</option>
                <option value="hidden">仅显示隐藏菜单</option>
              </select>
            </div>
          </div>
        </div>
        <div class="row wrap">
          <button class="btn ghost" @click="applyFilters">应用筛选</button>
          <button class="btn ghost" @click="resetFilters">重置筛选</button>
          <button class="btn ghost" :disabled="!activeMenu" @click="copySummary">复制菜单摘要</button>
        </div>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>标题</th>
                <th>路由名</th>
                <th>路径</th>
                <th>组件</th>
                <th>父级</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in rows"
                :key="item.ID"
                :class="{ selected: activeMenu?.ID === item.ID }"
                @click="inspect(item)"
              >
                <td>{{ item.ID }}</td>
                <td>{{ item.meta.title }}</td>
                <td>{{ item.name }}</td>
                <td>{{ item.path }}</td>
                <td>{{ item.component }}</td>
                <td>{{ item.parentId }}</td>
                <td><button class="btn ghost" @click.stop="inspect(item)">详情</button></td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="7">暂无匹配的菜单</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">菜单详情</h3>
        <div v-if="activeMenu" class="stack">
          <div class="data-table">
            <table>
              <tbody>
                <tr><td>标题</td><td>{{ activeMenu.meta.title }}</td></tr>
                <tr><td>图标</td><td>{{ activeMenu.meta.icon }}</td></tr>
                <tr><td>路径</td><td>{{ activeMenu.path }}</td></tr>
                <tr><td>组件</td><td>{{ activeMenu.component }}</td></tr>
                <tr><td>隐藏</td><td>{{ activeMenu.hidden ? "是" : "否" }}</td></tr>
                <tr><td>子菜单数</td><td>{{ activeMenu.children.length }}</td></tr>
                <tr><td>评级</td><td>{{ menuHealth(activeMenu) }}</td></tr>
              </tbody>
            </table>
          </div>
          <div class="tag-list">
            <span v-for="child in activeMenu.children" :key="child.ID" class="tag">{{ child.meta.title }}</span>
            <span v-if="!activeMenu.children.length" class="tag">暂无子菜单</span>
          </div>
        </div>
        <p v-else class="subtitle">请选择一个菜单查看详情。</p>
      </div>
    </div>

    <div v-if="activeMenu" class="card">
      <h3 class="title">{{ form.ID ? "编辑菜单" : "新增菜单" }}</h3>
      <div class="toolbar-grid">
        <div class="field">
          <label>标题</label>
          <input v-model.trim="form.meta.title" />
        </div>
        <div class="field">
          <label>图标</label>
          <input v-model.trim="form.meta.icon" />
        </div>
        <div class="field">
          <label>路由名</label>
          <input v-model.trim="form.name" />
        </div>
        <div class="field">
          <label>路径</label>
          <input v-model.trim="form.path" />
        </div>
        <div class="field">
          <label>组件</label>
          <input v-model.trim="form.component" />
        </div>
        <div class="field">
          <label>父级</label>
          <select v-model.number="form.parentId">
            <option :value="0">根节点</option>
            <option v-for="item in sourceRows" :key="item.ID" :value="item.ID">
              {{ item.meta.title }}
            </option>
          </select>
        </div>
        <div class="field">
          <label>排序</label>
          <input v-model.number="form.sort" type="number" min="0" />
        </div>
        <div class="field">
          <label>隐藏</label>
          <select v-model="form.hidden">
            <option :value="false">否</option>
            <option :value="true">是</option>
          </select>
        </div>
      </div>

      <div v-if="validationIssues.length" class="state-banner error">
        <strong>保存前请修正：</strong>
        <ul>
          <li v-for="item in validationIssues" :key="item">{{ item }}</li>
        </ul>
      </div>

      <div class="row wrap">
        <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="save">
          {{ saving ? "保存中..." : "保存菜单" }}
        </button>
        <button class="btn ghost" @click="inspect(activeMenu)">重置</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getMenuListApi, saveMenuApi } from "../../api/admin";
import type { MenuInfo } from "../../types";

const rows = ref<MenuInfo[]>([]);
const sourceRows = ref<MenuInfo[]>([]);
const activeMenu = ref<MenuInfo | null>(null);
const message = ref("");
const error = ref("");
const loading = ref(false);
const saving = ref(false);
const filters = reactive({
  keyword: "",
  visibility: "all" as "all" | "hidden" | "visible",
});
const form = reactive({
  ID: undefined as number | undefined,
  parentId: 0,
  name: "",
  path: "",
  component: "",
  sort: 0,
  hidden: false,
  meta: {
    title: "",
    icon: "",
  },
});

const topLevelCount = computed(() => sourceRows.value.filter((item) => item.parentId === 0).length);
const hiddenCount = computed(() => sourceRows.value.filter((item) => item.hidden).length);
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.meta.title.trim()) issues.push("菜单标题不能为空。");
  if (!form.name.trim()) issues.push("路由名不能为空。");
  if (!form.path.trim()) issues.push("路径不能为空。");
  if (!form.component.trim()) issues.push("组件不能为空。");
  return issues;
});

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const result = await getMenuListApi();
    rows.value = result.List;
    sourceRows.value = result.List;
    if (activeMenu.value) {
      const refreshed = sourceRows.value.find((item) => item.ID === activeMenu.value?.ID);
      if (refreshed) {
        activeMenu.value = refreshed;
        hydrateForm(refreshed);
      }
    }
    message.value = "已刷新菜单台账。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取菜单失败";
  } finally {
    loading.value = false;
  }
}

function inspect(menu: MenuInfo) {
  activeMenu.value = menu;
  hydrateForm(menu);
}

function showTopLevel() {
  rows.value = sourceRows.value.filter((item) => item.parentId === 0);
  message.value = "已切换为一级菜单视图。";
}

function applyFilters() {
  const keyword = filters.keyword.trim().toLowerCase();
  rows.value = sourceRows.value.filter((item) => {
    const matchKeyword =
      !keyword ||
      [item.meta.title, item.name, item.path, item.component].some((part) =>
        part.toLowerCase().includes(keyword),
      );
    const matchVisibility =
      filters.visibility === "all" ||
      (filters.visibility === "hidden" ? item.hidden : !item.hidden);
    return matchKeyword && matchVisibility;
  });
  message.value = `已筛选出 ${rows.value.length} 个菜单。`;
}

function resetFilters() {
  filters.keyword = "";
  filters.visibility = "all";
  rows.value = sourceRows.value;
  message.value = "已重置菜单筛选条件。";
}

function hydrateForm(menu: MenuInfo) {
  form.ID = menu.ID;
  form.parentId = menu.parentId;
  form.name = menu.name;
  form.path = menu.path;
  form.component = menu.component;
  form.sort = menu.sort;
  form.hidden = menu.hidden;
  form.meta.title = menu.meta.title;
  form.meta.icon = menu.meta.icon;
}

function openCreate() {
  activeMenu.value = {
    ID: 0,
    parentId: 0,
    name: "newMenu",
    path: "/new-menu",
    component: "views/admin/NewMenuView.vue",
    sort: sourceRows.value.length + 1,
    hidden: false,
    meta: { title: "新菜单", icon: "square-plus" },
    children: [],
  };
  hydrateForm(activeMenu.value);
  form.ID = undefined;
  message.value = "已打开新菜单草稿。";
}

function menuHealth(menu: MenuInfo) {
  if (menu.hidden) return "隐藏入口，需确认是否仍要保留";
  if (!menu.children.length && menu.parentId === 0) return "一级直达入口";
  if (menu.children.length) return `含 ${menu.children.length} 个子菜单`;
  return "常规菜单节点";
}

async function copySummary() {
  if (!activeMenu.value) return;
  const text = [
    `菜单：${activeMenu.value.meta.title}`,
    `路由名：${activeMenu.value.name}`,
    `路径：${activeMenu.value.path}`,
    `组件：${activeMenu.value.component}`,
    `显隐：${activeMenu.value.hidden ? "隐藏" : "可见"}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = `已复制菜单 ${activeMenu.value.meta.title} 摘要。`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = `已切换为手动复制菜单 ${activeMenu.value.meta.title} 摘要。`;
  }
}

async function save() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "菜单信息不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    const saved = await saveMenuApi({
      ID: form.ID,
      parentId: form.parentId,
      name: form.name,
      path: form.path,
      component: form.component,
      sort: form.sort,
      hidden: form.hidden,
      meta: { ...form.meta },
    });
    message.value = `菜单 ${saved.meta.title} 已保存`;
    await load();
    const current = sourceRows.value.find((item) => item.ID === saved.ID);
    if (current) {
      activeMenu.value = current;
      hydrateForm(current);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存菜单失败";
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>
