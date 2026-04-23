<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { OperationLogInfo } from '#/types/gin-ai-admin';

import { Page } from '@vben/common-ui';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteOperationLogApi,
  getOperationLogsApi,
} from '#/api/gin-ai-admin/admin';

interface OperationLogRow extends OperationLogInfo {
  id: number;
}

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
      { field: 'ID', title: 'ID', width: 90 },
      { field: 'method', slots: { default: 'method' }, title: '方法', width: 100 },
      { field: 'path', minWidth: 260, title: '路径' },
      { field: 'ip', title: '来源 IP', width: 160 },
      { field: 'status', slots: { default: 'status' }, title: '状态码', width: 110 },
      { field: 'latency', title: '耗时(ms)', width: 120 },
      { field: 'errorMessage', minWidth: 220, title: '错误信息' },
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
          const result = await getOperationLogsApi({
            page: page.currentPage,
            pageSize: page.pageSize,
            ...formValues,
          });
          return {
            items: (result.List ?? []).map((item) => ({ ...item, id: item.ID })),
            total: result.Total,
          };
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
    <Grid table-title="操作日志列表">
      <template #method="{ row }">
        <Tag :color="methodColor(row.method)">{{ row.method }}</Tag>
      </template>
      <template #status="{ row }">
        <Tag :color="statusColor(row.status)">{{ row.status }}</Tag>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button danger size="small" type="link" @click="onDelete(row as OperationLogRow)">
            删除
          </Button>
        </Space>
      </template>
    </Grid>
  </Page>
</template>
