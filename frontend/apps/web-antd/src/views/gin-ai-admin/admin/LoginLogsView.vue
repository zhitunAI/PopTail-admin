<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { LoginLogInfo } from '#/types/gin-ai-admin';

import { Page } from '@vben/common-ui';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteLoginLogApi,
  getLoginLogsApi,
} from '#/api/gin-ai-admin/admin';

interface LoginLogRow extends LoginLogInfo {
  id: number;
}

const formSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'username', label: '用户名' },
  { component: 'Input', fieldName: 'ip', label: '来源 IP' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '成功', value: true },
        { label: '失败', value: false },
      ],
    },
    fieldName: 'status',
    label: '状态',
  },
  { component: 'RangePicker', fieldName: 'createdAt', label: '创建时间' },
];

const [Grid, gridApi] = useVbenVxeGrid<LoginLogRow>({
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
      { field: 'username', title: '用户名', width: 160 },
      { field: 'ip', title: '来源 IP', width: 160 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 100 },
      { field: 'errorMessage', minWidth: 260, title: '结果说明' },
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
          const result = await getLoginLogsApi({
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
  } as VxeTableGridOptions<LoginLogRow>,
});

function onDelete(row: LoginLogRow) {
  Modal.confirm({
    content: `确认删除登录日志「${row.username || row.ID}」吗？`,
    onOk: async () => {
      await deleteLoginLogApi({ ID: row.ID });
      message.success('登录日志已删除');
      gridApi.query();
    },
    title: '删除登录日志',
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="登录日志列表">
      <template #status="{ row }">
        <Tag :color="row.status ? 'success' : 'error'">
          {{ row.status ? '成功' : '失败' }}
        </Tag>
      </template>
      <template #operation="{ row }">
        <Space size="small">
          <Button danger size="small" type="link" @click="onDelete(row as LoginLogRow)">
            删除
          </Button>
        </Space>
      </template>
    </Grid>
  </Page>
</template>
