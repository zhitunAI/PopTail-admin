<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';

import { ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

type ToolEntry = {
  desc: string;
  name: string;
  path: string;
};

const router = useRouter();
const sourceRows = ref<ToolEntry[]>([]);
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '模块' },
  { component: 'Input', fieldName: 'desc', label: '说明' },
];

const [Grid, gridApi] = useVbenVxeGrid<ToolEntry>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: '模块', width: 180 },
      { field: 'desc', slots: { default: 'desc' }, title: '说明', minWidth: 320 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '入口',
        width: 120,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const rows: ToolEntry[] = [
              { desc: '设置系统内部 AI 工作流，例如自动审核注册用户、自动审核用户发帖。', name: 'AI 工作流', path: '/system/tools/ai-workflow' },
              { desc: '系统对外 API 对接预留，用于签发和管理 API Token。', name: 'API Token', path: '/system/tools/api-tokens' },
              { desc: '添加 OpenAI、Qwen、Claude 等模型配置和调用地址。', name: '大模型配置', path: '/system/tools/llm-config' },
              { desc: '管理 skills 模块、说明文档和相关资产。', name: '技能管理', path: '/system/tools/skills' },
              { desc: '内部系统配置，按 tabs 分类维护。', name: '系统配置', path: '/system/tools/config' },
              { desc: '邮件配置、测试发送、正式收发记录。', name: '邮件管理', path: '/system/tools/plugin-email' },
              { desc: '公告内容、公告投放与附件维护。', name: '公告管理', path: '/system/tools/announcement' },
            ];
            sourceRows.value = rows;
            const name = String(formValues.name ?? '').trim().toLowerCase();
            const desc = String(formValues.desc ?? '').trim().toLowerCase();
            const filtered = rows.filter((item) =>
              (!name || item.name.toLowerCase().includes(name)) &&
              (!desc || item.desc.toLowerCase().includes(desc)),
            );
            const start = (page.currentPage - 1) * page.pageSize;
            return {
              items: filtered.slice(start, start + page.pageSize),
              total: filtered.length,
            };
          } finally {
            pageLoading.value = false;
          }
        },
      },
    },
    rowConfig: {
      keyField: 'path',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: true,
      search: true,
      zoom: true,
    },
  } as VxeTableGridOptions<ToolEntry>,
});

function navigate(path: string) {
  router.push(path).catch(() => undefined);
}

function openFirst() {
  const first = sourceRows.value[0];
  if (first) navigate(first.path);
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="系统工具">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="工具列表">
        <template #toolbar-tools>
          <Button @click="openFirst">
            <template #icon>
              <IconifyIcon icon="mdi:launch" />
            </template>
            打开首项
          </Button>
        </template>
        <template #name="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="62%" />
          <span v-else>{{ row.name }}</span>
        </template>
        <template #desc="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="86%" />
          <span v-else>{{ row.desc }}</span>
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="60px" />
          <Button v-else size="small" type="link" @click="navigate((row as ToolEntry).path)">打开</Button>
        </template>
      </Grid>
    </div>
  </Page>
</template>
