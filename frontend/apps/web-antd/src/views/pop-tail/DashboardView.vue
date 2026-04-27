<script setup lang="ts">
import type {
  WorkbenchProjectItem,
  WorkbenchQuickNavItem,
  WorkbenchTodoItem,
  WorkbenchTrendItem,
} from '@vben/common-ui';

import { computed, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import {
  AnalysisChartCard,
  Page,
  WorkbenchHeader,
  WorkbenchProject,
  WorkbenchQuickNav,
  WorkbenchTodo,
  WorkbenchTrends,
} from '@vben/common-ui';
import { preferences } from '@vben/preferences';
import { useUserStore } from '@vben/stores';
import { openWindow } from '@vben/utils';

import {
  getAnnouncementListApi,
  getPluginInstallListApi,
  getReleaseListApi,
  getRuntimeInfoApi,
} from '#/api/pop-tail/admin';
import { useAuthStore } from '#/store/pop-tail/auth';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';
import type {
  AnnouncementRecord,
  PluginInstallRecord,
  ReleaseRecord,
  RuntimeInfo,
} from '#/types/pop-tail';

const auth = useAuthStore();
const userStore = useUserStore();
const router = useRouter();
const loading = ref(false);
const runtime = ref<null | RuntimeInfo>(null);
const plugins = ref<PluginInstallRecord[]>([]);
const releases = ref<ReleaseRecord[]>([]);
const notices = ref<AnnouncementRecord[]>([]);
usePageRefreshLoading(reload);

const today = computed(() => {
  try {
    return new Date().toLocaleDateString('zh-CN', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    });
  } catch {
    return new Date().toISOString().slice(0, 10);
  }
});

const quickNavItems = computed<WorkbenchQuickNavItem[]>(() => [
  {
    color: 'var(--ant-color-primary)',
    icon: 'carbon:menu',
    title: '菜单管理',
    url: '/system/menus',
  },
  {
    color: 'var(--ant-color-success)',
    icon: 'carbon:link',
    title: 'API管理',
    url: '/system/apis',
  },
  {
    color: 'var(--ant-color-warning)',
    icon: 'carbon:user-role',
    title: '角色管理',
    url: '/system/authorities',
  },
  {
    color: 'var(--ant-color-info)',
    icon: 'carbon:user-multiple',
    title: '用户管理',
    url: '/system/users',
  },
  {
    color: 'var(--ant-color-primary)',
    icon: 'carbon:box',
    title: '系统工具',
    url: '/system/tools',
  },
  {
    color: 'var(--ant-color-error)',
    icon: 'carbon:dashboard',
    title: '管理总览',
    url: '/system/overview',
  },
]);

const projectItems = computed<WorkbenchProjectItem[]>(() =>
  plugins.value.slice(0, 6).map((item, index) => ({
    color: ([
      'var(--ant-color-primary)',
      '#3fb27f',
      '#e18525',
      '#bf0c2c',
      '#00d8ff',
      '#8b5cf6',
    ] as string[])[index % 6] ?? 'var(--ant-color-primary)',
    content: item.manifest || `${item.name} 已接入后台工具链。`,
    date: formatTime(item.createdAt),
    group: item.kind || item.target || '工具域',
    icon: pluginIcon(item.kind, index) ?? 'carbon:application-web',
    title: item.name,
    url: item.target?.startsWith('/') ? item.target : '/system/tools',
  })),
);

const trendItems = computed<WorkbenchTrendItem[]>(() => {
  const releaseItems = releases.value.slice(0, 3).map((item, index) => ({
    avatar: `svg:avatar-${(index % 4) + 1}`,
    content: `发布了 <a>${item.note}</a>`,
    date: formatTime(item.createdAt),
    title: '发布记录',
  }));
  const noticeItems = notices.value.slice(0, 3).map((item, index) => ({
    avatar: `svg:avatar-${((index + 1) % 4) + 1}`,
    content: `公告更新 <a>${item.title}</a>`,
    date: formatTime(item.CreatedAt),
    title: '系统公告',
  }));
  return [...releaseItems, ...noticeItems];
});

const todoItems = computed<WorkbenchTodoItem[]>(() => {
  const items: WorkbenchTodoItem[] = [];
  if (runtime.value) {
    items.push({
      completed: Boolean(runtime.value.redisEnabled),
      content: `当前底座：${runtime.value.os} / ${runtime.value.dbBackend} / Rust ${runtime.value.rustVersion}`,
      date: '现在',
      title: runtime.value.redisEnabled ? '运行环境已就绪' : '检查 Redis 接入状态',
    });
  }
  for (const item of notices.value.slice(0, 2)) {
    items.push({
      completed: false,
      content: item.content,
      date: formatTime(item.CreatedAt),
      title: item.title,
    });
  }
  for (const item of releases.value.slice(0, 2)) {
    items.push({
      completed: true,
      content: item.note,
      date: formatTime(item.createdAt),
      title: '发布记录已同步',
    });
  }
  return items.slice(0, 5);
});

