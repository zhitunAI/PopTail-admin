<template>
  <div class="stack">
    <div class="card">
      <div class="row between">
        <div>
          <h3 class="title">菜单图标工作台</h3>
          <p class="subtitle">按菜单场景筛选图标、复制名称，并快速决定当前菜单配置该用哪个图标。</p>
        </div>
        <button class="btn ghost" @click="resetFilters">重置筛选</button>
      </div>
      <div class="toolbar-grid">
        <div class="field">
          <label>关键词</label>
          <input v-model="keyword" placeholder="输入图标名称或场景" />
        </div>
        <div class="field">
          <label>使用场景</label>
          <select v-model="activeGroup">
            <option value="">全部</option>
            <option v-for="group in sceneOptions" :key="group" :value="group">{{ group }}</option>
          </select>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">图标概览</h3>
        <div class="muted-grid">
          <div class="stat-card">
            <h4>可用图标</h4>
            <p class="stat-value">{{ filteredIcons.length }}</p>
          </div>
          <div class="stat-card">
            <h4>分类数</h4>
            <p class="stat-value">{{ visibleGroups.length }}</p>
          </div>
          <div class="stat-card">
            <h4>当前推荐</h4>
            <p class="stat-value">{{ recommendedIcon?.name ?? "-" }}</p>
          </div>
        </div>
        <div class="tag-list">
          <button
            v-for="icon in filteredIcons"
            :key="icon.name"
            class="tag interactive"
            :class="{ active: selectedIcon?.name === icon.name }"
            @click="selectIcon(icon)"
          >
            {{ icon.name }}
          </button>
          <span v-if="!filteredIcons.length" class="tag">暂无匹配图标</span>
        </div>
      </div>

      <div class="card">
        <h3 class="title">图标详情</h3>
        <div v-if="selectedIcon" class="stack">
          <div class="data-table">
            <table>
              <tbody>
                <tr><td>图标名称</td><td>{{ selectedIcon.name }}</td></tr>
                <tr><td>推荐场景</td><td>{{ selectedIcon.group }}</td></tr>
                <tr><td>说明</td><td>{{ selectedIcon.note }}</td></tr>
                <tr><td>适用页面</td><td>{{ selectedIcon.targets.join("、") }}</td></tr>
              </tbody>
            </table>
          </div>
          <div class="tag-list">
            <button class="btn primary" @click="copyName(selectedIcon.name)">复制图标名</button>
            <button class="btn ghost" @click="copyTargets(selectedIcon)">复制适用说明</button>
          </div>
          <p v-if="message" class="subtitle">{{ message }}</p>
        </div>
        <p v-else class="subtitle">请选择左侧图标查看详情。</p>
      </div>
    </div>

    <div class="card">
      <h3 class="title">分类分布</h3>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>场景</th>
              <th>数量</th>
              <th>首选图标</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="group in visibleGroups" :key="group.name">
              <td>{{ group.name }}</td>
              <td>{{ group.items.length }}</td>
              <td>{{ group.items[0]?.name ?? "-" }}</td>
              <td>
                <button class="btn ghost" @click="focusGroup(group.name)">筛选本组</button>
              </td>
            </tr>
            <tr v-if="!visibleGroups.length">
              <td colspan="4">当前筛选下暂无图标分组</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

type IconCatalogItem = {
  group: string;
  name: string;
  note: string;
  targets: string[];
};

const keyword = ref("");
const activeGroup = ref("");
const message = ref("");

const iconCatalog: IconCatalogItem[] = [
  { name: "layout-grid", group: "总览", note: "适合首页、总览页、聚合看板。", targets: ["控制台", "系统总览"] },
  { name: "user", group: "账户", note: "适合用户、个人资料、成员管理。", targets: ["用户管理", "个人资料"] },
  { name: "shield", group: "权限", note: "适合角色、鉴权、权限控制。", targets: ["角色管理", "API 台账"] },
  { name: "menu", group: "导航", note: "适合菜单、导航、路由配置。", targets: ["菜单管理"] },
  { name: "book", group: "字典", note: "适合字典、配置规则、知识类台账。", targets: ["字典管理", "字典详情"] },
  { name: "sliders", group: "配置", note: "适合参数、系统配置、开关项。", targets: ["参数配置", "系统配置"] },
  { name: "server", group: "运行态", note: "适合服务状态、运行时、主机信息。", targets: ["系统状态", "运行态工作台"] },
  { name: "terminal", group: "工具链", note: "适合系统工具、命令、诊断入口。", targets: ["系统工具"] },
  { name: "plug", group: "插件", note: "适合插件市场、安装清单、发布记录。", targets: ["插件发布", "插件安装"] },
  { name: "package-plus", group: "交付", note: "适合打包、自动生成、交付资产。", targets: ["代码包管理", "自动代码"] },
  { name: "wand", group: "自动化", note: "适合 AI、自动流程、工作流入口。", targets: ["AI 工作流", "技能台账"] },
  { name: "image", group: "页面设计", note: "适合页面草图、素材设计与导图。", targets: ["页面草图", "表单设计器"] },
  { name: "history", group: "日志", note: "适合登录日志、操作日志、发布记录。", targets: ["登录日志", "操作日志", "版本记录"] },
  { name: "alert-circle", group: "异常", note: "适合错误日志、异常恢复、告警页。", targets: ["错误日志", "异常恢复"] },
  { name: "download", group: "导出", note: "适合导出模板、下载中心、发布物。", targets: ["导出模板", "发布记录"] },
];

const filteredIcons = computed(() => {
  const normalized = keyword.value.trim().toLowerCase();
  return iconCatalog.filter((item) => {
    const matchesKeyword =
      !normalized ||
      item.name.toLowerCase().includes(normalized) ||
      item.group.toLowerCase().includes(normalized) ||
      item.targets.some((target) => target.toLowerCase().includes(normalized));
    const matchesGroup = !activeGroup.value || item.group === activeGroup.value;
    return matchesKeyword && matchesGroup;
  });
});

const sceneOptions = computed(() =>
  [...new Set(iconCatalog.map((item) => item.group))].sort((a, b) => a.localeCompare(b)),
);

const visibleGroups = computed(() =>
  sceneOptions.value
    .map((group) => ({
      name: group,
      items: filteredIcons.value.filter((item) => item.group === group),
    }))
    .filter((group) => group.items.length),
);

const selectedIcon = ref<IconCatalogItem | null>(iconCatalog[0] ?? null);

const recommendedIcon = computed(() => {
  if (selectedIcon.value) {
    return selectedIcon.value;
  }
  return filteredIcons.value[0] ?? null;
});

function selectIcon(icon: IconCatalogItem) {
  selectedIcon.value = icon;
  message.value = `${icon.name} 已设为当前参考图标。`;
}

function focusGroup(group: string) {
  activeGroup.value = group;
  const first = filteredIcons.value.find((item) => item.group === group) ?? null;
  selectedIcon.value = first;
  message.value = `已聚焦 ${group} 分类。`;
}

async function copyName(name: string) {
  await navigator.clipboard.writeText(name);
  message.value = `已复制图标名：${name}`;
}

async function copyTargets(icon: IconCatalogItem) {
  const text = `${icon.name}：适合 ${icon.targets.join("、")}；${icon.note}`;
  await navigator.clipboard.writeText(text);
  message.value = `已复制 ${icon.name} 的适用说明。`;
}

function resetFilters() {
  keyword.value = "";
  activeGroup.value = "";
  selectedIcon.value = iconCatalog[0] ?? null;
  message.value = "";
}
</script>
