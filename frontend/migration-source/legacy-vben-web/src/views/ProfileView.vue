<template>
  <div class="profile-workbench">
    <section class="hero card">
      <div>
        <p class="eyebrow">个人资料与偏好工作台</p>
        <h2 class="title">围绕当前登录用户做资料维护、偏好回填与角色巡检</h2>
        <p class="subtitle">
          基于当前登录态与个人资料接口维护用户信息，并将本地偏好、最近个人动作和草稿恢复统一到一个工作台。
        </p>
      </div>
      <div class="hero-actions">
        <button class="btn primary" :disabled="savingProfile" @click="saveProfile">
          {{ savingProfile ? "保存中..." : "保存资料" }}
        </button>
        <button class="btn ghost" :disabled="savingPreferences" @click="savePreferences">
          {{ savingPreferences ? "回填中..." : "保存偏好" }}
        </button>
        <button class="btn ghost" @click="restoreFromCurrentProfile">恢复当前资料</button>
      </div>
    </section>

    <section class="summary-grid">
      <article class="stat-card">
        <span class="stat-label">资料完整度</span>
        <strong class="stat-value">{{ completenessScore }}%</strong>
        <span class="stat-note">{{ completenessHint }}</span>
      </article>
      <article class="stat-card">
        <span class="stat-label">待同步字段</span>
        <strong class="stat-value">{{ diffItems.length }}</strong>
        <span class="stat-note">{{ diffItems.length > 0 ? "存在草稿差异" : "与当前资料一致" }}</span>
      </article>
      <article class="stat-card">
        <span class="stat-label">角色覆盖</span>
        <strong class="stat-value">{{ authoritySummary.total }}</strong>
        <span class="stat-note">{{ authoritySummary.primary }}</span>
      </article>
      <article class="stat-card">
        <span class="stat-label">最近动作</span>
        <strong class="stat-value">{{ activityFeed.length }}</strong>
        <span class="stat-note">{{ lastActionLabel }}</span>
      </article>
    </section>

    <section class="split-grid">
      <article class="card">
        <div class="section-head">
          <div>
            <h3>资料维护</h3>
            <p>编辑昵称、手机号、邮箱，并在提交前对比当前登录资料。</p>
          </div>
          <span class="status-chip" :class="profileDirty ? 'warning' : 'success'">
            {{ profileDirty ? "草稿未同步" : "资料已同步" }}
          </span>
        </div>

        <div class="field-grid">
          <div class="field">
            <label>用户名</label>
            <input :value="auth.userInfo?.userName ?? '-'" readonly />
          </div>
          <div class="field">
            <label>用户 UUID</label>
            <input :value="auth.userInfo?.uuid ?? '-'" readonly />
          </div>
          <div class="field">
            <label>昵称</label>
            <input
              v-model.trim="draft.nickName"
              maxlength="64"
              placeholder="请输入昵称"
              @blur="trackDraftTouch('昵称')"
            />
          </div>
          <div class="field">
            <label>手机号</label>
            <input
              v-model.trim="draft.phone"
              maxlength="32"
              placeholder="请输入手机号"
              @blur="trackDraftTouch('手机号')"
            />
          </div>
          <div class="field full">
            <label>邮箱</label>
            <input
              v-model.trim="draft.email"
              maxlength="128"
              placeholder="请输入邮箱"
              @blur="trackDraftTouch('邮箱')"
            />
          </div>
        </div>

        <div class="banner info">
          <strong>当前状态：</strong>{{ profileStatusMessage }}
        </div>
        <div v-if="validationErrors.length > 0" class="banner danger">
          <strong>提交前检查：</strong>{{ validationErrors.join("；") }}
        </div>

        <div class="toolbar">
          <button class="btn primary" :disabled="savingProfile" @click="saveProfile">
            {{ savingProfile ? "保存中..." : "提交资料更新" }}
          </button>
          <button class="btn ghost" @click="saveDraftSnapshot">暂存本地草稿</button>
          <button class="btn ghost" :disabled="!hasStoredDraft" @click="restoreStoredDraft">
            回填本地草稿
          </button>
          <button class="btn ghost danger" :disabled="!hasStoredDraft" @click="clearStoredDraft">
            清空本地草稿
          </button>
        </div>

        <div class="section-block">
          <div class="section-head compact">
            <div>
              <h4>资料差异</h4>
              <p>提交前可直接查看本次将要同步的字段。</p>
            </div>
          </div>
          <div v-if="diffItems.length === 0" class="empty-state">当前草稿与已登录资料一致，无需额外同步。</div>
          <div v-else class="diff-list">
            <div v-for="item in diffItems" :key="item.label" class="diff-item">
              <span class="diff-label">{{ item.label }}</span>
              <div class="diff-values">
                <span class="before">{{ item.before || "未填写" }}</span>
                <span class="arrow">→</span>
                <span class="after">{{ item.after || "未填写" }}</span>
              </div>
            </div>
          </div>
        </div>
      </article>

      <article class="card">
        <div class="section-head">
          <div>
            <h3>角色概览</h3>
            <p>基于当前 auth 信息汇总角色、默认入口和权限覆盖情况。</p>
          </div>
          <span class="status-chip" :class="auth.userInfo?.enable === 1 ? 'success' : 'danger'">
            {{ auth.userInfo?.enable === 1 ? "账号启用" : "账号冻结" }}
          </span>
        </div>

        <div class="key-values">
          <div class="key-value">
            <span>主角色</span>
            <strong>{{ authoritySummary.primary }}</strong>
          </div>
          <div class="key-value">
            <span>默认路由</span>
            <strong>{{ auth.userInfo?.authority.defaultRouter ?? "-" }}</strong>
          </div>
          <div class="key-value">
            <span>权限路径数</span>
            <strong>{{ auth.policyPaths.length }}</strong>
          </div>
          <div class="key-value">
            <span>高频方法</span>
            <strong>{{ authoritySummary.topMethod }}</strong>
          </div>
        </div>

        <div class="tag-list">
          <span v-for="item in auth.userInfo?.authorities ?? []" :key="item.authorityId" class="tag">
            {{ item.authorityName }} / {{ item.authorityId }}
          </span>
        </div>

        <div class="section-block">
          <div class="section-head compact">
            <div>
              <h4>权限热点</h4>
              <p>根据当前权限包提取最常见方法与路径段，辅助巡检。</p>
            </div>
          </div>
          <div class="key-values mini">
            <div class="key-value">
              <span>最常见方法</span>
              <strong>{{ authoritySummary.topMethod }}</strong>
            </div>
            <div class="key-value">
              <span>热点路径段</span>
              <strong>{{ authoritySummary.topSegment }}</strong>
            </div>
            <div class="key-value">
              <span>备选角色数</span>
              <strong>{{ authoritySummary.alternativeCount }}</strong>
            </div>
          </div>
        </div>
      </article>
    </section>

    <section class="split-grid">
      <article class="card">
        <div class="section-head">
          <div>
            <h3>快捷偏好</h3>
            <p>本地维护首页动作、列表密度、提醒频率与个人值班摘要偏好。</p>
          </div>
          <span class="status-chip" :class="preferenceDirty ? 'warning' : 'success'">
            {{ preferenceDirty ? "偏好未保存" : "偏好已落盘" }}
          </span>
        </div>

        <div class="field-grid">
          <div class="field">
            <label>首页动作</label>
            <select v-model="preferences.homeAction">
              <option value="dashboard">dashboard</option>
              <option value="users">users</option>
              <option value="systemTools">systemTools</option>
              <option value="runtimeState">runtimeState</option>
            </select>
          </div>
          <div class="field">
            <label>列表密度</label>
            <select v-model="preferences.tableDensity">
              <option value="comfortable">comfortable</option>
              <option value="compact">compact</option>
            </select>
          </div>
          <div class="field">
            <label>提醒频率</label>
            <select v-model="preferences.digestFrequency">
              <option value="daily">daily</option>
              <option value="weekly">weekly</option>
              <option value="manual">manual</option>
            </select>
          </div>
          <div class="field">
            <label>个人视角</label>
            <select v-model="preferences.watchFocus">
              <option value="profile">profile</option>
              <option value="audit">audit</option>
              <option value="runtime">runtime</option>
            </select>
          </div>
        </div>

        <div class="toolbar">
          <button class="btn primary" :disabled="savingPreferences" @click="savePreferences">
            {{ savingPreferences ? "保存中..." : "保存本地偏好" }}
          </button>
          <button class="btn ghost" @click="restoreStoredPreferences">回填本地偏好</button>
        </div>

        <div class="section-block">
          <div class="section-head compact">
            <div>
              <h4>偏好摘要</h4>
              <p>用一句话快速描述当前工作模式，便于值班与交接。</p>
            </div>
          </div>
          <ul class="summary-list">
            <li v-for="item in preferenceSummary" :key="item.label">
              <span>{{ item.label }}</span>
              <strong>{{ item.value }}</strong>
            </li>
          </ul>
        </div>
      </article>

      <article class="card">
        <div class="section-head">
          <div>
            <h3>最近个人动作</h3>
            <p>记录资料编辑、草稿暂存、偏好回填等个人级操作，支持本地保留。</p>
          </div>
          <button class="btn ghost danger" :disabled="activityFeed.length === 0" @click="clearActivities">
            清空记录
          </button>
        </div>

        <div v-if="activityFeed.length === 0" class="empty-state">暂无个人动作记录，完成一次保存或草稿回填后会出现在这里。</div>
        <div v-else class="activity-list">
          <div v-for="item in activityFeed" :key="item.id" class="activity-item">
            <div>
              <strong>{{ item.label }}</strong>
              <p>{{ item.detail }}</p>
            </div>
            <span>{{ item.time }}</span>
          </div>
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { updateProfileApi } from "../api/admin";
import { useAuthStore } from "../stores/auth";

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
    auth.userInfo = user;
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
.profile-workbench {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card,
.stat-card {
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  background: #fff;
  padding: 20px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.05);
}

