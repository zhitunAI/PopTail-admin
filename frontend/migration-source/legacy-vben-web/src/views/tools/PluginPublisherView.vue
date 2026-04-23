<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">插件打包工作台</h3>
          <p class="subtitle">从菜单、接口、字典中组装插件清单，预览资源覆盖范围，并沉淀最近打包记录。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="loadWorkspace">
            {{ loading ? "刷新中..." : "刷新资源" }}
          </button>
          <button class="btn primary" :disabled="saving" @click="saveManifest">
            {{ saving ? "保存中..." : "保存清单" }}
          </button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>已选资源</h4>
        <p class="stat-value">{{ totalSelected }}</p>
        <p class="subtitle">菜单 {{ selected.menuIds.length }} / 接口 {{ selected.apiIds.length }} / 字典 {{ selected.dictionaryIds.length }}</p>
      </div>
      <div class="stat-card">
        <h4>资源池</h4>
        <p class="stat-value">{{ menuItems.length + apiItems.length + dictionaryItems.length }}</p>
        <p class="subtitle">当前可打包的总资源数</p>
      </div>
      <div class="stat-card">
        <h4>最近清单</h4>
        <p class="stat-value">{{ manifests.length }}</p>
        <p class="subtitle">{{ manifests[0]?.pluginName ?? "暂无保存记录" }}</p>
      </div>
      <div class="stat-card">
        <h4>推荐范围</h4>
        <p class="stat-value">{{ riskLevel }}</p>
        <p class="subtitle">{{ rolloutAdvice }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">基础信息</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>插件名</label>
            <input v-model.trim="pluginName" placeholder="输入插件名称" />
          </div>
          <div class="field">
            <label>菜单组名</label>
            <input v-model.trim="menuGroup" placeholder="例如 风控中心" />
          </div>
        </div>
        <div class="tag-list">
          <button class="tag action-tag" @click="applyTemplate('ops')">运维工具模板</button>
          <button class="tag action-tag" @click="applyTemplate('content')">内容治理模板</button>
          <button class="tag action-tag" @click="applyTemplate('minimal')">最小插件模板</button>
          <button class="tag action-tag" @click="resetSelection">清空选择</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">覆盖摘要</h3>
        <div class="data-table">
          <table>
            <tbody>
              <tr><td>菜单覆盖</td><td>{{ selectedMenuTitles.join("、") || "未选择" }}</td></tr>
              <tr><td>接口覆盖</td><td>{{ selectedApiTitles.join("、") || "未选择" }}</td></tr>
              <tr><td>字典覆盖</td><td>{{ selectedDictionaryTitles.join("、") || "未选择" }}</td></tr>
              <tr><td>发布建议</td><td>{{ rolloutAdvice }}</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">资源选择</h3>
        <div class="stack">
          <div class="field">
            <label>菜单</label>
            <select v-model="selected.menuIds" multiple size="8">
              <option v-for="item in menuItems" :key="item.ID" :value="item.ID">
                {{ item.meta.title }}
              </option>
            </select>
          </div>
          <div class="field">
            <label>接口</label>
            <select v-model="selected.apiIds" multiple size="8">
              <option v-for="item in apiItems" :key="item.ID" :value="item.ID">
                {{ item.description }}
              </option>
            </select>
          </div>
          <div class="field">
            <label>字典</label>
            <select v-model="selected.dictionaryIds" multiple size="8">
              <option v-for="item in dictionaryItems" :key="item.ID" :value="item.ID">
                {{ item.name }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <div class="card">
        <h3 class="title">安装清单预览</h3>
        <pre class="code-block">{{ manifest }}</pre>
        <div class="row wrap">
          <button class="btn ghost" @click="copyManifest">复制清单</button>
          <button class="btn ghost" :disabled="!manifests.length" @click="loadLatest">载入最近保存</button>
        </div>
        <p v-if="savedAt" class="subtitle">最近保存：{{ savedAt }}</p>
      </div>
    </div>

    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">最近保存记录</h3>
          <p class="subtitle">用于回看插件装配范围，快速复用既有清单。</p>
        </div>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>插件名</th>
              <th>菜单组</th>
              <th>资源数</th>
              <th>保存时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in manifests" :key="item.ID">
              <td>{{ item.ID }}</td>
              <td>{{ item.pluginName }}</td>
              <td>{{ item.menuGroup }}</td>
              <td>{{ item.menuIds.length + item.apiIds.length + item.dictionaryIds.length }}</td>
              <td>{{ formatTime(item.savedAt) }}</td>
              <td>
                <button class="btn ghost" @click="restoreManifest(item)">载入</button>
              </td>
            </tr>
            <tr v-if="!manifests.length">
              <td colspan="6">暂无插件清单记录</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  getApiListApi,
  getDictionaryListApi,
  getMenuListApi,
  getPluginManifestListApi,
  savePluginManifestApi,
} from "../../api/admin";
import type { ApiInfo, DictionaryInfo, MenuInfo, PluginManifestRecord } from "../../types";

const pluginName = ref("ops-toolkit");
const menuGroup = ref("运维工具");
const apiItems = ref<ApiInfo[]>([]);
const dictionaryItems = ref<DictionaryInfo[]>([]);
const menuItems = ref<MenuInfo[]>([]);
const manifests = ref<PluginManifestRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const message = ref("");
const error = ref("");
const savedAt = ref("");
const selected = ref({
  menuIds: [] as number[],
  apiIds: [] as number[],
  dictionaryIds: [] as number[],
});

const totalSelected = computed(
  () => selected.value.menuIds.length + selected.value.apiIds.length + selected.value.dictionaryIds.length,
);

const selectedMenuTitles = computed(() =>
  menuItems.value
    .filter((item) => selected.value.menuIds.includes(item.ID))
    .map((item) => item.meta.title),
);
const selectedApiTitles = computed(() =>
  apiItems.value
    .filter((item) => selected.value.apiIds.includes(item.ID))
    .map((item) => item.description),
);
const selectedDictionaryTitles = computed(() =>
  dictionaryItems.value
    .filter((item) => selected.value.dictionaryIds.includes(item.ID))
    .map((item) => item.name),
);

const riskLevel = computed(() => {
  if (totalSelected.value >= 12) {
    return "高";
  }
  if (totalSelected.value >= 6) {
    return "中";
  }
  return "低";
});

const rolloutAdvice = computed(() => {
  if (totalSelected.value === 0) {
    return "先至少选择一类资源，再生成插件清单。";
  }
  if (selected.value.apiIds.length && !selected.value.menuIds.length) {
    return "当前更像无 UI 能力插件，建议确认是否还需菜单入口。";
  }
  if (riskLevel.value === "高") {
    return "资源覆盖较大，建议先在 workspace 环境验证后再推广。";
  }
  return "当前覆盖范围适中，可继续进入安装与联调流程。";
});

const manifest = computed(() =>
  JSON.stringify(
    {
      pluginName: pluginName.value,
      menuGroup: menuGroup.value,
      menuIds: selected.value.menuIds,
      apiIds: selected.value.apiIds,
      dictionaryIds: selected.value.dictionaryIds,
      summary: {
        totalSelected: totalSelected.value,
        riskLevel: riskLevel.value,
      },
    },
    null,
    2,
  ),
);

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleString("zh-CN");
}