const chartValues = computed(() => [12, 18, 24, 31, 44, 45, 34, 36, 78, 89, 92, 93]);

const chartPath = computed(() => {
  const points = chartValues.value;
  const width = 520;
  const height = 220;
  const max = Math.max(...points);
  const min = Math.min(...points);
  const range = Math.max(max - min, 1);

  return points
    .map((point, index) => {
      const x = (index / Math.max(points.length - 1, 1)) * width;
      const y = height - ((point - min) / range) * (height - 12) - 6;
      return `${index === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(' ');
});

const chartArea = computed(() => `${chartPath.value} L 520 220 L 0 220 Z`);

function formatTime(value: number | string) {
  const date = new Date(typeof value === 'number' ? value : Date.parse(value));
  if (Number.isNaN(date.getTime())) {
    return '-';
  }
  return date.toLocaleString('zh-CN', { hour12: false });
}

function pluginIcon(kind?: string, index = 0): string {
  if (kind?.includes('release')) return 'carbon:rocket';
  if (kind?.includes('service')) return 'carbon:ibm-cloud';
  if (kind?.includes('plugin')) return 'carbon:plugin';
  return (
    ['carbon:logo-github', 'carbon:application-web', 'carbon:data-base'] as string[]
  )[index % 3] ?? 'carbon:application-web';
}

function navTo(item: WorkbenchProjectItem | WorkbenchQuickNavItem) {
  if (!item.url) return;
  if (item.url.startsWith('http')) {
    openWindow(item.url);
    return;
  }
  router.push(item.url).catch(() => undefined);
}

async function reload() {
  loading.value = true;
  try {
    await auth.bootstrap();
    const [runtimeInfo, pluginPage, releasePage, noticePage] = await Promise.all([
      getRuntimeInfoApi(),
      getPluginInstallListApi(),
      getReleaseListApi(),
      getAnnouncementListApi(),
    ]);
    runtime.value = runtimeInfo;
    plugins.value = pluginPage.List ?? [];
    releases.value = releasePage.List ?? [];
    notices.value = noticePage.List ?? [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void reload();
});
</script>

<template>
  <Page title="仪表盘">
    <div class="dashboard-page relative flex h-full min-h-0 flex-1 flex-col">
      <WorkbenchHeader :avatar="userStore.userInfo?.avatar || preferences.app.defaultAvatar">
        <template #title>
          欢迎回来，开始今天的 Coding 节奏
        </template>
        <template #description>
          {{ today }} · 已为你聚合核心业务数据、插件动态和系统公告
        </template>
      </WorkbenchHeader>

      <div class="workspace-grid">
        <div class="workspace-main">
          <WorkbenchProject :items="projectItems" title="项目" @click="navTo" />
          <WorkbenchTrends :items="trendItems" class="mt-5" title="最新动态" />
        </div>

        <div class="workspace-side">
          <WorkbenchQuickNav
            :items="quickNavItems"
            title="快捷导航"
            @click="navTo"
          />
          <WorkbenchTodo :items="todoItems" class="mt-5" title="待办事项" />
          <AnalysisChartCard class="mt-5" title="内容数据">
            <div class="mini-chart">
              <svg viewBox="0 0 520 220" xmlns="http://www.w3.org/2000/svg">
                <defs>
                  <linearGradient id="workspace-chart-fill" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stop-color="rgba(22, 119, 255, 0.22)" />
                    <stop offset="100%" stop-color="rgba(22, 119, 255, 0.03)" />
                  </linearGradient>
                </defs>
                <path :d="chartArea" fill="url(#workspace-chart-fill)" />
                <path
                  :d="chartPath"
                  fill="none"
                  stroke="var(--ant-color-primary)"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="4"
                />
              </svg>
            </div>
          </AnalysisChartCard>
        </div>
      </div>
    </div>
  </Page>
</template>

<style scoped>
.dashboard-page {
  padding: 20px;
}

.workspace-grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
  margin-top: 20px;
}

.workspace-main,
.workspace-side {
  width: 100%;
}

.mini-chart {
  padding: 8px 4px 0;
}

.mini-chart svg {
  display: block;
  height: 220px;
  width: 100%;
}

@media (min-width: 1024px) {
  .workspace-grid {
    flex-direction: row;
  }

  .workspace-main {
    padding-right: 16px;
    width: 60%;
  }

  .workspace-side {
    width: 40%;
  }
}
</style>
