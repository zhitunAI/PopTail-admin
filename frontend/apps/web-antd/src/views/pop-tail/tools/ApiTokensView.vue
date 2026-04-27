<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { ApiTokenIssueInput, ApiTokenRecord } from '#/types/pop-tail';

import { nextTick, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Alert, Button, message, Modal, Space, Tag } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  clearApiTokenApi,
  getApiTokenCurlPayloadApi,
  getApiTokenListApi,
  invalidateApiTokenApi,
  issueApiTokenApi,
} from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface ApiTokenRow extends ApiTokenRecord {
  id: number;
  subjectText: string;
}

const ttlOptions = ['24h', '7d', '30d'].map((value) => ({
  label: value,
  value,
}));

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'keyword', label: '关键词' },
  {
    component: 'Select',
    componentProps: () => ({
      allowClear: true,
      options: statusFilters.value,
    }),
    fieldName: 'status',
    label: '状态',
  },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: ttlOptions,
    },
    fieldName: 'ttl',
    label: '有效期',
  },
];

const drawerSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: 'Token 名称', rules: 'required' },
  { component: 'Input', fieldName: 'scope', label: '作用域', rules: 'required' },
  {
    component: 'Select',
    componentProps: {
      options: ttlOptions,
    },
    fieldName: 'ttl',
    label: '有效期',
    rules: 'required',
  },
  { component: 'Input', fieldName: 'remark', label: '备注' },
];

const latestToken = ref<null | string>(null);
const statusFilters = ref<Array<{ label: string; value: string }>>([]);
const editingDraft = ref<ApiTokenIssueInput>({
  name: 'gateway-sync',
  remark: '',
  scope: 'system.audit.read',
  ttl: '24h',
});
const clearing = ref(false);
const { canUse: canUseMenuButton } = useMenuButtonAccess();

const [DrawerForm, drawerFormApi] = useVbenForm({
  schema: drawerSchema,
  showDefaultActions: false,
});

const [Drawer, drawerApi] = useVbenDrawer({
  async onConfirm() {
    const { valid } = await drawerFormApi.validate();
    if (!valid) return;
    drawerApi.lock();
    try {
      const values = await drawerFormApi.getValues<ApiTokenIssueInput>();
      const created = await issueApiTokenApi({
        name: values.name.trim(),
        remark: values.remark?.trim() || undefined,
        scope: values.scope.trim(),
        ttl: values.ttl,
      });
      latestToken.value = created.token ?? null;
      message.success(`已创建 Token：${created.name}`);
      drawerApi.close();
      await onRefresh();
    } finally {
      drawerApi.unlock();
    }
  },
  async onOpenChange(open) {
    if (!open) return;
    drawerFormApi.resetForm();
    await nextTick();
    drawerFormApi.setValues({
      name: editingDraft.value.name,
      remark: editingDraft.value.remark ?? '',
      scope: editingDraft.value.scope,
      ttl: editingDraft.value.ttl,
    });
  },
});

const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const [Grid, gridApi] = useVbenVxeGrid<ApiTokenRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: 'Token 名称', width: 180 },
      { field: 'scope', slots: { default: 'scope' }, title: '作用域', minWidth: 220 },
      { field: 'ttl', slots: { default: 'ttl' }, title: '有效期', width: 110 },
      { field: 'status', slots: { default: 'status' }, title: '状态', width: 120 },
      { field: 'subjectText', slots: { default: 'subjectText' }, title: '绑定主体', width: 180 },
      { field: 'expiresAt', slots: { default: 'expiresAt' }, title: '到期时间', width: 180 },
      { field: 'lastUsedAt', slots: { default: 'lastUsedAt' }, title: '最近使用', width: 180 },
      { field: 'remark', slots: { default: 'remark' }, minWidth: 220, title: '备注' },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 220,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const result = await getApiTokenListApi();
            const rows = (result.List ?? []).map((item) => ({
              ...item,
              id: item.ID,
              subjectText: subjectText(item),
            }));
            const statuses = [...new Set(rows.map((item) => item.status).filter(Boolean))].map((value) => ({
              label: value,
              value,
            }));
            statusFilters.value = statuses;

            const keyword = String(formValues.keyword ?? '').trim().toLowerCase();
            const status = String(formValues.status ?? '').trim();
            const ttl = String(formValues.ttl ?? '').trim();
            const filtered = rows.filter((item) => {
              if (keyword && !`${item.name} ${item.scope} ${item.remark ?? ''}`.toLowerCase().includes(keyword)) {
                return false;
              }
              if (status && item.status !== status) {
                return false;
              }
              if (ttl && item.ttl !== ttl) {
                return false;
              }
              return true;
            });

            const pageSize = page.pageSize;
            const start = (page.currentPage - 1) * pageSize;
            return {
              items: filtered.slice(start, start + pageSize),
              total: filtered.length,
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
  } as VxeTableGridOptions<ApiTokenRow>,
});

function openCreate() {
  latestToken.value = null;
  editingDraft.value = {
    name: 'gateway-sync',
    remark: '',
    scope: 'system.audit.read',
    ttl: '24h',
  };
  drawerApi.open();
}

