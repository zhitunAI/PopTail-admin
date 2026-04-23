<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Drawer, Form, Input, Select, Space, Table, TableColumn, Tabs, TabPane, Tag, message } from 'ant-design-vue';

import { deleteLlmConfigApi, getLlmConfigListApi, saveLlmConfigApi } from '#/api/gin-ai-admin/admin';

type LlmProvider = 'claude' | 'openai' | 'qwen';
type LlmConfig = {
  apiKey: string;
  baseUrl: string;
  id: number;
  model: string;
  provider: LlmProvider;
  status: 'disabled' | 'enabled';
};

const drawerOpen = ref(false);
const editingId = ref<number | null>(null);
const records = ref<LlmConfig[]>([]);

const form = ref<LlmConfig>({
  id: 0,
  apiKey: '',
  baseUrl: '',
  model: '',
  provider: 'openai',
  status: 'enabled',
});

const providerOptions = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Qwen', value: 'qwen' },
  { label: 'Claude', value: 'claude' },
];

async function loadConfigs() {
  const page = await getLlmConfigListApi();
  records.value = page.List.map((item) => ({
    apiKey: item.apiKey,
    baseUrl: item.baseUrl,
    id: item.ID,
    model: item.model,
    provider: item.provider as LlmProvider,
    status: item.status as LlmConfig['status'],
  }));
}

function openCreate() {
  editingId.value = null;
  form.value = { id: 0, apiKey: '', baseUrl: '', model: '', provider: 'openai', status: 'enabled' };
  drawerOpen.value = true;
}

function openEdit(record: LlmConfig) {
  editingId.value = record.id;
  form.value = { ...record };
  drawerOpen.value = true;
}

async function save() {
  if (!form.value.model.trim() || !form.value.baseUrl.trim()) {
    message.warning('请先补全模型名和地址');
    return;
  }
  await saveLlmConfigApi({
    ID: editingId.value ?? undefined,
    apiKey: form.value.apiKey,
    baseUrl: form.value.baseUrl,
    model: form.value.model,
    provider: form.value.provider,
    status: form.value.status,
  });
  await loadConfigs();
  drawerOpen.value = false;
  message.success('模型配置已保存');
}

async function remove(id: number) {
  await deleteLlmConfigApi(id);
  if (editingId.value === id) editingId.value = null;
  await loadConfigs();
  message.success('模型配置已删除');
}

onMounted(() => {
  void loadConfigs();
});
</script>

<template>
  <Page auto-content-height title="大模型配置">
    <Tabs>
      <TabPane key="models" tab="模型列表">
        <div class="mb-4 flex justify-end">
          <Button type="primary" @click="openCreate">
            <IconifyIcon class="size-5" icon="mdi:plus" />
            添加模型
          </Button>
        </div>
        <Table :data-source="records" :pagination="false" row-key="id" :scroll="{ x: 900 }">
          <TableColumn data-index="provider" key="provider" title="供应商" />
          <TableColumn data-index="model" key="model" title="模型" />
          <TableColumn data-index="baseUrl" key="baseUrl" title="API 地址" />
          <TableColumn key="status" title="状态">
            <template #default="{ record }">
              <Tag :color="record.status === 'enabled' ? 'success' : 'default'">{{ record.status }}</Tag>
            </template>
          </TableColumn>
          <TableColumn fixed="right" key="actions" title="操作" width="140">
            <template #default="{ record }">
              <Space>
                <Button size="small" type="link" @click="openEdit(record)">编辑</Button>
                <Button danger size="small" type="link" @click="remove(record.id)">删除</Button>
              </Space>
            </template>
          </TableColumn>
        </Table>
      </TabPane>
      <TabPane key="providers" tab="接入说明">
        <div class="rounded-lg border border-[var(--ant-color-border)] p-4 text-sm text-[var(--ant-color-text-secondary)]">
          支持录入 `OpenAI / Qwen / Claude` 等大模型的 API 地址、模型名、密钥与启停状态。保存后可供 AI 工作流、自动审核、内容审核等内部流程引用。
        </div>
      </TabPane>
    </Tabs>

    <Drawer v-model:open="drawerOpen" :title="editingId == null ? '新增模型' : '编辑模型'" width="520">
      <Form layout="vertical">
        <Form.Item label="供应商">
          <Select v-model:value="form.provider" :options="providerOptions" />
        </Form.Item>
        <Form.Item label="模型名称">
          <Input v-model:value="form.model" placeholder="例如 gpt-4.1 / qwen-max / claude-sonnet-4" />
        </Form.Item>
        <Form.Item label="API 地址">
          <Input v-model:value="form.baseUrl" placeholder="例如 https://api.openai.com/v1" />
        </Form.Item>
        <Form.Item label="API Key">
          <Input v-model:value="form.apiKey" placeholder="请输入 API Key" />
        </Form.Item>
        <Form.Item label="状态">
          <Select v-model:value="form.status" :options="[{ label: '启用', value: 'enabled' }, { label: '禁用', value: 'disabled' }]" />
        </Form.Item>
        <div class="flex justify-end">
          <Button type="primary" @click="save">保存</Button>
        </div>
      </Form>
    </Drawer>
  </Page>
</template>
