<template>
  <div class="stack">
    <div class="card">
      <h3 class="title">客户示例</h3>
      <p class="subtitle">恢复客户示例的增删改查流程，数据通过 Rust 后端维护。</p>
      <div class="toolbar-grid">
        <div class="field">
          <label>客户名称</label>
          <input v-model="keyword" placeholder="输入客户名称、状态或等级" @keyup.enter="loadCustomers" />
        </div>
        <div class="field">
          <label>说明</label>
          <p class="subtitle">当前页与原系统一样保留“客户台账 + 单条详情 + 编辑入口”的联动结构。</p>
        </div>
      </div>
      <div class="row">
        <button class="btn ghost" @click="loadCustomers">查询</button>
        <button class="btn primary" @click="startCreate">新增客户</button>
      </div>
      <p v-if="message" class="subtitle">{{ message }}</p>
    </div>

    <div class="split-grid">
      <div class="card">
        <h3 class="title">客户列表</h3>
        <div class="data-table">
          <table>
            <thead>
              <tr>
                <th>接入时间</th>
                <th>客户</th>
                <th>电话</th>
                <th>等级</th>
                <th>状态</th>
                <th>归属用户</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in rows" :key="item.ID">
                <td>{{ formatTime(item.CreatedAt) }}</td>
                <td>{{ item.customerName }}</td>
                <td>{{ item.customerPhoneData }}</td>
                <td>{{ item.customerLevel }}</td>
                <td>{{ item.customerStatus }}</td>
                <td>{{ item.sysUserId }}</td>
                <td>
                  <div class="row">
                    <button class="btn ghost" @click="openDetail(item.ID)">查看</button>
                    <button class="btn ghost" @click="startEdit(item)">编辑</button>
                    <button class="btn ghost" @click="remove(item.ID)">删除</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="7">暂无客户记录</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="stack">
        <div class="card">
          <h3 class="title">客户详情</h3>
          <div v-if="selected" class="data-table">
            <table>
              <tbody>
                <tr><td>客户名</td><td>{{ selected.customerName }}</td></tr>
                <tr><td>电话</td><td>{{ selected.customerPhoneData }}</td></tr>
                <tr><td>等级</td><td>{{ selected.customerLevel }}</td></tr>
                <tr><td>状态</td><td>{{ selected.customerStatus }}</td></tr>
                <tr><td>归属用户</td><td>{{ selected.sysUserId }}</td></tr>
                <tr><td>备注</td><td>{{ selected.remark || "-" }}</td></tr>
                <tr><td>更新时间</td><td>{{ formatTime(selected.UpdatedAt) }}</td></tr>
              </tbody>
            </table>
          </div>
          <p v-else class="subtitle">请选择一条客户记录查看详情。</p>
        </div>

        <div class="card">
          <h3 class="title">{{ mode === "edit" ? "编辑客户" : "新增客户" }}</h3>
          <div class="toolbar-grid">
            <div class="field">
              <label>客户名</label>
              <input v-model="draft.customerName" />
            </div>
            <div class="field">
              <label>联系电话</label>
              <input v-model="draft.customerPhoneData" />
            </div>
            <div class="field">
              <label>等级</label>
              <select v-model="draft.customerLevel">
                <option value="A">A</option>
                <option value="B">B</option>
                <option value="C">C</option>
              </select>
            </div>
            <div class="field">
              <label>状态</label>
              <select v-model="draft.customerStatus">
                <option value="跟进中">跟进中</option>
                <option value="已签约">已签约</option>
                <option value="暂停中">暂停中</option>
              </select>
            </div>
          </div>
          <div class="field">
            <label>备注</label>
            <textarea v-model="draft.remark" rows="4" />
          </div>
          <div class="row">
            <button class="btn primary" @click="submit">{{ mode === "edit" ? "保存变更" : "创建客户" }}</button>
            <button class="btn ghost" @click="resetDraft">重置</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import {
  createCustomerApi,
  deleteCustomerApi,
  getCustomerDetailApi,
  getCustomerListApi,
  updateCustomerApi,
} from "../../api/admin";
import type { CustomerRecord } from "../../types";

const rows = ref<CustomerRecord[]>([]);
const selected = ref<CustomerRecord | null>(null);
const keyword = ref("");
const message = ref("");
const mode = ref<"create" | "edit">("create");
const draft = reactive({
  ID: 0,
  customerName: "",
  customerPhoneData: "",
  customerLevel: "A",
  customerStatus: "跟进中",
  remark: "",
});

async function loadCustomers() {
  const result = await getCustomerListApi({ keyword: keyword.value.trim() });
  rows.value = result.List;
  if (selected.value) {
    const next = rows.value.find((item) => item.ID === selected.value?.ID);
    if (next) {
      selected.value = next;
    }
  }
}

async function openDetail(id: number) {
  selected.value = await getCustomerDetailApi(id);
}

function startCreate() {
  mode.value = "create";
  resetDraft();
}

function startEdit(item: CustomerRecord) {
  mode.value = "edit";
  draft.ID = item.ID;
  draft.customerName = item.customerName;
  draft.customerPhoneData = item.customerPhoneData;
  draft.customerLevel = item.customerLevel;
  draft.customerStatus = item.customerStatus;
  draft.remark = item.remark;
}

async function submit() {
  if (!draft.customerName.trim() || !draft.customerPhoneData.trim()) {
    message.value = "请填写客户名称和电话";
    return;
  }

  const payload = {
    customerName: draft.customerName.trim(),
    customerPhoneData: draft.customerPhoneData.trim(),
    customerLevel: draft.customerLevel,
    customerStatus: draft.customerStatus,
    remark: draft.remark.trim(),
  };

  if (mode.value === "edit" && draft.ID) {
    const saved = await updateCustomerApi({ ID: draft.ID, ...payload });
    selected.value = saved;
    message.value = `客户 ${saved.customerName} 已更新`;
  } else {
    const created = await createCustomerApi(payload);
    selected.value = created;
    message.value = `客户 ${created.customerName} 已创建`;
  }
  await loadCustomers();
  resetDraft();
}

async function remove(id: number) {
  await deleteCustomerApi(id);
  if (selected.value?.ID === id) {
    selected.value = null;
  }
  message.value = `客户 #${id} 已删除`;
  await loadCustomers();
}

function resetDraft() {
  draft.ID = 0;
  draft.customerName = "";
  draft.customerPhoneData = "";
  draft.customerLevel = "A";
  draft.customerStatus = "跟进中";
  draft.remark = "";
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN");
}

onMounted(async () => {
  await loadCustomers();
  if (rows.value[0]) {
    await openDetail(rows.value[0].ID);
  }
});
</script>
