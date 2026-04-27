<script setup lang="ts">
import type { SystemConfigInfo } from '#/types/pop-tail';

import { computed, onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { updatePreferences } from '@vben/preferences';

import { Button, Form, Input, InputNumber, Select, Switch, TabPane, Tabs, Upload, message } from 'ant-design-vue';

import { getSystemConfigApi, updateSystemConfigApi } from '#/api/pop-tail/admin';
import { STANDARD_BUTTON_LABELS, useMenuButtonAccess } from '#/utils/menu-button-access';
import { usePageRefreshLoading } from '#/utils/use-page-refresh-loading';

type ExtendedConfig = SystemConfigInfo & {
  email: {
    from: string;
    host: string;
    isLoginAuth: boolean;
    isSSL: boolean;
    nickname: string;
    port: number;
    secret: string;
    to: string;
  };
  jwt: {
    bufferTime: string;
    expiresTime: string;
    issuer: string;
    signingKey: string;
  };
  redis: {
    addr: string;
    db: number;
    password: string;
  };
  system: {
    dbType: string;
    iplimitCount: number;
    iplimitTime: number;
    logoUrl: string;
    ossType: string;
    routerPrefix: string;
    useMongo: boolean;
    useRedis: boolean;
    useStrictAuth: boolean;
  };
  zap: {
    director: string;
    format: string;
    level: string;
    logInConsole: boolean;
    prefix: string;
    retentionDay: number;
    showLine: boolean;
  };
};

type PersistedExtendedConfig = {
  email?: Pick<
    ExtendedConfig['email'],
    'from' | 'host' | 'isLoginAuth' | 'isSSL' | 'nickname' | 'port' | 'to'
  >;
  jwt?: Pick<ExtendedConfig['jwt'], 'bufferTime' | 'expiresTime' | 'issuer'>;
  redis?: Pick<ExtendedConfig['redis'], 'addr' | 'db'>;
  system?: ExtendedConfig['system'];
  zap?: ExtendedConfig['zap'];
};

const STORAGE_KEY = 'pop-tail-system-config-v1';
const DEFAULT_LOGO_URL = '/brand/logo.svg';
const activeTab = ref('system');
const loading = ref(false);
const saving = ref(false);
const { canUse } = useMenuButtonAccess();
usePageRefreshLoading(load);

const form = reactive<ExtendedConfig>({
  bindAddress: '',
  compatibilityRefreshHeaders: true,
  databaseUrl: '',
  email: {
    from: '',
    host: '',
    isLoginAuth: true,
    isSSL: true,
    nickname: '',
    port: 465,
    secret: '',
    to: '',
  },
  jwt: {
    bufferTime: '1d',
    expiresTime: '7d',
    issuer: 'gaa',
    signingKey: '',
  },
  multipointEnabled: false,
  redis: {
    addr: '',
    db: 0,
    password: '',
  },
  redisUrl: '',
  system: {
    dbType: 'pgsql',
    iplimitCount: 15000,
    iplimitTime: 3600,
    logoUrl: '',
    ossType: 'local',
    routerPrefix: '',
    useMongo: false,
    useRedis: true,
    useStrictAuth: false,
  },
  zap: {
    director: 'log',
    format: 'console',
    level: 'info',
    logInConsole: true,
    prefix: '[PopTail-admin]',
    retentionDay: -1,
    showLine: true,
  },
});

const baseline = ref(JSON.stringify(form));

const dbTypeOptions = [
  { label: 'PostgreSQL', value: 'pgsql' },
  { label: 'MySQL', value: 'mysql' },
  { label: 'SQLite', value: 'sqlite' },
  { label: 'MSSQL', value: 'mssql' },
  { label: 'Oracle', value: 'oracle' },
];

const ossOptions = [
  { label: '本地', value: 'local' },
  { label: '七牛', value: 'qiniu' },
  { label: '腾讯 COS', value: 'tencent-cos' },
  { label: '阿里 OSS', value: 'aliyun-oss' },
  { label: '华为 OBS', value: 'huawei-obs' },
  { label: 'Cloudflare R2', value: 'cloudflare-r2' },
  { label: 'MinIO', value: 'minio' },
];

const zapLevelOptions = ['debug', 'info', 'warn', 'error', 'fatal', 'trace', 'off'].map((value) => ({ label: value, value }));
const zapFormatOptions = ['console', 'json'].map((value) => ({ label: value, value }));

const dirty = computed(() => JSON.stringify(form) !== baseline.value);

function applyConfig(config: SystemConfigInfo) {
  form.bindAddress = config.bindAddress;
  form.databaseUrl = config.databaseUrl;
  form.redisUrl = config.redisUrl;
  form.multipointEnabled = config.multipointEnabled;
  form.compatibilityRefreshHeaders = config.compatibilityRefreshHeaders;
  form.redis.addr = config.redisUrl.replace(/^redis:\/\//, '');
  form.system.useRedis = true;
}

function loadLocalConfig() {
  if (typeof window === 'undefined') return;
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return;
  try {
    const parsed = JSON.parse(raw) as PersistedExtendedConfig;
    Object.assign(form.email, parsed.email ?? {});
    Object.assign(form.jwt, parsed.jwt ?? {});
    Object.assign(form.redis, parsed.redis ?? {});
    Object.assign(form.system, parsed.system ?? {});
    Object.assign(form.zap, parsed.zap ?? {});
  } catch {
    // ignore broken local state
  }
}

function saveLocalConfig() {
  if (typeof window === 'undefined') return;
  const persisted: PersistedExtendedConfig = {
    email: {
      from: form.email.from,
      host: form.email.host,
      isLoginAuth: form.email.isLoginAuth,
      isSSL: form.email.isSSL,
      nickname: form.email.nickname,
      port: form.email.port,
      to: form.email.to,
    },
    jwt: {
      bufferTime: form.jwt.bufferTime,
      expiresTime: form.jwt.expiresTime,
      issuer: form.jwt.issuer,
    },
    redis: {
      addr: form.redis.addr,
      db: form.redis.db,
    },
    system: { ...form.system },
    zap: { ...form.zap },
  };
  window.localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify(persisted),
  );
}

function resolveLogoUrl(url?: string) {
  const trimmed = (url || '').trim();
  return trimmed || DEFAULT_LOGO_URL;
}

function applyRuntimeLogo(url?: string) {
  const source = resolveLogoUrl(url);
  updatePreferences({
    logo: {
      source,
      sourceDark: source,
    },
  });
}

async function load() {
  loading.value = true;
  try {
    const config = await getSystemConfigApi();
    applyConfig(config);
    loadLocalConfig();
    form.system.logoUrl = resolveLogoUrl(form.system.logoUrl);
    applyRuntimeLogo(form.system.logoUrl);
    baseline.value = JSON.stringify(form);
  } finally {
    loading.value = false;
  }
}

async function save() {
  saving.value = true;
  try {
    form.system.logoUrl = resolveLogoUrl(form.system.logoUrl);
    await updateSystemConfigApi({
      bindAddress: form.bindAddress.trim(),
      compatibilityRefreshHeaders: form.compatibilityRefreshHeaders,
      databaseUrl: form.databaseUrl.trim(),
      multipointEnabled: form.multipointEnabled,
      redisUrl: form.redisUrl.trim(),
    });
    saveLocalConfig();
    applyRuntimeLogo(form.system.logoUrl);
    baseline.value = JSON.stringify(form);
    message.success('系统配置已保存');
  } finally {
    saving.value = false;
  }
}

const logoAccept = [
  '.svg',
  '.png',
  '.jpg',
  '.jpeg',
  '.webp',
  '.gif',
  '.ico',
  'image/svg+xml',
  'image/png',
  'image/jpeg',
  'image/webp',
  'image/gif',
  'image/x-icon',
].join(',');

async function readFileAsDataUrl(file: File) {
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ''));
    reader.onerror = () => reject(new Error('读取文件失败'));
    reader.readAsDataURL(file);
  });
}

