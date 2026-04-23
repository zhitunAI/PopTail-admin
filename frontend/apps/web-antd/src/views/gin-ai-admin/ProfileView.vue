
<template>
  <Page
    title="个人资料与偏好工作台"
  >
    <div class="vben-page profile-page">
      <Card :bordered="false" class="hero-card">
        <div class="hero-head">
          <div class="hero-title-row">
            <IconifyIcon class="hero-icon" icon="carbon:user-profile" />
            <div>
              <h2>资料维护、偏好设置与角色巡检</h2>
              <p>基于当前登录态与个人资料接口维护用户信息，并将本地偏好、最近动作和草稿恢复统一到一个页面。</p>
            </div>
          </div>
          <Space wrap>
            <Button :loading="savingProfile" type="primary" @click="saveProfile">
              <template #icon>
                <IconifyIcon icon="carbon:save" />
              </template>
              保存资料
            </Button>
            <Button :loading="savingPreferences" @click="savePreferences">保存偏好</Button>
            <Button @click="restoreFromCurrentProfile">恢复当前资料</Button>
          </Space>
        </div>
      </Card>

      <Row :gutter="[16, 16]">
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :precision="0" :value="completenessScore" suffix="%" title="资料完整度">
              <template #prefix>
                <IconifyIcon icon="carbon:checkmark-outline" />
              </template>
            </Statistic>
            <p>{{ completenessHint }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="diffItems.length" title="待同步字段">
              <template #prefix>
                <IconifyIcon icon="carbon:compare" />
              </template>
            </Statistic>
            <p>{{ diffItems.length > 0 ? '存在草稿差异' : '与当前资料一致' }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="authoritySummary.total" title="角色覆盖">
              <template #prefix>
                <IconifyIcon icon="carbon:user-role" />
              </template>
            </Statistic>
            <p>{{ authoritySummary.primary }}</p>
          </Card>
        </Col>
        <Col :lg="6" :md="12" :xs="24">
          <Card :bordered="false" class="metric-card">
            <Statistic :value="activityFeed.length" title="最近动作">
              <template #prefix>
                <IconifyIcon icon="carbon:recently-viewed" />
              </template>
            </Statistic>
            <p>{{ lastActionLabel }}</p>
          </Card>
        </Col>
      </Row>

      <div class="alert-stack">
        <Alert :message="profileStatusMessage" show-icon type="info" />
        <Alert v-if="validationErrors.length > 0" :message="validationErrors.join('；')" show-icon type="warning" />
      </div>

      <Row :gutter="[16, 16]">
        <Col :lg="14" :xs="24">
          <Card :bordered="false" class="panel-card" title="资料维护">
            <template #extra>
              <Tag :color="profileDirty ? 'warning' : 'success'">
                {{ profileDirty ? '草稿未同步' : '资料已同步' }}
              </Tag>
            </template>
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
                    <Input v-model:value="draft.nickName" :maxlength="64" placeholder="请输入昵称" @blur="trackDraftTouch('昵称')" />
                  </FormItem>
                </Col>
                <Col :md="12" :xs="24">
                  <FormItem label="手机号">
                    <Input v-model:value="draft.phone" :maxlength="32" placeholder="请输入手机号" @blur="trackDraftTouch('手机号')" />
                  </FormItem>
                </Col>
                <Col :xs="24">
                  <FormItem label="邮箱">
                    <Input v-model:value="draft.email" :maxlength="128" placeholder="请输入邮箱" @blur="trackDraftTouch('邮箱')" />
                  </FormItem>
                </Col>
              </Row>
            </Form>

            <Space wrap>
              <Button :loading="savingProfile" type="primary" @click="saveProfile">提交资料更新</Button>
              <Button @click="saveDraftSnapshot">暂存本地草稿</Button>
              <Button :disabled="!hasStoredDraft" @click="restoreStoredDraft">回填本地草稿</Button>
              <Button :disabled="!hasStoredDraft" danger @click="clearStoredDraft">清空本地草稿</Button>
            </Space>

            <Table
              :columns="[
                { title: '字段', dataIndex: 'label', key: 'label' },
                { title: '当前值', dataIndex: 'before', key: 'before' },
                { title: '草稿值', dataIndex: 'after', key: 'after' },
              ]"
              :data-source="diffItems"
              :locale="{ emptyText: '当前草稿与已登录资料一致，无需额外同步。' }"
              :pagination="false"
              class="top-gap"
              row-key="label"
              size="small"
            />
          </Card>
        </Col>
        <Col :lg="10" :xs="24">
          <Card :bordered="false" class="panel-card" title="角色概览">
            <template #extra>
              <Tag :color="auth.userInfo?.enable === 1 ? 'success' : 'error'">
                {{ auth.userInfo?.enable === 1 ? '账号启用' : '账号冻结' }}
              </Tag>
            </template>
            <Descriptions :column="1" bordered size="small">
              <DescriptionsItem label="主角色">{{ authoritySummary.primary }}</DescriptionsItem>
              <DescriptionsItem label="默认路由">{{ auth.userInfo?.authority.defaultRouter ?? '-' }}</DescriptionsItem>
              <DescriptionsItem label="权限路径数">{{ auth.policyPaths.length }}</DescriptionsItem>
              <DescriptionsItem label="高频方法">{{ authoritySummary.topMethod }}</DescriptionsItem>
              <DescriptionsItem label="热点路径段">{{ authoritySummary.topSegment }}</DescriptionsItem>
              <DescriptionsItem label="备选角色数">{{ authoritySummary.alternativeCount }}</DescriptionsItem>
            </Descriptions>
            <Space class="tag-wrap top-gap" wrap>
              <Tag v-for="item in auth.userInfo?.authorities ?? []" :key="item.authorityId" color="blue">
                {{ item.authorityName }} / {{ item.authorityId }}
              </Tag>
            </Space>
          </Card>
        </Col>
      </Row>

      <Row :gutter="[16, 16]">
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="快捷偏好">
            <template #extra>
              <Tag :color="preferenceDirty ? 'warning' : 'success'">
                {{ preferenceDirty ? '偏好未保存' : '偏好已落盘' }}
              </Tag>
            </template>
            <Form layout="vertical">
              <Row :gutter="16">
                <Col :md="12" :xs="24">
                  <FormItem label="首页动作">
                    <Select
                      v-model:value="preferences.homeAction"
                      :options="[
                        { label: 'dashboard', value: 'dashboard' },
                        { label: 'users', value: 'users' },
                        { label: 'systemTools', value: 'systemTools' },
                        { label: 'runtimeState', value: 'runtimeState' },
                      ]"
                    />
                  </FormItem>
                </Col>
                <Col :md="12" :xs="24">
                  <FormItem label="列表密度">
                    <Select
                      v-model:value="preferences.tableDensity"
                      :options="[
                        { label: 'comfortable', value: 'comfortable' },
                        { label: 'compact', value: 'compact' },
                      ]"
                    />
                  </FormItem>
                </Col>
                <Col :md="12" :xs="24">
                  <FormItem label="提醒频率">
                    <Select
                      v-model:value="preferences.digestFrequency"
                      :options="[
                        { label: 'daily', value: 'daily' },
                        { label: 'weekly', value: 'weekly' },
                        { label: 'manual', value: 'manual' },
                      ]"
                    />
                  </FormItem>
                </Col>
                <Col :md="12" :xs="24">
                  <FormItem label="个人视角">
                    <Select
                      v-model:value="preferences.watchFocus"
                      :options="[
                        { label: 'profile', value: 'profile' },
                        { label: 'audit', value: 'audit' },
                        { label: 'runtime', value: 'runtime' },
                      ]"
                    />
                  </FormItem>
                </Col>
              </Row>
            </Form>
            <Space wrap>
              <Button :loading="savingPreferences" type="primary" @click="savePreferences">保存本地偏好</Button>
              <Button @click="restoreStoredPreferences">回填本地偏好</Button>
            </Space>
            <Descriptions :column="1" bordered class="top-gap" size="small">
              <DescriptionsItem v-for="item in preferenceSummary" :key="item.label" :label="item.label">
                {{ item.value }}
              </DescriptionsItem>
            </Descriptions>
          </Card>
        </Col>
        <Col :lg="12" :xs="24">
          <Card :bordered="false" class="panel-card" title="最近动作与草稿状态">
            <template #extra>
              <Space>
                <Tag color="default">草稿 {{ hasStoredDraft ? '已保存' : '未保存' }}</Tag>
                <Button size="small" @click="clearActivities">清空活动</Button>
              </Space>
            </template>
            <div v-if="activityFeed.length" class="record-list">
              <div v-for="item in activityFeed" :key="item.id" class="record-item">
                <div class="record-head">
                  <strong>{{ item.label }}</strong>
                  <span class="soft-text">{{ item.time }}</span>
                </div>
                <p>{{ item.detail }}</p>
              </div>
            </div>
            <Empty v-else description="暂无最近动作记录" />
          </Card>
        </Col>
      </Row>
    </div>
  </Page>
