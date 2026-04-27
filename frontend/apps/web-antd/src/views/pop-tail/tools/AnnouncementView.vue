<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { AnnouncementAttachment, AnnouncementDataSource, AnnouncementRecord } from '#/types/pop-tail';

import { computed, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Alert, Button, Drawer, Form, Input, Modal, Select, Space, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createAnnouncementApi,
  deleteAnnouncementApi,
  getAnnouncementDataSourceApi,
  getAnnouncementListApi,
  updateAnnouncementApi,
} from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface AnnouncementRow extends AnnouncementRecord {
  attachmentCount: number;
  authorLabel: string;
  createdAtText: string;
  id: number;
}

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'title', label: '标题' },
  {
    component: 'Select',
    componentProps: () => ({
      allowClear: true,
      options: dataSource.value.userID,
    }),
    fieldName: 'userID',
    label: '作者',
  },
];

const dataSource = ref<AnnouncementDataSource>({ userID: [] });
const sourceRows = ref<AnnouncementRow[]>([]);
const drawerOpen = ref(false);
const editing = ref<AnnouncementRow | null>(null);
const { canUse: canUseMenuButton } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});
const form = ref({
  ID: undefined as number | undefined,
  attachmentsText: '',
  content: '',
  title: '',
  userID: 1,
});
const title = computed(() => (editing.value ? '编辑公告' : '新增公告'));
const showAuthorLoadWarning = ref(false);

const [Grid, gridApi] = useVbenVxeGrid<AnnouncementRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'title', slots: { default: 'title' }, title: '标题', minWidth: 260 },
      { field: 'authorLabel', slots: { default: 'authorLabel' }, title: '作者', width: 160 },
      { field: 'attachmentCount', slots: { default: 'attachmentCount' }, title: '附件数', width: 100 },
      { field: 'createdAtText', slots: { default: 'createdAtText' }, title: '创建时间', width: 180 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 140,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const list = await getAnnouncementListApi({
              page: page.currentPage,
              pageSize: page.pageSize,
              title: String(formValues.title ?? '').trim(),
            });

            try {
              dataSource.value = await getAnnouncementDataSourceApi();
              showAuthorLoadWarning.value = false;
            } catch {
              dataSource.value = { userID: [] };
              showAuthorLoadWarning.value = true;
            }

            const rows = (list.List ?? []).map((item) => ({
              ...item,
              attachmentCount: item.attachments.length,
              authorLabel: authorLabel(item.userID),
              createdAtText: formatTime(item.CreatedAt),
              id: item.ID,
            }));
            sourceRows.value = rows;
            const userID = Number(formValues.userID ?? 0);
            const filtered = rows.filter((item) => (!userID || item.userID === userID));
            return {
              items: filtered,
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
  } as VxeTableGridOptions<AnnouncementRow>,
});

function canCreate() {
  return canUseMenuButton(['新增公告', '新增', 'create']);
}

function canEdit() {
  return canUseMenuButton(['保存公告', '编辑公告', '更新公告', 'edit', 'update']);
}

function canDelete() {
  return canUseMenuButton(['删除', '删除公告', 'delete']);
}

function authorLabel(userID: number) {
  return dataSource.value.userID.find((item) => item.value === userID)?.label || `用户 ${userID}`;
}

function formatAttachmentsText(items: AnnouncementAttachment[]) {
  return items.map((item) => `${item.name} | ${item.url}`).join('\n');
}

function parseAttachmentsText(text: string) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [namePart, ...urlParts] = line.split('|');
      const url = (urlParts.join('|') || namePart || '').trim();
      const name = urlParts.length > 0 ? (namePart ?? '').trim() : inferAttachmentName(url);
      return { name: name || inferAttachmentName(url), url };
    })
    .filter((item) => item.url);
}