async function uploadLogo(file: File) {
  const fileName = file.name.toLowerCase();
  const allowed = ['.svg', '.png', '.jpg', '.jpeg', '.webp', '.gif', '.ico'];
  const matched = allowed.some((suffix) => fileName.endsWith(suffix));
  if (!matched) {
    message.error('仅支持 svg、png、jpg、jpeg、webp、gif、ico 格式');
    return false;
  }
  const dataUrl = await readFileAsDataUrl(file);
  form.system.logoUrl = dataUrl;
  applyRuntimeLogo(dataUrl);
  saveLocalConfig();
  baseline.value = JSON.stringify(form);
  message.success('Logo 已上传');
  return false;
}

function restore() {
  void load();
}

onMounted(() => {
  void load();
});
</script>

<template>
  <Page title=" ">
    <template #extra>
      <div class="flex flex-wrap items-center gap-2">
        <Button v-if="canUse(STANDARD_BUTTON_LABELS.edit)" :loading="saving" :disabled="!dirty" type="primary"
          @click="save">保存配置</Button>
        <Button :loading="loading" @click="load">重新加载</Button>
        <Button :disabled="!dirty" @click="restore">恢复</Button>
      </div>
    </template>

    <div class="relative flex h-full min-h-0 flex-1 flex-col">
      <Tabs v-model:activeKey="activeTab">
        <TabPane key="system" tab="系统配置">
          <Form layout="vertical">
            <Form.Item label="系统 Logo">
              <div class="flex items-center gap-4">
                <img v-if="form.system.logoUrl" :src="form.system.logoUrl"
                  class="size-14 rounded-lg border border-[var(--ant-color-border)] object-contain bg-[var(--ant-color-bg-container)] p-2"
                  alt="system-logo" />
                <Upload :before-upload="uploadLogo" :show-upload-list="false" :accept="logoAccept"
                  :disabled="!canUse(STANDARD_BUTTON_LABELS.edit)">
                  <Button>上传 Logo</Button>
                </Upload>
              </div>
            </Form.Item>
            <Form.Item label="Logo 地址">
              <Input v-model:value="form.system.logoUrl" placeholder="https://... 或上传生成的地址" />
            </Form.Item>
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="Bind Address"><Input v-model:value="form.bindAddress" /></Form.Item>
              <Form.Item label="Database URL"><Input v-model:value="form.databaseUrl" /></Form.Item>
              <Form.Item label="Redis URL"><Input v-model:value="form.redisUrl" /></Form.Item>
              <Form.Item label="数据库类型"><Select v-model:value="form.system.dbType" :options="dbTypeOptions" />
              </Form.Item>
              <Form.Item label="Oss 类型"><Select v-model:value="form.system.ossType" :options="ossOptions" /></Form.Item>
              <Form.Item label="路由前缀"><Input v-model:value="form.system.routerPrefix" /></Form.Item>
              <Form.Item label="限流次数">
                <InputNumber v-model:value="form.system.iplimitCount" class="w-full" />
              </Form.Item>
              <Form.Item label="限流时间">
                <InputNumber v-model:value="form.system.iplimitTime" class="w-full" />
              </Form.Item>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="多点登录">
                <Switch v-model:checked="form.multipointEnabled" />
              </Form.Item>
              <Form.Item label="兼容刷新头">
                <Switch v-model:checked="form.compatibilityRefreshHeaders" />
              </Form.Item>
              <Form.Item label="使用 Redis">
                <Switch v-model:checked="form.system.useRedis" />
              </Form.Item>
              <Form.Item label="使用 Mongo">
                <Switch v-model:checked="form.system.useMongo" />
              </Form.Item>
              <Form.Item label="严格角色模式">
                <Switch v-model:checked="form.system.useStrictAuth" />
              </Form.Item>
            </div>
          </Form>
        </TabPane>

        <TabPane key="jwt" tab="JWT">
          <Form layout="vertical">
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="签名 Key">
                <Input.Password v-model:value="form.jwt.signingKey" autocomplete="new-password" />
              </Form.Item>
              <Form.Item label="签发者"><Input v-model:value="form.jwt.issuer" /></Form.Item>
              <Form.Item label="有效期"><Input v-model:value="form.jwt.expiresTime" /></Form.Item>
              <Form.Item label="缓冲期"><Input v-model:value="form.jwt.bufferTime" /></Form.Item>
            </div>
          </Form>
        </TabPane>

        <TabPane key="redis" tab="Redis">
          <Form layout="vertical">
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="地址"><Input v-model:value="form.redis.addr" /></Form.Item>
              <Form.Item label="密码">
                <Input.Password v-model:value="form.redis.password" autocomplete="new-password" />
              </Form.Item>
              <Form.Item label="数据库">
                <InputNumber v-model:value="form.redis.db" class="w-full" :min="0" />
              </Form.Item>
            </div>
          </Form>
        </TabPane>

        <TabPane key="email" tab="邮箱配置">
          <Form layout="vertical">
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="接收者邮箱"><Input v-model:value="form.email.to" /></Form.Item>
              <Form.Item label="发送者邮箱"><Input v-model:value="form.email.from" /></Form.Item>
              <Form.Item label="Host"><Input v-model:value="form.email.host" /></Form.Item>
              <Form.Item label="端口">
                <InputNumber v-model:value="form.email.port" class="w-full" />
              </Form.Item>
              <Form.Item label="昵称"><Input v-model:value="form.email.nickname" /></Form.Item>
              <Form.Item label="Secret">
                <Input.Password v-model:value="form.email.secret" autocomplete="new-password" />
              </Form.Item>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="SSL">
                <Switch v-model:checked="form.email.isSSL" />
              </Form.Item>
              <Form.Item label="LoginAuth">
                <Switch v-model:checked="form.email.isLoginAuth" />
              </Form.Item>
            </div>
          </Form>
        </TabPane>

        <TabPane key="zap" tab="日志配置">
          <Form layout="vertical">
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="级别"><Select v-model:value="form.zap.level" :options="zapLevelOptions" /></Form.Item>
              <Form.Item label="输出格式"><Select v-model:value="form.zap.format" :options="zapFormatOptions" /></Form.Item>
              <Form.Item label="日志前缀"><Input v-model:value="form.zap.prefix" /></Form.Item>
              <Form.Item label="日志目录"><Input v-model:value="form.zap.director" /></Form.Item>
              <Form.Item label="保留天数">
                <InputNumber v-model:value="form.zap.retentionDay" class="w-full" />
              </Form.Item>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <Form.Item label="显示行号">
                <Switch v-model:checked="form.zap.showLine" />
              </Form.Item>
              <Form.Item label="输出控制台">
                <Switch v-model:checked="form.zap.logInConsole" />
              </Form.Item>
            </div>
          </Form>
        </TabPane>
      </Tabs>
    </div>
  </Page>
</template>
