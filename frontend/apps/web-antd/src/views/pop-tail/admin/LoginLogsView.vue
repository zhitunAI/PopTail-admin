<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { LoginLogInfo } from '#/types/pop-tail';

import { Page } from '@vben/common-ui';

import { Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import {
  deleteLoginLogApi,
  getLoginLogsApi,
} from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface LoginLogRow extends LoginLogInfo {
  id: number;
}

const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await gridApi.query();
});

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
      { field: 'ID', slots: { default: 'id' }, title: 'ID', width: 90 },
      { field: 'username', slots: { default: 'username' }, title: '用户名', width: 160 },
      { field: 'ip', slots: { default: 'ip' }, title: '来源 IP', width: 160 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 100 },
      { field: 'errorMessage', slots: { default: 'errorMessage' }, minWidth: 260, title: '结果说明' },
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
            const result = await getLoginLogsApi({
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
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Grid class="h-full min-h-0 flex-1" table-title="登录日志列表">
      <template #id="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="40px" />
        <span v-else>{{ row.ID }}</span>
      </template>
      <template #username="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="62%" />
        <span v-else>{{ row.username }}</span>
      </template>
      <template #ip="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="58%" />
        <span v-else>{{ row.ip }}</span>
      </template>
      <template #status="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
        <Tag v-else :color="row.status ? 'success' : 'error'">
          {{ row.status ? '成功' : '失败' }}
        </Tag>
      </template>
      <template #errorMessage="{ row }">
        <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
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
            @click="onDelete(row as LoginLogRow)"
          >
            删除
          </Button>
        </Space>
      </template>
      </Grid>
    </div>
  </Page>
</template>