function canUseApiTokenAction(action: 'clear' | 'copy' | 'issue') {
  if (action === 'issue') return canUseMenuButton(['签发', '签发JWT', '登记令牌', '新增', 'issue', 'create']);
  if (action === 'copy') return canUseMenuButton(['复制', '复制令牌摘要', 'Curl示例', 'copy']);
  return canUseMenuButton(['清空记录', '作废', '作废令牌', '删除', 'clear', 'delete', 'invalidate']);
}

function statusColor(status: string) {
  const lower = status.toLowerCase();
  if (['active', 'issued', 'ready'].includes(lower)) return 'success';
  if (lower.includes('pending')) return 'processing';
  return 'default';
}

function subjectText(row: ApiTokenRecord) {
  if (row.user?.nickName || row.user?.userName) {
    return `${row.user.nickName ?? row.user.userName} (#${row.user.ID})`;
  }
  if (row.userId) return `用户 #${row.userId}`;
  if (row.authorityId) return `角色 #${row.authorityId}`;
  return '-';
}

function formatDateTime(value?: null | number | string) {
  if (!value) return '-';
  if (typeof value === 'number') {
    const date = new Date(value > 1e12 ? value : value * 1000);
    return Number.isNaN(date.getTime()) ? '-' : date.toLocaleString();
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? String(value) : date.toLocaleString();
}

async function copyDigest(row: ApiTokenRow) {
  const text = `${row.name} | ${row.scope} | ${row.ttl} | ${row.status}`;
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    message.success('摘要已复制');
    return;
  }
  window.prompt('当前环境不支持自动复制，请手动复制：', text);
}

async function copyCurl(row: ApiTokenRow) {
  const payload = await getApiTokenCurlPayloadApi(row.ID);
  const text = payload.command || payload.header || payload.cookie || '';
  if (!text) {
    message.warning('当前 Token 没有可复制的 Curl 信息');
    return;
  }
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    message.success('Curl 示例已复制');
    return;
  }
  window.prompt('当前环境不支持自动复制，请手动复制：', text);
}

function onInvalidate(row: ApiTokenRow) {
  invalidateToken(row);
}

function invalidateToken(row: ApiTokenRow) {
  Modal.confirm({
    content: `确认作废 Token「${row.name}」吗？`,
    onOk: async () => {
      const hide = message.loading({
        content: `正在作废 ${row.name}`,
        duration: 0,
        key: `api-token-invalidate-${row.ID}`,
      });
      try {
        await invalidateApiTokenApi({ ID: row.ID });
        await onRefresh();
        message.success({
          content: 'Token 已作废',
          key: `api-token-invalidate-${row.ID}`,
        });
      } catch (error) {
        hide();
        message.error(error instanceof Error ? error.message : '作废 Token 失败');
        throw error;
      }
    },
    title: '作废 Token',
  });
}

function clearAllTokens() {
  Modal.confirm({
    content: '确认清空当前全部 Token 台账吗？',
    onOk: async () => {
      clearing.value = true;
      try {
        await clearApiTokenApi();
        latestToken.value = null;
        await onRefresh();
        message.success('Token 台账已清空');
      } finally {
        clearing.value = false;
      }
    },
    title: '清空 Token',
  });
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <div class="h-full min-h-0 p-4">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Alert v-if="latestToken" class="mb-4" message="最近一次创建的 Token 已返回完整值，请及时复制保存。" show-icon type="success" />

      <Grid class="h-full min-h-0 flex-1" table-title="Token 列表">
        <template #toolbar-tools>
          <Space>
            <Button v-if="canUseApiTokenAction('clear')" danger :loading="clearing" @click="clearAllTokens">
              清空记录
            </Button>
            <Button v-if="canUseApiTokenAction('issue')" type="primary" @click="openCreate">
              <IconifyIcon class="size-5" icon="mdi:plus" />
              新增 Token
            </Button>
          </Space>
        </template>

        <template #name="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="68%" />
          <span v-else>{{ row.name }}</span>
        </template>
        <template #scope="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
          <span v-else>{{ row.scope }}</span>
        </template>
        <template #ttl="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
          <span v-else>{{ row.ttl }}</span>
        </template>
        <template #status="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="64px" />
          <Tag v-else :color="statusColor(row.status)">{{ row.status }}</Tag>
        </template>
        <template #subjectText="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
          <span v-else>{{ row.subjectText }}</span>
        </template>
        <template #expiresAt="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="76%" />
          <span v-else>{{ formatDateTime(row.expiresAt) }}</span>
        </template>
        <template #lastUsedAt="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
          <span v-else>{{ formatDateTime(row.lastUsedAt) }}</span>
        </template>
        <template #remark="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="84%" />
          <span v-else>{{ row.remark || '-' }}</span>
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="170px" />
          <Space v-else>
            <Button v-if="canUseApiTokenAction('copy')" size="small" type="link"
              @click="copyDigest(row as ApiTokenRow)">摘要</Button>
            <Button v-if="canUseApiTokenAction('copy')" size="small" type="link"
              @click="copyCurl(row as ApiTokenRow)">Curl</Button>
            <Button v-if="canUseApiTokenAction('clear')" danger size="small" type="link"
              @click="onInvalidate(row as ApiTokenRow)">作废</Button>
          </Space>
        </template>
      </Grid>
    </div>

    <Drawer class="w-full max-w-160" title="新增 Token">
      <DrawerForm class="mx-4" layout="vertical" />
    </Drawer>
  </div>
</template>
