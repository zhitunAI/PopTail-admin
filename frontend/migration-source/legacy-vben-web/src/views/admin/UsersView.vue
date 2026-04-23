<template>
  <div class="stack">
    <div class="card">
      <div class="row between wrap">
        <div>
          <h3 class="title">用户工作台</h3>
          <p class="subtitle">按账号、联系方式和角色筛选用户，并直接完成启停切换、详情联动和草稿维护。</p>
        </div>
        <div class="row wrap">
          <button class="btn ghost" :disabled="loading" @click="load">刷新用户</button>
          <button class="btn primary" @click="openCreate">新增用户</button>
        </div>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>

    <div class="muted-grid">
      <div class="stat-card">
        <h4>用户总数</h4>
        <p class="stat-value">{{ rows.length }}</p>
        <p class="subtitle">当前命中筛选的用户数</p>
      </div>
      <div class="stat-card">
        <h4>启用用户</h4>
        <p class="stat-value">{{ enabledCount }}</p>
        <p class="subtitle">冻结 {{ rows.length - enabledCount }} 个</p>
      </div>
      <div class="stat-card">
        <h4>角色种类</h4>
        <p class="stat-value">{{ authorityNames.length }}</p>
        <p class="subtitle">已出现在用户列表中的主角色数</p>
      </div>
      <div class="stat-card">
        <h4>当前查看</h4>
        <p class="stat-value">{{ activeUser?.userName || "-" }}</p>
        <p class="subtitle">{{ activeUser ? userAdvice(activeUser) : "请选择用户查看详情" }}</p>
      </div>
    </div>

    <div class="card">
      <h3 class="title">筛选与用户列表</h3>
      <div class="toolbar-grid">
        <div class="field">
          <label>用户名</label>
          <input v-model.trim="filters.username" />
        </div>
        <div class="field">
          <label>昵称</label>
          <input v-model.trim="filters.nickName" />
        </div>
        <div class="field">
          <label>手机号</label>
          <input v-model.trim="filters.phone" />
        </div>
        <div class="field">
          <label>邮箱</label>
          <input v-model.trim="filters.email" />
        </div>
      </div>
      <div class="row wrap">
        <button class="btn ghost" @click="load">查询</button>
        <button class="btn ghost" @click="reset">重置</button>
        <button class="btn ghost" :disabled="!activeUser" @click="copySummary">复制用户摘要</button>
      </div>

      <div class="data-table">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>用户名</th>
              <th>昵称</th>
              <th>手机号</th>
              <th>邮箱</th>
              <th>主角色</th>
              <th>状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in rows"
              :key="item.ID"
              :class="{ selected: activeUser?.ID === item.ID }"
              @click="viewUser(item)"
            >
              <td>{{ item.ID }}</td>
              <td>{{ item.userName }}</td>
              <td>{{ item.nickName }}</td>
              <td>{{ item.phone }}</td>
              <td>{{ item.email }}</td>
              <td>{{ item.authority.authorityName }}</td>
              <td>{{ item.enable === 1 ? "正常" : "冻结" }}</td>
              <td class="row wrap">
                <button class="btn ghost" @click.stop="viewUser(item)">详情</button>
                <button class="btn ghost" @click.stop="toggleUser(item)">
                  {{ item.enable === 1 ? "冻结" : "启用" }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">{{ drawerMode === "create" ? "新增用户" : "用户详情" }}</h3>
        <div class="toolbar-grid">
          <div class="field">
            <label>用户名</label>
            <input v-model.trim="form.userName" :disabled="drawerMode !== 'create'" />
          </div>
          <div class="field">
            <label>昵称</label>
            <input v-model.trim="form.nickName" />
          </div>
          <div class="field">
            <label>手机号</label>
            <input v-model.trim="form.phone" />
          </div>
          <div class="field">
            <label>邮箱</label>
            <input v-model.trim="form.email" />
          </div>
          <div class="field">
            <label>主角色</label>
            <select v-model.number="form.authorityId">
              <option v-for="item in authorities" :key="item.authorityId" :value="item.authorityId">
                {{ item.authorityName }}
              </option>
            </select>
          </div>
          <div class="field">
            <label>状态</label>
            <select v-model.number="form.enable">
              <option :value="1">正常</option>
              <option :value="2">冻结</option>
            </select>
          </div>
        </div>

        <div v-if="validationIssues.length" class="state-banner error">
          <strong>保存前请修正：</strong>
          <ul>
            <li v-for="item in validationIssues" :key="item">{{ item }}</li>
          </ul>
        </div>

        <div class="row wrap">
          <button class="btn primary" :disabled="saving || validationIssues.length > 0" @click="save">
            {{ saving ? "保存中..." : "保存用户" }}
          </button>
          <button class="btn ghost" :disabled="!activeUser" @click="resetCurrentUser">重置当前内容</button>
        </div>
      </div>

      <div class="card">
        <h3 class="title">用户概览</h3>
        <div v-if="activeUser" class="data-table">
          <table>
            <tbody>
              <tr><td>用户名</td><td>{{ activeUser.userName }}</td></tr>
              <tr><td>昵称</td><td>{{ activeUser.nickName }}</td></tr>
              <tr><td>主角色</td><td>{{ activeUser.authority.authorityName }}</td></tr>
              <tr><td>状态</td><td>{{ activeUser.enable === 1 ? "正常" : "冻结" }}</td></tr>
              <tr><td>可切换角色</td><td>{{ activeUser.authorities.map((item) => item.authorityName).join(", ") || "-" }}</td></tr>
              <tr><td>建议</td><td>{{ userAdvice(activeUser) }}</td></tr>
            </tbody>
          </table>
        </div>
        <p v-else class="subtitle">请选择一个用户查看详情。</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { getAuthorityListApi, getUserListApi, saveUserApi } from "../../api/admin";
import type { AuthorityInfo, UserInfo } from "../../types";

const rows = ref<UserInfo[]>([]);
const authorities = ref<AuthorityInfo[]>([]);
const activeUser = ref<UserInfo | null>(null);
const drawerMode = ref<"view" | "create">("view");
const message = ref("");
const error = ref("");
const loading = ref(false);
const saving = ref(false);
const filters = reactive({
  username: "",
  nickName: "",
  phone: "",
  email: "",
});
const form = reactive({
  ID: undefined as number | undefined,
  userName: "",
  nickName: "",
  authorityId: 888,
  phone: "",
  email: "",
  enable: 1,
});

const authorityNames = computed(() =>
  Array.from(new Set(rows.value.map((item) => item.authority.authorityName))),
);
const enabledCount = computed(() => rows.value.filter((item) => item.enable === 1).length);
const validationIssues = computed(() => {
  const issues: string[] = [];
  if (!form.userName.trim()) issues.push("用户名不能为空。");
  if (!form.nickName.trim()) issues.push("昵称不能为空。");
  if (!form.authorityId) issues.push("请选择主角色。");
  return issues;
});

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const [users, authorityList] = await Promise.all([
      getUserListApi(filters),
      getAuthorityListApi(),
    ]);
    rows.value = users.List;
    authorities.value = authorityList;
    if (activeUser.value) {
      const refreshed = rows.value.find((item) => item.ID === activeUser.value?.ID);
      if (refreshed) {
        activeUser.value = refreshed;
        fillForm(refreshed);
      }
    }
    message.value = `已加载 ${rows.value.length} 个用户。`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "获取用户失败";
  } finally {
    loading.value = false;
  }
}

