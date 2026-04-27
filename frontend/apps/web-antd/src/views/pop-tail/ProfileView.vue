<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';

import { Page } from '@vben/common-ui';

import { Alert, Button, Card, Col, Form, FormItem, Input, Row, Space } from 'ant-design-vue';

import { updateProfileApi } from '#/api/pop-tail/admin';
import { useAuthStore } from '#/store/pop-tail/auth';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

interface ProfileDraft {
  email: string;
  nickName: string;
  password: string;
  phone: string;
}

const auth = useAuthStore();
const savingProfile = ref(false);
const statusMessage = ref('当前资料已加载，可直接修改并保存。');
usePageRefreshLoading(async () => {
  syncDraftFromUser();
});

const draft = reactive<ProfileDraft>({
  email: '',
  nickName: '',
  password: '',
  phone: '',
});

const baseProfile = computed<ProfileDraft>(() => ({
  email: auth.userInfo?.email ?? '',
  nickName: auth.userInfo?.nickName ?? '',
  password: '',
  phone: auth.userInfo?.phone ?? '',
}));

const validationErrors = computed(() => {
  const issues: string[] = [];
  if (!draft.nickName.trim()) {
    issues.push('昵称不能为空');
  }
  if (draft.phone.trim() && !/^[0-9+\-() ]{6,32}$/.test(draft.phone.trim())) {
    issues.push('手机号格式不正确');
  }
  if (draft.email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(draft.email.trim())) {
    issues.push('邮箱格式不正确');
  }
  if (draft.password.trim() && draft.password.trim().length < 6) {
    issues.push('新密码至少 6 位');
  }
  return issues;
});

function syncDraftFromUser() {
  Object.assign(draft, { ...baseProfile.value, password: '' });
}

watch(
  () => auth.userInfo,
  () => {
    syncDraftFromUser();
  },
  { immediate: true },
);

async function saveProfile() {
  if (validationErrors.value.length > 0) {
    statusMessage.value = validationErrors.value.join('；');
    return;
  }

  savingProfile.value = true;
  statusMessage.value = '正在保存个人资料...';
  try {
    const user = await updateProfileApi({
      email: draft.email.trim(),
      nickName: draft.nickName.trim(),
      password: draft.password.trim() || undefined,
      phone: draft.phone.trim(),
    });
    await auth.hydrateAccessEnvelope(user);
    syncDraftFromUser();
    statusMessage.value = draft.password.trim() ? '个人资料和密码已保存。' : '个人资料已保存。';
  } catch (error) {
    statusMessage.value = error instanceof Error ? error.message : '保存个人资料失败';
  } finally {
    savingProfile.value = false;
  }
}
</script>

<template>
  <Page title="个人资料">
    <div class="profile-page relative flex h-full min-h-0 flex-1 flex-col">
      <Card :bordered="false" class="profile-card">
        <Alert
          :message="statusMessage"
          :type="validationErrors.length > 0 ? 'warning' : 'info'"
          class="mb-4"
          show-icon
        />

        <Form layout="vertical">
          <Row :gutter="16">
            <Col :md="12" :xs="24">
              <FormItem label="用户名">
                <Input :value="auth.userInfo?.userName ?? '-'" readonly />
              </FormItem>
            </Col>
            <Col :md="12" :xs="24">
              <FormItem label="用户 UUID">
                <Input :value="auth.userInfo?.uuid ?? '-'" readonly />
              </FormItem>
            </Col>
            <Col :md="12" :xs="24">
              <FormItem label="昵称">
                <Input v-model:value="draft.nickName" :maxlength="64" placeholder="请输入昵称" />
              </FormItem>
            </Col>
            <Col :md="12" :xs="24">
              <FormItem label="手机号">
                <Input v-model:value="draft.phone" :maxlength="32" placeholder="请输入手机号" />
              </FormItem>
            </Col>
            <Col :xs="24">
              <FormItem label="邮箱">
                <Input v-model:value="draft.email" :maxlength="128" placeholder="请输入邮箱" />
              </FormItem>
            </Col>
            <Col :xs="24">
              <FormItem label="新密码">
                <Input.Password
                  v-model:value="draft.password"
                  :maxlength="64"
                  autocomplete="new-password"
                  placeholder="留空则不修改密码"
                />
              </FormItem>
            </Col>
          </Row>
        </Form>

        <Space>
          <Button
            :loading="savingProfile"
            type="primary"
            @click="saveProfile"
          >
            保存资料
          </Button>
        </Space>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
.profile-page {
  display: flex;
  flex-direction: column;
}

.profile-card {
  border-radius: 16px;
}
</style>
