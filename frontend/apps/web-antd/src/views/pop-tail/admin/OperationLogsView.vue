<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { OperationLogInfo } from '#/types/pop-tail';

import { Page } from '@vben/common-ui';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteOperationLogApi,
  getOperationLogsApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface OperationLogRow extends OperationLogInfo {
  id: number;
}

const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await gridApi.query();
});

const methodOptions = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'].map((value) => ({ label: value, value }));

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'path', label: '请求路径' },
  { component: 'Input', fieldName: 'ip', label: '来源 IP' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: methodOptions,
    },
    fieldName: 'method',
    label: '方法',
  },
  { component: 'RangePicker', fieldName: 'createdAt', label: '创建时间' },
];

const [Grid, gridApi] = useVbenVxeGrid<OperationLogRow>({
  formOptions: {
    collapsed: false,
    fieldMappingTime: [['createdAt', ['startCreatedAt', 'endCreatedAt']]],
    schema: formSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'ID', slots: { default: 'id' }, title: 'ID', width: 90 },
      { field: 'method', slots: { default: 'method' }, title: '方法', width: 100 },
      { field: 'path', slots: { default: 'path' }, minWidth: 260, title: '路径' },
      { field: 'ip', slots: { default: 'ip' }, title: '来源 IP', width: 160 },
      { field: 'status', slots: { default: 'status' }, title: '状态码', width: 110 },
      { field: 'latency', slots: { default: 'latency' }, title: '耗时(ms)', width: 120 },
      { field: 'errorMessage', slots: { default: 'errorMessage' }, minWidth: 220, title: '错误信息' },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 100,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const result = await getOperationLogsApi({
              page: page.currentPage,
              pageSize: page.pageSize,
              ...formValues,
            });
            return {
              items: (result.List ?? []).map((item) => ({ ...item, id: item.ID })),
              total: result.Total,
            };
          } finally {
            pageLoading.value = false;
          }
        },
      },
    },
    rowConfig: {
      keyField: 'id',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: true,
      search: true,
      zoom: true,
    },
  } as VxeTableGridOptions<OperationLogRow>,
});

function methodColor(method: string) {
  if (method === 'GET') return 'success';
  if (method === 'DELETE') return 'error';
  if (method === 'PUT' || method === 'PATCH') return 'warning';
  return 'processing';
}

function statusColor(status: number) {
  if (status >= 500) return 'error';
  if (status >= 400) return 'warning';
  if (status >= 300) return 'processing';
  return 'success';
}

function onDelete(row: OperationLogRow) {
  Modal.confirm({
    content: `确认删除操作日志「${row.ID}」吗？`,
    onOk: async () => {
      await deleteOperationLogApi({ ID: row.ID });
      message.success('操作日志已删除');
      gridApi.query();
    },
    title: '删除操作日志',
  });
}
</script>

<template>
  <Page auto-content-height>
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="操作日志列表">
      <template #id="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="40px" />
        <span v-else>{{ row.ID }}</span>
      </template>
      <template #method="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
        <Tag v-else :color="methodColor(row.method)">{{ row.method }}</Tag>
      </template>
      <template #path="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
        <span v-else>{{ row.path }}</span>
      </template>
      <template #ip="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="60%" />
        <span v-else>{{ row.ip }}</span>
      </template>
      <template #status="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="58px" />
        <Tag v-else :color="statusColor(row.status)">{{ row.status }}</Tag>
      </template>
      <template #latency="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="52px" />
        <span v-else>{{ row.latency }}</span>
      </template>
      <template #errorMessage="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
        <span v-else>{{ row.errorMessage }}</span>
      </template>
      <template #operation="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="60px" />
        <Space v-else size="small">
          <Button
            v-if="canUse(STANDARD_BUTTON_LABELS.delete)"
            danger
            size="small"
            type="link"
            @click="onDelete(row as OperationLogRow)"
          >
            删除
          </Button>
        </Space>
      </template>
      </Grid>
    </div>
  </Page>
</template>