async function loadWorkspace() {
  loading.value = true;
  error.value = "";
  try {
    const [menus, apis, dictionaries, manifestResult] = await Promise.all([
      getMenuListApi(),
      getApiListApi(),
      getDictionaryListApi(),
      getPluginManifestListApi(),
    ]);
    menuItems.value = menus.List;
    apiItems.value = apis.List;
    dictionaryItems.value = dictionaries.List;
    manifests.value = [...manifestResult.List].sort((a, b) => b.ID - a.ID);
    if (!savedAt.value && manifests.value[0]) {
      restoreManifest(manifests.value[0]);
    }
    message.value = `已加载菜单 ${menuItems.value.length}、接口 ${apiItems.value.length}、字典 ${dictionaryItems.value.length}。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "加载插件资源失败";
  } finally {
    loading.value = false;
  }
}

function restoreManifest(record: PluginManifestRecord) {
  pluginName.value = record.pluginName;
  menuGroup.value = record.menuGroup;
  selected.value = {
    menuIds: [...record.menuIds],
    apiIds: [...record.apiIds],
    dictionaryIds: [...record.dictionaryIds],
  };
  savedAt.value = formatTime(record.savedAt);
  message.value = `已载入插件清单 #${record.ID}`;
}

function loadLatest() {
  if (manifests.value[0]) {
    restoreManifest(manifests.value[0]);
  }
}

function resetSelection() {
  selected.value = {
    menuIds: [],
    apiIds: [],
    dictionaryIds: [],
  };
  message.value = "已清空当前资源选择。";
}

function applyTemplate(kind: "ops" | "content" | "minimal") {
  if (kind === "minimal") {
    selected.value = {
      menuIds: menuItems.value.slice(0, 1).map((item) => item.ID),
      apiIds: apiItems.value.slice(0, 1).map((item) => item.ID),
      dictionaryIds: [],
    };
    message.value = "已套用最小插件模板。";
    return;
  }

  if (kind === "ops") {
    pluginName.value = "ops-toolkit";
    menuGroup.value = "运维工具";
    selected.value = {
      menuIds: menuItems.value.slice(0, 2).map((item) => item.ID),
      apiIds: apiItems.value.slice(0, 3).map((item) => item.ID),
      dictionaryIds: dictionaryItems.value.slice(0, 1).map((item) => item.ID),
    };
    message.value = "已套用运维工具模板。";
    return;
  }

  pluginName.value = "content-guard";
  menuGroup.value = "内容治理";
  selected.value = {
    menuIds: menuItems.value.slice(0, 2).map((item) => item.ID),
    apiIds: apiItems.value.slice(0, 2).map((item) => item.ID),
    dictionaryIds: dictionaryItems.value.slice(0, 2).map((item) => item.ID),
  };
  message.value = "已套用内容治理模板。";
}

async function copyManifest() {
  try {
    await navigator.clipboard.writeText(manifest.value);
    message.value = "已复制插件清单。";
  } catch {
    window.prompt("复制插件清单", manifest.value);
    message.value = "当前环境无法直接写入剪贴板，已切换为手动复制。";
  }
}

async function saveManifest() {
  saving.value = true;
  error.value = "";
  try {
    const saved = await savePluginManifestApi({
      pluginName: pluginName.value,
      menuGroup: menuGroup.value,
      menuIds: selected.value.menuIds,
      apiIds: selected.value.apiIds,
      dictionaryIds: selected.value.dictionaryIds,
    });
    savedAt.value = formatTime(saved.savedAt);
    message.value = `已保存插件清单 #${saved.ID}，共覆盖 ${totalSelected.value} 项资源。`;
    await loadWorkspace();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存插件清单失败";
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void loadWorkspace();
});
</script>