function inferAttachmentName(url: string) {
  const normalized = url.split(/[?#]/)[0] ?? '';
  const parts = normalized.split('/').map((item) => item.trim()).filter(Boolean);
  return parts.length > 0 ? decodeURIComponent(parts[parts.length - 1] || '附件') : '附件';
}

function openCreate() {
  form.value = {
    ID: undefined,
    attachmentsText: '',
    content: '',
    title: '',
    userID: dataSource.value.userID[0]?.value ?? 1,
  };
  editing.value = null;
  drawerOpen.value = true;
}

function openEdit(row: AnnouncementRow) {
  editing.value = row;
  form.value = {
    ID: row.ID,
    attachmentsText: formatAttachmentsText(row.attachments),
    content: row.content,
    title: row.title,
    userID: row.userID,
  };
  drawerOpen.value = true;
}

async function save() {
  if (!form.value.title.trim() || !form.value.content.trim()) {
    message.warning('请先补全标题和内容');
    return;
  }
  const payload = {
    ID: form.value.ID,
    attachments: parseAttachmentsText(form.value.attachmentsText),
    content: form.value.content.trim(),
    title: form.value.title.trim(),
    userID: form.value.userID,
  };
  if (editing.value) {
    await updateAnnouncementApi(payload);
    message.success('公告已更新');
  } else {
    await createAnnouncementApi(payload);
    message.success('公告已创建');
  }
  drawerOpen.value = false;
  await onRefresh();
}

function remove(row: AnnouncementRow) {
  Modal.confirm({
    content: `确认删除公告「${row.title}」吗？`,
    onOk: async () => {
      await deleteAnnouncementApi(row.ID);
      await onRefresh();
      message.success('公告已删除');
    },
    title: '删除公告',
  });
}

function formatTime(value: number) {
  return new Date(value).toLocaleString('zh-CN');
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="公告管理">
    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Alert
        v-if="showAuthorLoadWarning"
        class="mb-4"
        message="作者数据源加载失败，当前以用户 ID 展示作者。"
        show-icon
        type="warning"
      />

      <Grid class="h-full min-h-0 flex-1" table-title="公告列表">
        <template #toolbar-tools>
          <Button v-if="canCreate()" type="primary" @click="openCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            新增公告
          </Button>
        </template>
        <template #title="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="82%" />
          <span v-else>{{ row.title }}</span>
        </template>
        <template #authorLabel="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="60%" />
          <span v-else>{{ row.authorLabel }}</span>
        </template>
        <template #attachmentCount="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="40px" />
          <span v-else>{{ row.attachmentCount }}</span>
        </template>
        <template #createdAtText="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
          <span v-else>{{ row.createdAtText }}</span>
        </template>
        <template #operation="{ row }">
          <PageRefreshCellSkeleton v-if="pageLoading" width="120px" />
          <Space v-else>
            <Button v-if="canEdit()" size="small" type="link" @click="openEdit(row as AnnouncementRow)">编辑</Button>
            <Button v-if="canDelete()" danger size="small" type="link" @click="remove(row as AnnouncementRow)">删除</Button>
          </Space>
        </template>
      </Grid>
    </div>

    <Drawer v-model:open="drawerOpen" :title="title" width="720">
      <Form layout="vertical">
        <Form.Item label="标题">
          <Input v-model:value="form.title" />
        </Form.Item>
        <Form.Item label="作者">
          <Select v-model:value="form.userID" :options="dataSource.userID" />
        </Form.Item>
        <Form.Item label="内容">
          <Input.TextArea v-model:value="form.content" :rows="8" />
        </Form.Item>
        <Form.Item label="附件">
          <Input.TextArea
            v-model:value="form.attachmentsText"
            :rows="6"
            placeholder="每行一条，格式：附件名 | 链接"
          />
        </Form.Item>
        <div class="flex justify-end">
          <Button v-if="canEdit() || canCreate()" type="primary" @click="save">保存</Button>
        </div>
      </Form>
    </Drawer>
  </Page>
</template>
