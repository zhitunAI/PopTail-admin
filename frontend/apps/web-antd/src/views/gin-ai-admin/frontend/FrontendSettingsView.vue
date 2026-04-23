<script setup lang="ts">
import type { FrontendSettingsInfo } from '#/types/gin-ai-admin';

import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Button, Form, Input, InputNumber, TabPane, Tabs, Upload, message } from 'ant-design-vue';

import {
  getFrontendSettingsApi,
  saveFrontendSettingsApi,
  uploadFileAssetApi,
} from '#/api/gin-ai-admin/admin';

const loading = ref(false);
const saving = ref(false);

const form = reactive<FrontendSettingsInfo>({
  logoUrl: '',
  mailFrom: '',
  recordNumber: '',
  siteDescription: '',
  siteName: '',
  siteSlogan: '',
  smtpHost: '',
  smtpPassword: '',
  smtpPort: 465,
  smtpUser: '',
});

function applySettings(settings: FrontendSettingsInfo) {
  Object.assign(form, settings);
}

async function loadSettings() {
  loading.value = true;
  try {
    applySettings(await getFrontendSettingsApi());
  } finally {
    loading.value = false;
  }
}

async function saveSettings() {
  saving.value = true;
  try {
    applySettings(await saveFrontendSettingsApi({ ...form }));
    message.success('前台设置已保存');
  } finally {
    saving.value = false;
  }
}

async function uploadLogo(file: File) {
  const asset = await uploadFileAssetApi(file);
  form.logoUrl = asset.url;
  message.success('Logo 已上传');
  return false;
}

onMounted(() => {
  void loadSettings();
});
</script>

<template>
  <Page auto-content-height title="前台设置">
    <Tabs>
      <TabPane key="base" tab="基础设置">
        <Form :model="form" layout="vertical" class="max-w-220" :disabled="loading">
          <Form.Item label="Logo 上传">
            <div class="flex items-center gap-4">
              <img
                v-if="form.logoUrl"
                :src="form.logoUrl"
                class="size-14 rounded-lg border border-[var(--ant-color-border)] object-cover"
                alt="logo"
              />
              <Upload :before-upload="uploadLogo" :show-upload-list="false" accept="image/*">
                <Button>上传 Logo</Button>
              </Upload>
            </div>
          </Form.Item>
          <Form.Item label="Logo 地址">
            <Input v-model:value="form.logoUrl" placeholder="https://..." />
          </Form.Item>
          <Form.Item label="网站名称">
            <Input v-model:value="form.siteName" placeholder="请输入网站名称" />
          </Form.Item>
          <Form.Item label="网站标语">
            <Input v-model:value="form.siteSlogan" placeholder="例如：开启你的 Agent 模式，即刻造梦！" />
          </Form.Item>
          <Form.Item label="网站描述">
            <Input.TextArea v-model:value="form.siteDescription" :rows="4" />
          </Form.Item>
          <Form.Item label="备案号">
            <Input v-model:value="form.recordNumber" />
          </Form.Item>
          <Button type="primary" :loading="saving" @click="saveSettings">保存基础设置</Button>
        </Form>
      </TabPane>
      <TabPane key="mail" tab="邮件设置">
        <Form :model="form" layout="vertical" class="max-w-220" :disabled="loading">
          <Form.Item label="SMTP Host">
            <Input v-model:value="form.smtpHost" placeholder="smtp.example.com" />
          </Form.Item>
          <Form.Item label="SMTP Port">
            <InputNumber v-model:value="form.smtpPort" class="w-full" :min="1" :max="65_535" />
          </Form.Item>
          <Form.Item label="SMTP 用户">
            <Input v-model:value="form.smtpUser" />
          </Form.Item>
          <Form.Item label="SMTP 密码">
            <Input.Password v-model:value="form.smtpPassword" />
          </Form.Item>
          <Form.Item label="发件人">
            <Input v-model:value="form.mailFrom" placeholder="站点名称 <notice@example.com>" />
          </Form.Item>
          <Button type="primary" :loading="saving" @click="saveSettings">保存邮件设置</Button>
        </Form>
      </TabPane>
    </Tabs>
  </Page>
</template>
