<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">系统工具工作台</h3>
          <p class="subtitle">汇总 MCP、技能、插件安装与自动代码资产，给出工具健康矩阵、推荐流程与快速入口。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="reload">
            {{ loading ? "刷新中..." : "刷新工具状态" }}
          </button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>MCP 工具</h4>
        <p class="stat-value">{{ stats.mcpTools }}</p>
        <p class="subtitle">{{ stats.mcpState }}</p>
      </div>
      <div class="stat-card">
        <h4>技能资产</h4>
        <p class="stat-value">{{ stats.skills }}</p>
        <p class="subtitle">{{ skillAdvice }}</p>
      </div>
      <div class="stat-card">
        <h4>插件安装</h4>
        <p class="stat-value">{{ stats.plugins }}</p>
        <p class="subtitle">{{ pluginAdvice }}</p>
      </div>
      <div class="stat-card">
        <h4>自动代码</h4>
        <p class="stat-value">{{ stats.autoCode }}</p>
        <p class="subtitle">{{ autoCodeAdvice }}</p>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">工具健康矩阵</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>模块</th>
                <th>数量</th>
                <th>健康度</th>
                <th>说明</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in healthRows" :key="item.name">
                <td>{{ item.name }}</td>
                <td>{{ item.count }}</td>
                <td>{{ item.health }}</td>
                <td>{{ item.detail }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="card">
        <h3 class="title">推荐流程</h3>
        <div class="signal-list">
          <div v-for="item in suggestions" :key="item.title" class="signal-item">
            <div class="row between wrap">
              <strong>{{ item.title }}</strong>
              <RouterLink class="btn ghost" :to="item.path">打开</RouterLink>
            </div>
            <p>{{ item.detail }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">工具入口</h3>
          <p class="subtitle">按健康度与用途组织工具入口，方便从总览页直接跳转到目标模块。</p>
        </div>
        <span class="tag">{{ rows.length }} 个入口</span>
      </div>
      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>模块</th>
              <th>状态摘要</th>
              <th>健康度</th>
              <th>入口</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in rows" :key="item.path">
              <td>{{ item.name }}</td>
              <td>{{ item.desc }}</td>
              <td>{{ item.health }}</td>
              <td><RouterLink class="btn ghost" :to="item.path">打开</RouterLink></td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="4">暂无工具入口数据</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { RouterLink } from "vue-router";
import {
  getAutoCodeRegistryApi,
  getMcpStatusApi,
  getMcpToolListApi,
  getPluginInstallListApi,
  getSkillListApi,
} from "../../api/admin";

const loading = ref(false);
const message = ref("");
const error = ref("");
const stats = reactive({
  mcpTools: 0,
  mcpState: "-",
  skills: 0,
  plugins: 0,
  autoCode: 0,
});

const rows = ref<Array<{ desc: string; health: string; name: string; path: string }>>([]);

const skillAdvice = computed(() => (stats.skills > 0 ? "技能资产已接后端" : "建议先补齐技能说明"));
const pluginAdvice = computed(() => (stats.plugins > 0 ? "存在安装记录，可继续回归" : "尚无安装记录"));
const autoCodeAdvice = computed(() => (stats.autoCode > 0 ? "已有蓝图台账" : "建议先生成第一份蓝图"));

const healthRows = computed(() => [
  {
    name: "MCP",
    count: stats.mcpTools,
    health: stats.mcpTools > 0 ? "可用" : "待补齐",
    detail: stats.mcpState,
  },
  {
    name: "技能",
    count: stats.skills,
    health: stats.skills > 0 ? "稳定" : "待建设",
    detail: skillAdvice.value,
  },
  {
    name: "插件",
    count: stats.plugins,
    health: stats.plugins > 0 ? "可回归" : "待安装",
    detail: pluginAdvice.value,
  },
  {
    name: "自动代码",
    count: stats.autoCode,
    health: stats.autoCode > 0 ? "可推进" : "待生成",
    detail: autoCodeAdvice.value,
  },
]);

const suggestions = computed(() => [
  {
    title: "先整理蓝图",
    detail: "优先进入自动代码页梳理结构、字段与输出目标，减少后续工具定义反复修改。",
    path: "/system/tools/autocode",
  },
  {
    title: "再定义 MCP",
    detail: "完成蓝图后进入 MCP 生成/测试页，把工具输入输出与执行约束整理完整。",
    path: "/system/tools/autocode/mcp",
  },
  {
    title: "最后补技能与安装链路",
    detail: "技能资产、邮件插件、公告管理与插件安装页用于沉淀工作流说明和交付运营链路。",
    path: "/system/tools/skills",
  },
]);

async function reload() {
  loading.value = true;
  error.value = "";
  try {
    const [mcpStatus, mcpTools, skills, installs, autoCode] = await Promise.all([
      getMcpStatusApi(),
      getMcpToolListApi(),
      getSkillListApi(),
      getPluginInstallListApi(),
      getAutoCodeRegistryApi(),
    ]);

    stats.mcpTools = mcpTools.length;
    stats.mcpState = mcpStatus.message;
    stats.skills = skills.length;
    stats.plugins = installs.Total;
    stats.autoCode = autoCode.Total;

    rows.value = [
      {
        name: "MCP 生成",
        desc: `已登记 ${mcpTools.length} 个工具，服务状态 ${mcpStatus.state}`,
        health: mcpTools.length ? "可用" : "待补齐",
        path: "/system/tools/autocode/mcp",
      },
      {
        name: "MCP 测试",
        desc: `联调入口，当前 ${mcpStatus.reachable ? "可达" : "未启动"}`,
        health: mcpStatus.reachable ? "联通" : "待启动",
        path: "/system/tools/autocode/mcp-test",
      },
      {
        name: "技能管理",
        desc: `技能 ${skills.length} 个，支持文件资产编辑`,
        health: skills.length ? "稳定" : "待补齐",
        path: "/system/tools/skills",
      },
      {
        name: "插件安装",
        desc: `安装记录 ${installs.Total} 条`,
        health: installs.Total ? "可回归" : "待安装",
        path: "/system/tools/install-plugin",
      },
      {
        name: "自动代码台账",
        desc: `蓝图 ${autoCode.Total} 条`,
        health: autoCode.Total ? "可推进" : "待生成",
        path: "/system/tools/autocode-admin",
      },
      {
        name: "运行状态",
        desc: "容器与服务实时状态",
        health: mcpStatus.reachable ? "建议巡检" : "待排查",
        path: "/system/tools/runtime-state",
      },
      {
        name: "邮件插件",
        desc: "测试邮件与正式发送记录",
        health: "可验证",
        path: "/system/tools/plugin-email",
      },
      {
        name: "公告管理",
        desc: "公告列表、附件与作者数据源",
        health: "可验证",
        path: "/system/tools/announcement",
      },
    ];

    message.value = "已同步工具状态、资产数量与推荐流程。";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "刷新系统工具状态失败";
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void reload();
});
</script>