</template>

<script setup lang="ts">
import { Page } from '@vben/common-ui';
import { IconifyIcon } from '@vben/icons';
import { Alert, Button, Card, Col, Descriptions, DescriptionsItem, Empty, Form, FormItem, Input, Row, Select, Space, Statistic, Table, Tag } from 'ant-design-vue';
import { computed, reactive, ref, watch } from "vue";
import { updateProfileApi } from "#/api/gin-ai-admin/admin";
import { useAuthStore } from "#/store/gin-ai-admin/auth";

interface ProfileDraft {
  nickName: string;
  phone: string;
  email: string;
}

interface ProfilePreferenceDraft {
  homeAction: string;
  tableDensity: string;
  digestFrequency: string;
  watchFocus: string;
}

interface ActivityRecord {
  id: string;
  time: string;
  label: string;
  detail: string;
}

const PROFILE_DRAFT_KEY = "gaa-profile-draft";
const PROFILE_PREFERENCES_KEY = "gaa-profile-preferences";
const PROFILE_ACTIVITY_KEY = "gaa-profile-activity-log";

const auth = useAuthStore();

const draft = reactive<ProfileDraft>(readCurrentProfile());
const preferences = reactive<ProfilePreferenceDraft>(loadPreferences());
const persistedPreferenceSnapshot = ref<ProfilePreferenceDraft>(loadPreferences());
const activityFeed = ref<ActivityRecord[]>(loadActivities());
const savingProfile = ref(false);
const savingPreferences = ref(false);
const storedDraftExists = ref(Boolean(localStorage.getItem(PROFILE_DRAFT_KEY)));
const profileStatusMessage = ref("当前资料与本地偏好已完成加载，可直接编辑或回填。");

