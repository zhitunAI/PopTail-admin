<script lang="ts" setup>
import type { MenuButtonInfo, MenuInfo, MenuParameterInfo } from '#/types/pop-tail';

import type { VbenFormSchema } from '#/adapter/form';
import type { MenuRow, TemplateMenuType } from '../types';

import { computed, nextTick, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import { breakpointsTailwind, useBreakpoints } from '@vueuse/core';
import { Button, Input, Select } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { saveMenuApi } from '#/api/pop-tail/admin';

import { getMenuTypeOptions } from '../data';

const emit = defineEmits<{
  success: [];
}>();

type MenuFormValues = {
  authCode?: string;
  component?: string;
  hidden?: boolean;
  linkSrc?: string;
  meta?: {
    affixTab?: boolean;
    hideChildrenInMenu?: boolean;
    hideInBreadcrumb?: boolean;
    hideInMenu?: boolean;
    hideInTab?: boolean;
    icon?: string;
    keepAlive?: boolean;
    title?: string;
  };
  name: string;
  parentId?: number;
  path?: string;
  sort?: number;
  status?: number;
  type: TemplateMenuType;
};

const formData = ref<MenuRow>();
const sourceMenu = computed(() => formData.value?.menu);
const parameterRows = ref<MenuParameterInfo[]>([]);
const buttonRows = ref<MenuButtonInfo[]>([]);
const componentOptions = [
  'views/admin/SystemOverviewView.vue',
  'views/admin/UsersView.vue',
  'views/admin/AuthoritiesView.vue',
  'views/admin/MenusView.vue',
  'views/admin/ApisView.vue',
  'views/admin/IconGalleryView.vue',
  'views/admin/DictionariesView.vue',
  'views/admin/DictionaryDetailView.vue',
  'views/admin/ParamsView.vue',
  'views/admin/OperationLogsView.vue',
  'views/admin/LoginLogsView.vue',
  'views/admin/SystemStateView.vue',
  'views/admin/SystemToolsView.vue',
  'views/tools/AiWorkflowView.vue',
  'views/tools/ApiTokensView.vue',
  'views/tools/LlmConfigView.vue',
  'views/tools/SkillsView.vue',
  'views/tools/SystemConfigView.vue',
  'views/tools/EmailPluginView.vue',
  'views/tools/AnnouncementView.vue',
].map((value) => ({ value }));

const parentOptions = ref<Array<{ label: string; value: number }>>([]);

const schema: VbenFormSchema[] = [
  {
    component: 'RadioGroup',
    componentProps: {
      buttonStyle: 'solid',
      optionType: 'button',
      options: getMenuTypeOptions(),
    },
    defaultValue: 'menu',
    fieldName: 'type',
    formItemClass: 'col-span-2 md:col-span-2',
    label: '菜单类型',
  },
  {
    component: 'Input',
    fieldName: 'name',
    label: '路由名称',
    rules: 'required',
  },
  {
    component: 'Select',
    componentProps() {
      return {
        allowClear: true,
        class: 'w-full',
        options: parentOptions.value,
        showSearch: true,
      };
    },
    fieldName: 'parentId',
    label: '上级菜单',
  },
  {
    component: 'Input',
    fieldName: 'meta.title',
    label: '菜单名称',
    rules: 'required',
  },
  {
    component: 'Input',
    dependencies: {
      show: (values) => ['catalog', 'embedded', 'menu'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'path',
    label: '路由路径',
    rules: 'required',
  },
  {
    component: 'Input',
    dependencies: {
      show: (values) => ['embedded', 'menu'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'activePath',
    label: '激活菜单',
  },
  {
    component: 'IconPicker',
    componentProps: {
      prefix: 'carbon',
    },
    dependencies: {
      show: (values) => ['catalog', 'embedded', 'link', 'menu'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'meta.icon',
    label: '图标',
  },
  {
    component: 'IconPicker',
    componentProps: {
      prefix: 'carbon',
    },
    dependencies: {
      show: (values) => ['catalog', 'embedded', 'menu'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'meta.activeIcon',
    label: '激活图标',
  },
  {
    component: 'AutoComplete',
    componentProps: {
      allowClear: true,
      class: 'w-full',
      filterOption(input: string, option: { value: string }) {
        return option.value.toLowerCase().includes(input.toLowerCase());
      },
      options: componentOptions,
    },
    dependencies: {
      show: (values) => values.type === 'menu',
      triggerFields: ['type'],
    },
    fieldName: 'component',
    label: '组件路径',
    rules: 'required',
  },
  {
    component: 'Input',
    dependencies: {
      show: (values) => ['embedded', 'link'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'linkSrc',
    label: '链接地址',
  },
  {
    component: 'Input',
    dependencies: {
      show: (values) => ['button', 'catalog', 'embedded', 'menu'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'authCode',
    label: '权限标识',
  },
  {
    component: 'RadioGroup',
    componentProps: {
      buttonStyle: 'solid',
      optionType: 'button',
      options: [
        { label: '启用', value: 1 },
        { label: '禁用', value: 0 },
      ],
    },
    defaultValue: 1,
    fieldName: 'status',
    label: '状态',
  },
  {
    component: 'InputNumber',
    componentProps: {
      class: 'w-full',
    },
    fieldName: 'sort',
    label: '排序',
  },
  {
    component: 'Divider',
    dependencies: {
      show: (values) => !['button', 'link'].includes(values.type),
      triggerFields: ['type'],
    },
    fieldName: 'divider1',
    formItemClass: 'col-span-2 md:col-span-2 pb-0',
    hideLabel: true,
    renderComponentContent() {
      return {
        default: () => '高级设置',
      };
    },
  },
  checkboxField('meta.keepAlive', '缓存页面', ['menu']),
  checkboxField('meta.affixTab', '固定标签页', ['embedded', 'menu']),
  checkboxField('meta.hideInMenu', '菜单隐藏', ['catalog', 'embedded', 'link', 'menu']),
  checkboxField('meta.hideChildrenInMenu', '隐藏子菜单', ['catalog', 'menu']),
  checkboxField('meta.hideInBreadcrumb', '面包屑隐藏', ['catalog', 'embedded', 'menu']),
  checkboxField('meta.hideInTab', '标签页隐藏', ['catalog', 'embedded', 'menu']),
];

function checkboxField(fieldName: string, label: string, types: string[]): VbenFormSchema {
  return {
    component: 'Checkbox',
    dependencies: {
      show: (values) => types.includes(values.type),
      triggerFields: ['type'],
    },
    fieldName,
    renderComponentContent() {
      return {
        default: () => label,
      };
    },
  };
}

const breakpoints = useBreakpoints(breakpointsTailwind);
const isHorizontal = computed(() => breakpoints.greaterOrEqual('md').value);

const [Form, formApi] = useVbenForm({
  commonConfig: {
    colon: true,
    formItemClass: 'col-span-2 md:col-span-1',
  },
  schema,
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2 gap-x-4',
});

const [Drawer, drawerApi] = useVbenDrawer({
  onConfirm: onSubmit,
  async onOpenChange(isOpen) {
    if (!isOpen) return;
    const data = drawerApi.getData<MenuRow & { parentMenus?: MenuInfo[]; pid?: string }>();
    setParentOptions(data?.parentMenus ?? []);
    formData.value = data?.menu ? data : undefined;
    parameterRows.value = (data?.menu?.parameters ?? []).map((item) => ({ ...item }));
    buttonRows.value = (data?.menu?.menuBtn ?? []).map((item) => ({ ...item }));
    formApi.resetForm();
    await nextTick();
    formApi.setValues(toFormValues(data));
  },
});

async function onSubmit() {
  const { valid } = await formApi.validate();
  if (!valid) return;
  drawerApi.lock();
  try {
    const values = await formApi.getValues<MenuFormValues>();
    const source = sourceMenu.value;
    await saveMenuApi({
      ID: source?.ID,
      component: resolveComponent(values),
      hidden: values.status === 0 || Boolean(values.meta?.hideInMenu),
      menuBtn: normalizeButtons(source?.ID, buttonRows.value),
      meta: {
        ...(source?.meta ?? {}),
        ...(values.meta ?? {}),
        activeName: values.type === 'link' ? 'link' : undefined,
        icon: values.meta?.icon ?? '',
        title: values.meta?.title ?? values.name,
      },
      name: values.name,
      parameters: normalizeParameters(source?.ID, parameterRows.value),
      parentId: Number(values.parentId ?? 0),
      path: values.path || values.linkSrc || `/${values.name}`,
      sort: Number(values.sort ?? source?.sort ?? 0),
    });
    drawerApi.close();
    emit('success');
  } finally {
    drawerApi.unlock();
  }
}

function resolveComponent(values: MenuFormValues) {
  if (values.type === 'embedded') return values.linkSrc || '';
  if (values.type === 'link') return '';
  if (values.type === 'catalog') return '';
  return values.component ?? '';
}

function toFormValues(data?: MenuRow & { pid?: string }): Partial<MenuFormValues> {
  const source = data?.menu;
  const type = data?.type ?? 'menu';
  return {
    authCode: source?.name ?? '',
    component: source?.component ?? '',
    linkSrc: type === 'link' ? source?.path : source?.component,
    name: source?.name ?? '',
    parentId: Number(data?.pid ?? source?.parentId ?? 0),
    path: source?.path ?? '',
    sort: source?.sort ?? 0,
    status: source?.hidden ? 0 : 1,
    type,
    meta: {
      ...(source?.meta ?? {}),
      hideInMenu: Boolean(source?.hidden),
      title: source?.meta?.title ?? '',
    },
  };
}

function setParentOptions(menus: MenuInfo[]) {
  const rows: Array<{ label: string; value: number }> = [{ label: '根菜单', value: 0 }];

  function walk(list: MenuInfo[], level = 0) {
    for (const item of list) {
      rows.push({
        label: `${'— '.repeat(level)}${item.meta?.title || item.name}`,
        value: item.ID,
      });
      if (Array.isArray(item.children) && item.children.length > 0) {
        walk(item.children, level + 1);
      }
    }
  }

  walk(menus);
  parentOptions.value = rows;
}

function addParameterRow() {
  parameterRows.value.push({
    ID: 0,
    key: '',
    sysBaseMenuID: sourceMenu.value?.ID ?? 0,
    type: 'query',
    value: '',
  });
}

function removeParameterRow(index: number) {
  parameterRows.value.splice(index, 1);
}

function addButtonRow() {
  buttonRows.value.push({
    ID: 0,
    desc: '',
    name: '',
    sysBaseMenuID: sourceMenu.value?.ID ?? 0,
  });
}

function removeButtonRow(index: number) {
  buttonRows.value.splice(index, 1);
}

function normalizeParameters(menuId: number | undefined, rows: MenuParameterInfo[]) {
  return rows
    .map((item) => ({
      ...item,
      ID: Number(item.ID ?? 0),
      key: item.key.trim(),
      sysBaseMenuID: menuId ?? Number(item.sysBaseMenuID ?? 0),
      type: item.type || 'query',
      value: item.value.trim(),
    }))
    .filter((item) => item.key);
}

function normalizeButtons(menuId: number | undefined, rows: MenuButtonInfo[]) {
  return rows
    .map((item) => ({
      ...item,
      ID: Number(item.ID ?? 0),
      desc: item.desc.trim(),
      name: item.name.trim(),
      sysBaseMenuID: menuId ?? Number(item.sysBaseMenuID ?? 0),
    }))
    .filter((item) => item.name);
}

const drawerTitle = computed(() => (formData.value?.id ? '编辑菜单' : '新增菜单'));
</script>

<template>
  <Drawer class="w-full max-w-200" :title="drawerTitle">
    <div class="mx-4 space-y-6">
      <Form :layout="isHorizontal ? 'horizontal' : 'vertical'" />

      <section class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-base font-semibold">菜单参数配置</h3>
          <Button type="primary" @click="addParameterRow">新增菜单参数</Button>
        </div>
        <div class="rounded-lg border border-gray-200 p-3">
          <div class="mb-2 grid grid-cols-[140px_1fr_1fr_auto] gap-3 text-sm font-medium text-gray-500">
            <span>参数类型</span>
            <span>参数key</span>
            <span>参数值</span>
            <span>操作</span>
          </div>
          <div v-if="parameterRows.length === 0" class="py-8 text-center text-gray-400">暂无数据</div>
          <div
            v-for="(item, index) in parameterRows"
            :key="`${item.ID}-${index}`"
            class="mb-2 grid grid-cols-[140px_1fr_1fr_auto] gap-3"
          >
            <Select
              v-model:value="item.type"
              :options="[
                { label: 'query', value: 'query' },
                { label: 'param', value: 'param' },
                { label: 'dynamic', value: 'dynamic' },
              ]"
            />
            <Input v-model:value="item.key" placeholder="参数 key" />
            <Input v-model:value="item.value" placeholder="参数值" />
            <Button danger @click="removeParameterRow(index)">删除</Button>
          </div>
        </div>
      </section>

      <section class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-base font-semibold">可控按钮配置</h3>
          <Button type="primary" @click="addButtonRow">新增可控按钮</Button>
        </div>
        <div class="rounded-lg border border-gray-200 p-3">
          <div class="mb-2 grid grid-cols-[1fr_1fr_auto] gap-3 text-sm font-medium text-gray-500">
            <span>按钮名称</span>
            <span>备注</span>
            <span>操作</span>
          </div>
          <div v-if="buttonRows.length === 0" class="py-8 text-center text-gray-400">暂无数据</div>
          <div
            v-for="(item, index) in buttonRows"
            :key="`${item.ID}-${index}`"
            class="mb-2 grid grid-cols-[1fr_1fr_auto] gap-3"
          >
            <Input v-model:value="item.name" placeholder="如：新增 / 编辑 / 删除 / 查看" />
            <Input v-model:value="item.desc" placeholder="按钮备注" />
            <Button danger @click="removeButtonRow(index)">删除</Button>
          </div>
        </div>
      </section>
    </div>
  </Drawer>
</template>
