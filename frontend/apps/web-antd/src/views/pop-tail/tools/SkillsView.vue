<script lang="ts" setup>
import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridOptions } from '#/adapter/vxe-table';
import type { SkillAssetRecord, SkillRecord, SkillSummary, SkillToolDefinition } from '#/types/pop-tail';

import { computed, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Alert, Button, Drawer, Form, Input, Modal, Select, Space, Switch, Tabs, TabPane, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  createSkillAssetApi,
  deleteSkillApi,
  getGlobalConstraintApi,
  getSkillAssetApi,
  getSkillDetailApi,
  getSkillListApi,
  getSkillToolsApi,
  saveGlobalConstraintApi,
  saveSkillApi,
  saveSkillAssetApi,
} from '#/api/pop-tail/admin';
import PageRefreshCellSkeleton from '#/components/page-refresh-cell-skeleton.vue';
import { useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

const assetKinds = ['script', 'resource', 'reference', 'template'] as const;
const assetKindLabels = {
  script: '脚本',
  resource: '资源',
  reference: '参考',
  template: '模板',
} as const;

type SkillRow = SkillSummary & {
  id: string;
  tagsText: string;
  updatedAtText: string;
};

const filterSchema: VbenFormSchema[] = [
  { component: 'Input', fieldName: 'name', label: '技能名称' },
  { component: 'Input', fieldName: 'description', label: '描述' },
  {
    component: 'Select',
    componentProps: {
      allowClear: true,
      options: [
        { label: '启用', value: true },
        { label: '停用', value: false },
      ],
    },
    fieldName: 'enabled',
    label: '状态',
  },
];

const drawerOpen = ref(false);
const activeTab = ref('list');
const sourceRows = ref<SkillRow[]>([]);
const tools = ref<SkillToolDefinition[]>([]);
const currentSkill = ref<null | SkillRecord>(null);
const currentSkillName = ref('');
const globalConstraint = ref('');
const activeAssetKind = ref<(typeof assetKinds)[number]>('script');
const selectedAssetName = ref('');
const newAssetName = ref('');
const assetContent = ref('');
const savingSkill = ref(false);
const savingAsset = ref(false);
const savingConstraint = ref(false);
const { canUse } = useMenuButtonAccess();
const { pageLoading } = usePageRefreshLoading(async () => {
  await onRefresh();
});

const draft = reactive({
  name: '',
  description: '',
  allowedTools: '',
  context: '',
  agent: '',
  markdown: '',
  enabled: true,
  tags: '',
});

const currentAssets = computed<SkillAssetRecord[]>(() => {
  if (!currentSkill.value) return [];
  switch (activeAssetKind.value) {
    case 'reference':
      return currentSkill.value.references;
    case 'resource':
      return currentSkill.value.resources;
    case 'template':
      return currentSkill.value.templates;
    default:
      return currentSkill.value.scripts;
  }
});

const skillAdvice = computed(() => {
  if (!currentSkill.value) return '请选择技能后再编辑约束、Markdown 或资产。';
  if (!currentSkill.value.enabled) return '当前技能处于停用态，建议确认是否仍需保留。';
  if (!currentAssets.value.length) return '当前技能没有同类资产，建议补充脚本、资源或参考。';
  return '技能结构较完整，可继续维护约束与资产内容。';
});

const [Grid, gridApi] = useVbenVxeGrid<SkillRow>({
  formOptions: {
    collapsed: false,
    schema: filterSchema,
    showCollapseButton: true,
    submitOnChange: true,
  },
  gridOptions: {
    columns: [
      { field: 'name', slots: { default: 'name' }, title: '技能名称', width: 180 },
      { field: 'description', slots: { default: 'description' }, title: '描述', minWidth: 260 },
      { field: 'tagsText', slots: { default: 'tagsText' }, title: '标签', minWidth: 180 },
      { field: 'enabled', slots: { default: 'enabled' }, title: '状态', width: 120 },
      { field: 'updatedAtText', slots: { default: 'updatedAtText' }, title: '更新时间', width: 180 },
      {
        align: 'center',
        field: 'operation',
        fixed: 'right',
        slots: { default: 'operation' },
        title: '操作',
        width: 200,
      },
    ],
    height: 'auto',
    keepSource: true,
    proxyConfig: {
      ajax: {
        query: async ({ page }, formValues) => {
          pageLoading.value = true;
          try {
            const [skillRows, toolRows, constraint] = await Promise.all([
              getSkillListApi(),
              getSkillToolsApi(),
              getGlobalConstraintApi(),
            ]);
            tools.value = toolRows;
            globalConstraint.value = constraint.content;
            const rows = skillRows
              .map((item) => ({
                ...item,
                id: item.name,
                tagsText: item.tags.join(', ') || '-',
                updatedAtText: formatTime(item.updatedAt),
              }))
              .toSorted((a, b) => b.updatedAt - a.updatedAt);
            sourceRows.value = rows;
            const name = String(formValues.name ?? '').trim().toLowerCase();
            const description = String(formValues.description ?? '').trim().toLowerCase();
            const hasEnabled = typeof formValues.enabled === 'boolean';
            const filtered = rows.filter((item) =>
              (!name || item.name.toLowerCase().includes(name)) &&
              (!description || item.description.toLowerCase().includes(description)) &&
              (!hasEnabled || item.enabled === formValues.enabled),
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
      keyField: 'id',
    },
    toolbarConfig: {
      custom: true,
      export: false,
      refresh: true,
      search: true,
      zoom: true,
    },
  } as VxeTableGridOptions<SkillRow>,
});

function resetDraft() {
  draft.name = '';
  draft.description = '';
  draft.allowedTools = '';
  draft.context = '';
  draft.agent = '';
  draft.markdown = '## Instructions\n- 描述使用场景\n';
  draft.enabled = true;
  draft.tags = '';
}

function openCreate() {
  currentSkill.value = null;
  currentSkillName.value = '';
  resetDraft();
  drawerOpen.value = true;
}

async function openEdit(row: SkillRow) {
  const detail = await getSkillDetailApi(row.name);
  applySkillToDraft(detail);
  currentSkill.value = detail;
  currentSkillName.value = detail.name;
  drawerOpen.value = true;
}

async function openWorkbench(row: SkillRow) {
  await selectSkill(row.name);
  activeTab.value = 'workbench';
}

function applySkillToDraft(skill: SkillRecord) {
  draft.name = skill.name;
  draft.description = skill.description;
  draft.allowedTools = skill.allowedTools;
  draft.context = skill.context;
  draft.agent = skill.agent;
  draft.markdown = skill.markdown;
  draft.enabled = skill.enabled;
  draft.tags = skill.tags.join(', ');
}

async function saveSkill() {
  if (!draft.name.trim() || !draft.description.trim() || !draft.markdown.trim()) {
    message.warning('请先补全技能名称、描述和 Markdown');
    return;
  }
  savingSkill.value = true;
  try {
    const saved = await saveSkillApi({
      name: draft.name.trim(),
      description: draft.description.trim(),
      allowedTools: draft.allowedTools.trim(),
      context: draft.context.trim(),
      agent: draft.agent.trim(),
      markdown: draft.markdown,
      enabled: draft.enabled,
      tags: draft.tags.split(',').map((item) => item.trim()).filter(Boolean),
    });
    currentSkill.value = saved;
    currentSkillName.value = saved.name;
    drawerOpen.value = false;
    await onRefresh();
    message.success(`技能 ${saved.name} 已保存`);
  } finally {
    savingSkill.value = false;
  }
}

function removeSkill(row: SkillRow) {
  Modal.confirm({
    content: `确认删除技能「${row.name}」吗？`,
    onOk: async () => {
      await deleteSkillApi(row.name);
      if (currentSkillName.value === row.name) {
        currentSkill.value = null;
        currentSkillName.value = '';
        selectedAssetName.value = '';
        assetContent.value = '';
      }
      await onRefresh();
      message.success(`技能 ${row.name} 已删除`);
    },
    title: '删除技能',
  });
}

async function selectSkill(name: string) {
  const detail = await getSkillDetailApi(name);
  currentSkill.value = detail;
  currentSkillName.value = name;
  applySkillToDraft(detail);
  selectedAssetName.value = currentAssets.value[0]?.name || '';
  await loadAssetContent();
}

async function saveConstraint() {
  savingConstraint.value = true;
  try {
    const saved = await saveGlobalConstraintApi(globalConstraint.value);
    globalConstraint.value = saved.content;
    message.success('全局约束已保存');
  } finally {
    savingConstraint.value = false;
  }
}

async function switchAssetKind(kind: (typeof assetKinds)[number]) {
  activeAssetKind.value = kind;
  selectedAssetName.value = currentAssets.value[0]?.name || '';
  await loadAssetContent();
}

async function loadAssetContent() {
  if (!currentSkillName.value || !selectedAssetName.value) {
    assetContent.value = '';
    return;
  }
  const file = await getSkillAssetApi(activeAssetKind.value, {
    skillName: currentSkillName.value,
    name: selectedAssetName.value,
  });
  assetContent.value = file.content;
}

async function createAsset() {
  if (!currentSkillName.value || !newAssetName.value.trim()) {
    message.warning('请先选择技能并填写文件名');
    return;
  }
  await createSkillAssetApi(activeAssetKind.value, {
    skillName: currentSkillName.value,
    name: newAssetName.value.trim(),
  });
  await selectSkill(currentSkillName.value);
  selectedAssetName.value = newAssetName.value.trim();
  newAssetName.value = '';
  await loadAssetContent();
  message.success('资产已创建');
}

async function saveAsset() {
  if (!currentSkillName.value || !selectedAssetName.value) {
    message.warning('请先选择资产文件');
    return;
  }
  savingAsset.value = true;
  try {
    await saveSkillAssetApi(activeAssetKind.value, {
      skillName: currentSkillName.value,
      name: selectedAssetName.value,
      content: assetContent.value,
    });
    await selectSkill(currentSkillName.value);
    message.success('资产已保存');
  } finally {
    savingAsset.value = false;
  }
}

function formatTime(value: number) {
  return new Date(value).toLocaleString('zh-CN');
}

async function onRefresh() {
  await gridApi.query();
}
</script>

<template>
  <Page auto-content-height title="技能管理">
    <Tabs v-model:activeKey="activeTab" class="flex h-full min-h-0 flex-1 flex-col">
      <TabPane key="list" tab="技能列表" class="h-full min-h-0">
        <div class="flex h-full min-h-0 flex-1 flex-col">
          <Grid class="h-full min-h-0 flex-1" table-title="技能列表">
            <template #toolbar-tools>
              <Space>
                <Button type="primary" @click="openCreate">
                  <IconifyIcon class="size-5" icon="mdi:plus" />
                  新增技能
                </Button>
              </Space>
            </template>
            <template #name="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="68%" />
              <span v-else>{{ row.name }}</span>
            </template>
            <template #description="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="84%" />
              <span v-else>{{ row.description }}</span>
            </template>
            <template #tagsText="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="72%" />
              <span v-else>{{ row.tagsText }}</span>
            </template>
            <template #enabled="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="56px" />
              <Tag v-else :color="row.enabled ? 'success' : 'default'">{{ row.enabled ? '启用' : '停用' }}</Tag>
            </template>
            <template #updatedAtText="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="74%" />
              <span v-else>{{ row.updatedAtText }}</span>
            </template>
            <template #operation="{ row }">
              <PageRefreshCellSkeleton v-if="pageLoading" width="170px" />
              <Space v-else>
                <Button size="small" type="link" @click="openWorkbench(row as SkillRow)">工作台</Button>
                <Button v-if="canUse(['保存技能', '编辑'])" size="small" type="link" @click="openEdit(row as SkillRow)">编辑</Button>
                <Button v-if="canUse(['删除', '删除技能'])" danger size="small" type="link" @click="removeSkill(row as SkillRow)">删除</Button>
              </Space>
            </template>
          </Grid>
        </div>
      </TabPane>

      <TabPane key="workbench" tab="技能工作台">
        <div class="grid gap-4 xl:grid-cols-[1.2fr_1fr]">
          <div class="rounded-xl border border-[var(--ant-color-border)] p-4">
            <div class="mb-3 flex items-center justify-between gap-3">
              <div class="flex items-center gap-2">
                <IconifyIcon class="text-lg" icon="lucide:brain-circuit" />
                <span class="font-medium">全局约束</span>
              </div>
              <Button :loading="savingConstraint" @click="saveConstraint">保存约束</Button>
            </div>
            <Input.TextArea v-model:value="globalConstraint" :rows="8" />
            <div class="mt-3 flex flex-wrap gap-2">
              <Tag v-for="tool in tools" :key="tool.key">{{ tool.label }} · {{ tool.summary }}</Tag>
            </div>
          </div>

          <div class="rounded-xl border border-[var(--ant-color-border)] p-4">
            <div class="mb-3 flex items-center justify-between gap-3">
              <span class="font-medium">当前技能</span>
              <Button v-if="currentSkillName" @click="drawerOpen = true">编辑技能</Button>
            </div>
            <Alert v-if="currentSkillName" class="mb-3" type="info" show-icon :message="currentSkillName" />
            <div class="text-sm text-[var(--ant-color-text-secondary)]">{{ skillAdvice }}</div>
          </div>

          <div class="rounded-xl border border-[var(--ant-color-border)] p-4 xl:col-span-2">
            <div class="mb-4 flex flex-wrap items-center gap-2">
              <Button
                v-for="kind in assetKinds"
                :key="kind"
                :type="activeAssetKind === kind ? 'primary' : 'default'"
                @click="switchAssetKind(kind)"
              >
                {{ assetKindLabels[kind] }}
              </Button>
            </div>
            <div class="mb-4 grid gap-3 md:grid-cols-[1fr_1fr_auto]">
              <Input v-model:value="newAssetName" placeholder="新文件名，例如 check-gap.sh" />
              <Select v-model:value="selectedAssetName" placeholder="选择文件" @change="loadAssetContent">
                <Select.Option v-for="file in currentAssets" :key="file.name" :value="file.name">{{ file.name }}</Select.Option>
              </Select>
              <Space>
                <Button @click="createAsset">创建文件</Button>
                <Button type="primary" :loading="savingAsset" @click="saveAsset">保存文件</Button>
              </Space>
            </div>
            <Input.TextArea v-model:value="assetContent" :rows="16" />
          </div>
        </div>
      </TabPane>
    </Tabs>

    <Drawer v-model:open="drawerOpen" :title="currentSkillName ? '编辑技能' : '新增技能'" width="720">
      <Form layout="vertical">
        <div class="grid grid-cols-2 gap-4">
          <Form.Item label="技能名称"><Input v-model:value="draft.name" /></Form.Item>
          <Form.Item label="标签"><Input v-model:value="draft.tags" placeholder="使用逗号分隔" /></Form.Item>
          <Form.Item label="描述"><Input v-model:value="draft.description" /></Form.Item>
          <Form.Item label="Allowed Tools"><Input v-model:value="draft.allowedTools" /></Form.Item>
          <Form.Item label="Context"><Input v-model:value="draft.context" /></Form.Item>
          <Form.Item label="Agent"><Input v-model:value="draft.agent" /></Form.Item>
        </div>
        <Form.Item label="Markdown">
          <Input.TextArea v-model:value="draft.markdown" :rows="14" />
        </Form.Item>
        <Form.Item label="启用">
          <Switch v-model:checked="draft.enabled" />
        </Form.Item>
        <div class="flex justify-end">
          <Button type="primary" :loading="savingSkill" @click="saveSkill">保存技能</Button>
        </div>
      </Form>
    </Drawer>
  </Page>
</template>

<style scoped>
:deep(.ant-tabs-content-holder) {
  display: flex;
  flex: 1;
  min-height: 0;
}

:deep(.ant-tabs-content) {
  min-height: 0;
  height: 100%;
}

:deep(.ant-tabs-tabpane) {
  min-height: 0;
}
</style>