const baseProfile = computed<ProfileDraft>(() => ({
  nickName: auth.userInfo?.nickName ?? "",
  phone: auth.userInfo?.phone ?? "",
  email: auth.userInfo?.email ?? "",
}));

const diffItems = computed(() => [
  {
    label: "昵称",
    before: baseProfile.value.nickName,
    after: draft.nickName,
  },
  {
    label: "手机号",
    before: baseProfile.value.phone,
    after: draft.phone,
  },
  {
    label: "邮箱",
    before: baseProfile.value.email,
    after: draft.email,
  },
].filter((item) => item.before !== item.after));

const profileDirty = computed(() => diffItems.value.length > 0);
const hasStoredDraft = computed(() => storedDraftExists.value);
const preferenceDirty = computed(
  () => JSON.stringify(preferences) !== JSON.stringify(persistedPreferenceSnapshot.value),
);

const validationErrors = computed(() => {
  const issues: string[] = [];
  if (!draft.nickName.trim()) {
    issues.push("昵称不能为空");
  }
  if (draft.phone.trim() && !/^[0-9+\-() ]{6,32}$/.test(draft.phone.trim())) {
    issues.push("手机号格式不正确");
  }
  if (draft.email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(draft.email.trim())) {
    issues.push("邮箱格式不正确");
  }
  return issues;
});

const completenessScore = computed(() => {
  const fields = [
    draft.nickName.trim(),
    draft.phone.trim(),
    draft.email.trim(),
    preferences.homeAction.trim(),
    preferences.tableDensity.trim(),
    preferences.digestFrequency.trim(),
    preferences.watchFocus.trim(),
  ];
  const filled = fields.filter(Boolean).length;
  const bonus = validationErrors.value.length === 0 ? 1 : 0;
  return Math.min(100, Math.round(((filled + bonus) / (fields.length + 1)) * 100));
});

const completenessHint = computed(() => {
  if (validationErrors.value.length > 0) {
    return "有待修正字段";
  }
  if (completenessScore.value >= 90) {
    return "资料与偏好较完整";
  }
  if (completenessScore.value >= 70) {
    return "建议补全联系信息";
  }
  return "建议尽快补全资料";
});

const methodCounter = computed(() => {
  return auth.policyPaths.reduce<Record<string, number>>((acc, item) => {
    acc[item.method] = (acc[item.method] ?? 0) + 1;
    return acc;
  }, {});
});

const segmentCounter = computed(() => {
  return auth.policyPaths.reduce<Record<string, number>>((acc, item) => {
    const segment = item.path.split("/").filter(Boolean)[0] ?? "root";
    acc[segment] = (acc[segment] ?? 0) + 1;
    return acc;
  }, {});
});

const authoritySummary = computed(() => ({
  total: auth.userInfo?.authorities.length ?? 0,
  primary: auth.userInfo?.authority.authorityName ?? "未识别主角色",
  alternativeCount: Math.max((auth.userInfo?.authorities.length ?? 1) - 1, 0),
  topMethod: pickTopEntry(methodCounter.value),
  topSegment: pickTopEntry(segmentCounter.value),
}));

