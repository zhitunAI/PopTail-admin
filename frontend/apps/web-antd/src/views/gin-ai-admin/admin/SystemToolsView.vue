<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';

import { Button, Table, TableColumn } from 'ant-design-vue';

type ToolEntry = {
  desc: string;
  name: string;
  path: string;
};

const router = useRouter();
const rows = ref<ToolEntry[]>([]);

function navigate(path: string) {
  router.push(path).catch(() => undefined);
}

function reload() {
  rows.value = [
    {
      desc: '设置系统内部 AI 工作流，例如自动审核注册用户、自动审核用户发帖。',
      name: 'AI 工作流',
      path: '/system/tools/ai-workflow',
    },
    {
      desc: '系统对外 API 对接预留，用于签发和管理 API Token。',
      name: 'API Token',
      path: '/system/tools/api-tokens',
    },
    {
      desc: '添加 OpenAI、Qwen、Claude 等模型配置和调用地址。',
      name: '大模型配置',
      path: '/system/tools/llm-config',
    },
    {
      desc: '管理 skills 模块、说明文档和相关资产。',
      name: '技能管理',
      path: '/system/tools/skills',
    },
    {
      desc: '内部系统配置，按 tabs 分类维护。',
      name: '系统配置',
      path: '/system/tools/config',
    },
    {
      desc: '邮件配置、测试发送、正式收发记录。',
      name: '邮件管理',
      path: '/system/tools/plugin-email',
    },
    {
      desc: '公告内容、公告投放与附件维护。',
      name: '公告管理',
      path: '/system/tools/announcement',
    },
  ];
}

onMounted(() => {
  reload();
});
</script>

<template>
  <Page auto-content-height title="系统工具">
    <div class="mb-4 flex justify-end">
      <Button @click="reload">
        <template #icon>
          <IconifyIcon icon="mdi:refresh" />
        </template>
        刷新
      </Button>
    </div>
    <Table :data-source="rows" :pagination="false" row-key="path" :scroll="{ x: 900 }">
      <TableColumn data-index="name" key="name" title="模块" width="180" />
      <TableColumn data-index="desc" key="desc" title="说明" />
      <TableColumn fixed="right" key="actions" title="入口" width="120">
        <template #default="{ record }">
          <Button size="small" type="link" @click="navigate(record.path)">打开</Button>
        </template>
      </TableColumn>
    </Table>
  </Page>
</template>