.hero {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
}

.eyebrow {
  margin: 0 0 8px;
  font-size: 12px;
  color: #2563eb;
  font-weight: 700;
}

.title {
  margin: 0;
  font-size: 24px;
  line-height: 1.4;
}

.subtitle {
  margin: 10px 0 0;
  color: #64748b;
  line-height: 1.6;
}

.hero-actions,
.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.summary-grid,
.split-grid,
.field-grid,
.key-values {
  display: grid;
  gap: 16px;
}

.summary-grid {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.split-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.field-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.field.full {
  grid-column: 1 / -1;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field label,
.stat-label,
.key-value span,
.summary-list span {
  font-size: 13px;
  color: #64748b;
}

.field input,
.field select {
  width: 100%;
  min-height: 40px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid #cbd5e1;
  outline: none;
  transition: border-color 0.2s ease;
}

.field input:focus,
.field select:focus {
  border-color: #2563eb;
}

.section-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 16px;
}

.section-head h3,
.section-head h4 {
  margin: 0;
}

.section-head p {
  margin: 6px 0 0;
  color: #64748b;
  line-height: 1.5;
}

.section-head.compact {
  margin-bottom: 12px;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.status-chip.success {
  background: #dcfce7;
  color: #166534;
}

.status-chip.warning {
  background: #fef3c7;
  color: #92400e;
}

.status-chip.danger {
  background: #fee2e2;
  color: #991b1b;
}

.stat-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.stat-value {
  font-size: 28px;
  line-height: 1;
}

.stat-note {
  color: #0f172a;
  font-size: 13px;
}

.banner {
  margin-top: 16px;
  padding: 12px 14px;
  border-radius: 12px;
  font-size: 14px;
}

.banner.info {
  background: #eff6ff;
  color: #1d4ed8;
}

.banner.danger {
  background: #fef2f2;
  color: #b91c1c;
}

.section-block {
  margin-top: 18px;
  padding-top: 18px;
  border-top: 1px solid #e5e7eb;
}

.diff-list,
.activity-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.diff-item,
.activity-item {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border-radius: 12px;
  background: #f8fafc;
}

.diff-label {
  min-width: 70px;
  font-weight: 700;
}

.diff-values {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  justify-content: flex-end;
}

.before {
  color: #64748b;
}

.after {
  color: #0f172a;
  font-weight: 700;
}

.arrow {
  color: #2563eb;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
}

.tag {
  display: inline-flex;
  align-items: center;
  min-height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  background: #eff6ff;
  color: #1d4ed8;
  font-size: 12px;
  font-weight: 700;
}

.key-values {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.key-values.mini {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.key-value {
  padding: 14px;
  border-radius: 12px;
  background: #f8fafc;
}

.key-value strong,
.summary-list strong {
  display: block;
  margin-top: 8px;
  font-size: 15px;
  color: #0f172a;
}

.summary-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.summary-list li {
  padding: 14px;
  border-radius: 12px;
  background: #f8fafc;
}

.activity-item p {
  margin: 6px 0 0;
  color: #64748b;
  line-height: 1.5;
}

.activity-item span {
  flex-shrink: 0;
  color: #64748b;
  font-size: 12px;
}

.empty-state {
  padding: 20px 14px;
  border-radius: 12px;
  background: #f8fafc;
  color: #64748b;
  text-align: center;
}

.btn {
  min-height: 38px;
  padding: 0 14px;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  background: #fff;
  cursor: pointer;
  font-weight: 700;
}

.btn.primary {
  background: #2563eb;
  border-color: #2563eb;
  color: #fff;
}

.btn.ghost {
  color: #0f172a;
}

.btn.danger {
  color: #b91c1c;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

@media (max-width: 1024px) {
  .summary-grid,
  .split-grid,
  .field-grid,
  .key-values,
  .key-values.mini,
  .summary-list {
    grid-template-columns: 1fr;
  }

  .hero,
  .section-head,
  .diff-item,
  .activity-item {
    flex-direction: column;
  }

  .diff-values {
    justify-content: flex-start;
  }
}
</style>