const preferenceSummary = computed(() => [
  { label: "默认首页", value: preferences.homeAction },
  { label: "列表密度", value: preferences.tableDensity },
  { label: "提醒频率", value: preferences.digestFrequency },
  { label: "关注重心", value: preferences.watchFocus },
]);

const lastActionLabel = computed(() => activityFeed.value[0]?.label ?? "暂无记录");

watch(
  () => auth.userInfo,
  () => {
    if (!profileDirty.value) {
      Object.assign(draft, baseProfile.value);
    }
  },
  { deep: true },
);

function readCurrentProfile(): ProfileDraft {
  return {
    nickName: auth.userInfo?.nickName ?? "",
    phone: auth.userInfo?.phone ?? "",
    email: auth.userInfo?.email ?? "",
  };
}

function loadPreferences(): ProfilePreferenceDraft {
  const raw = localStorage.getItem(PROFILE_PREFERENCES_KEY);
  if (!raw) {
    return {
      homeAction: "dashboard",
      tableDensity: "comfortable",
      digestFrequency: "daily",
      watchFocus: "profile",
    };
  }
  try {
    const parsed = JSON.parse(raw) as Partial<ProfilePreferenceDraft>;
    return {
      homeAction: parsed.homeAction ?? "dashboard",
      tableDensity: parsed.tableDensity ?? "comfortable",
      digestFrequency: parsed.digestFrequency ?? "daily",
      watchFocus: parsed.watchFocus ?? "profile",
    };
  } catch {
    return {
      homeAction: "dashboard",
      tableDensity: "comfortable",
      digestFrequency: "daily",
      watchFocus: "profile",
    };
  }
}