function reset() {
  filters.username = "";
  filters.nickName = "";
  filters.phone = "";
  filters.email = "";
  void load();
}

function viewUser(user: UserInfo) {
  drawerMode.value = "view";
  activeUser.value = user;
  fillForm(user);
}

function openCreate() {
  drawerMode.value = "create";
  message.value = "";
  const authority = authorities.value[0];
  const draft = {
    ID: 0,
    uuid: crypto.randomUUID(),
    userName: "new.user",
    nickName: "新用户",
    authorityId: authority?.authorityId ?? 888,
    authority: authority ?? {
      ID: 0,
      authorityId: 888,
      authorityName: "访客",
      defaultRouter: "dashboard",
      parentId: 0,
      children: [],
    },
    authorities: authority ? [authority] : [],
    headerImg: "",
    phone: "",
    email: "",
    enable: 1,
  };
  activeUser.value = draft;
  fillForm(draft);
}

function fillForm(user: UserInfo) {
  form.ID = drawerMode.value === "create" ? undefined : user.ID;
  form.userName = user.userName;
  form.nickName = user.nickName;
  form.authorityId = user.authorityId;
  form.phone = user.phone;
  form.email = user.email;
  form.enable = user.enable;
}

function userAdvice(user: UserInfo) {
  if (user.enable !== 1) return "用户当前处于冻结状态，建议确认是否仍需保留角色权限。";
  if ((user.authorities?.length ?? 0) > 1) return "该用户可切换多个角色，建议验证默认入口与授权范围。";
  return "常规用户，可继续检查联系方式与角色配置。";
}

async function toggleUser(user: UserInfo) {
  const saved = await saveUserApi({
    ID: user.ID,
    userName: user.userName,
    nickName: user.nickName,
    authorityId: user.authorityId,
    phone: user.phone,
    email: user.email,
    enable: user.enable === 1 ? 2 : 1,
  });
  message.value = `用户 ${saved.userName} 状态已更新`;
  activeUser.value = saved;
  fillForm(saved);
  await load();
}

async function copySummary() {
  if (!activeUser.value) return;
  const text = [
    `用户：${activeUser.value.userName}`,
    `昵称：${activeUser.value.nickName}`,
    `角色：${activeUser.value.authority.authorityName}`,
    `状态：${activeUser.value.enable === 1 ? "正常" : "冻结"}`,
  ].join("\n");
  try {
    await navigator.clipboard.writeText(text);
    message.value = `已复制用户 ${activeUser.value.userName} 摘要。`;
  } catch {
    window.prompt("当前环境不支持自动复制，请手动复制：", text);
    message.value = `已切换为手动复制用户 ${activeUser.value.userName} 摘要。`;
  }
}

async function save() {
  if (validationIssues.value.length) {
    error.value = validationIssues.value[0] ?? "用户信息不完整";
    return;
  }
  saving.value = true;
  error.value = "";
  try {
    const saved = await saveUserApi({ ...form });
    drawerMode.value = "view";
    activeUser.value = saved;
    fillForm(saved);
    message.value = `用户 ${saved.userName} 已保存`;
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "保存用户失败";
  } finally {
    saving.value = false;
  }
}

function resetCurrentUser() {
  if (!activeUser.value) return;
  fillForm(activeUser.value);
}

onMounted(() => {
  void load();
});
</script>