function loadActivities(): ActivityRecord[] {
  const raw = localStorage.getItem(PROFILE_ACTIVITY_KEY);
  if (!raw) {
    return [
      {
        id: createId(),
        time: new Date().toLocaleString("zh-CN"),
        label: "个人工作台已初始化",
        detail: "已从当前登录态加载资料、角色与本地偏好。",
      },
    ];
  }
  try {
    const parsed = JSON.parse(raw) as ActivityRecord[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function saveActivities(records: ActivityRecord[]) {
  localStorage.setItem(PROFILE_ACTIVITY_KEY, JSON.stringify(records));
}

function createId() {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function pickTopEntry(counter: Record<string, number>) {
  const [label = "暂无"] =
    Object.entries(counter).sort((left, right) => right[1] - left[1])[0] ?? [];
  return label;
}

function pushActivity(label: string, detail: string) {
  const next = [
    {
      id: createId(),
      time: new Date().toLocaleString("zh-CN"),
      label,
      detail,
    },
    ...activityFeed.value,
  ].slice(0, 12);
  activityFeed.value = next;
  saveActivities(next);
}

function trackDraftTouch(fieldLabel: string) {
  profileStatusMessage.value = `已更新${fieldLabel}草稿，可保存到服务端或暂存到本地。`;
}

function restoreFromCurrentProfile() {
  Object.assign(draft, baseProfile.value);
  profileStatusMessage.value = "已使用当前登录资料覆盖本地草稿。";
  pushActivity("恢复当前资料", "已将当前登录资料重新载入编辑区。");
}

function saveDraftSnapshot() {
  localStorage.setItem(
    PROFILE_DRAFT_KEY,
    JSON.stringify({
      ...draft,
      savedAt: new Date().toLocaleString("zh-CN"),
    }),
  );
  storedDraftExists.value = true;
  profileStatusMessage.value = "个人资料草稿已暂存到本地。";
  pushActivity("暂存本地草稿", "已保存昵称、手机号与邮箱草稿，稍后可一键回填。");
}

function restoreStoredDraft() {
  const raw = localStorage.getItem(PROFILE_DRAFT_KEY);
  if (!raw) {
    profileStatusMessage.value = "未找到可回填的本地资料草稿。";
    return;
  }
  try {
    const parsed = JSON.parse(raw) as Partial<ProfileDraft>;
    draft.nickName = parsed.nickName ?? draft.nickName;
    draft.phone = parsed.phone ?? draft.phone;
    draft.email = parsed.email ?? draft.email;
    profileStatusMessage.value = "已回填本地资料草稿。";
    pushActivity("回填本地草稿", "已将最近一次暂存的本地资料草稿应用到编辑区。");
  } catch {
    profileStatusMessage.value = "本地草稿损坏，无法回填。";
  }
}

function clearStoredDraft() {
  localStorage.removeItem(PROFILE_DRAFT_KEY);
  storedDraftExists.value = false;
  profileStatusMessage.value = "本地资料草稿已清空。";
  pushActivity("清空本地草稿", "已删除当前浏览器保存的个人资料草稿。");
}

async function saveProfile() {
  if (validationErrors.value.length > 0) {
    profileStatusMessage.value = `无法保存：${validationErrors.value.join("；")}`;
    return;
  }
  savingProfile.value = true;
  profileStatusMessage.value = "正在同步个人资料，请稍候...";
  try {
    const changedCount = diffItems.value.length;
    const user = await updateProfileApi({
      nickName: draft.nickName.trim(),
      phone: draft.phone.trim(),
      email: draft.email.trim(),
    });
    await auth.hydrateAccessEnvelope(user);
    localStorage.removeItem(PROFILE_DRAFT_KEY);
    storedDraftExists.value = false;
    profileStatusMessage.value = "个人资料已同步到服务端，并刷新当前登录态。";
    pushActivity(
      "保存个人资料",
      `已同步 ${changedCount === 0 ? "资料" : `${changedCount} 个字段`} 并刷新当前用户信息。`,
    );
  } catch (error) {
    profileStatusMessage.value = error instanceof Error ? error.message : "保存个人资料失败";
    pushActivity("保存个人资料失败", profileStatusMessage.value);
  } finally {
    savingProfile.value = false;
  }
}

async function savePreferences() {
  savingPreferences.value = true;
  try {
    localStorage.setItem(PROFILE_PREFERENCES_KEY, JSON.stringify(preferences));
    persistedPreferenceSnapshot.value = { ...preferences };
    profileStatusMessage.value = "本地偏好已保存，可在下次进入个人中心时自动回填。";
    pushActivity(
      "保存个人偏好",
      `默认首页=${preferences.homeAction}，列表密度=${preferences.tableDensity}，提醒频率=${preferences.digestFrequency}。`,
    );
  } finally {
    savingPreferences.value = false;
  }
}

function restoreStoredPreferences() {
  const restored = loadPreferences();
  Object.assign(preferences, restored);
  persistedPreferenceSnapshot.value = { ...restored };
  profileStatusMessage.value = "已从本地重新载入个人偏好。";
  pushActivity("回填个人偏好", "已从浏览器本地设置恢复首页动作与值班偏好。");
}

function clearActivities() {
  activityFeed.value = [];
  saveActivities([]);
  profileStatusMessage.value = "个人动作记录已清空。";
}
</script>


<style scoped>
.vben-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-card,
.panel-card,
.metric-card {
  border-radius: 16px;
}

.hero-head,
.hero-title-row,
.record-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.hero-title-row {
  align-items: center;
}

.hero-icon,
.result-icon {
  font-size: 28px;
  color: var(--ant-color-primary);
}

.hero-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.metric-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.metric-card p,
.panel-card p,
.hero-card p,
.field-hint,
.soft-text,
.footnote,
.selection-card span {
  margin: 0;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.panel-card :deep(.ant-card-body) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tag-wrap {
  width: 100%;
}

.alert-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.top-gap {
  margin-top: 12px;
}

.record-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.record-item,
.selection-card,
.inner-card :deep(.ant-card-body) {
  border: 1px solid var(--ant-color-border-secondary);
  border-radius: 14px;
  padding: 12px 14px;
  background: var(--ant-color-fill-quaternary);
}

.selection-card {
  text-align: left;
  width: 100%;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.selection-card:hover {
  border-color: var(--ant-color-primary);
  transform: translateY(-1px);
}

.selection-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.section-caption {
  margin: 0 0 8px;
  color: var(--ant-color-text-description);
  font-size: 13px;
}

.full-width {
  width: 100%;
}

.toolbar-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toolbar-field label,
.section-head h3,
.hero-card h2,
.inner-card h3 {
  margin: 0;
}

.section-head p {
  margin: 4px 0 0;
}

.profile-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.form-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.bullet-list {
  margin: 0;
  padding-left: 18px;
  color: var(--ant-color-text-description);
}

.bullet-list li + li {
  margin-top: 6px;
}

.session-summary {
  display: flex;
  gap: 12px;
  align-items: center;
}

.summary-avatar {
  background: color-mix(in srgb, var(--ant-color-primary) 12%, white);
  color: var(--ant-color-primary);
}

@media (max-width: 768px) {
  .hero-head,
  .hero-title-row,
  .record-head,
  .form-meta {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
