import type { VbenFormSchema } from '#/adapter/form';
import type { VxeTableGridColumns } from '#/adapter/vxe-table';

import type { RoleRow } from './types';

export function useFormSchema(): VbenFormSchema[] {
  return [
    {
      component: 'Input',
      fieldName: 'authorityName',
      label: '角色名称',
      rules: 'required',
    },
    {
      component: 'InputNumber',
      componentProps: {
        class: 'w-full',
        min: 1,
      },
      fieldName: 'authorityId',
      label: '角色ID',
      rules: 'required',
    },
    {
      component: 'Input',
      fieldName: 'defaultRouter',
      label: '默认路由',
    },
    {
      component: 'InputNumber',
      componentProps: {
        class: 'w-full',
        min: 0,
      },
      fieldName: 'parentId',
      label: '父角色ID',
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
  ];
}

export function useGridFormSchema(): VbenFormSchema[] {
  return [
    {
      component: 'Input',
      fieldName: 'name',
      label: '角色名称',
    },
    {
      component: 'Input',
      fieldName: 'id',
      label: '角色ID',
    },
    {
      component: 'Select',
      componentProps: {
        allowClear: true,
        options: [
          { label: '启用', value: 1 },
          { label: '禁用', value: 0 },
        ],
      },
      fieldName: 'status',
      label: '状态',
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: '备注',
    },
    {
      component: 'RangePicker',
      fieldName: 'createTime',
      label: '创建时间',
    },
  ];
}

export function useColumns(): VxeTableGridColumns<RoleRow> {
  return [
    {
      field: 'name',
      title: '角色名称',
      width: 200,
    },
    {
      field: 'id',
      title: '角色ID',
      width: 200,
    },
    {
      field: 'status',
      slots: { default: 'status' },
      title: '状态',
      width: 120,
    },
    {
      field: 'remark',
      minWidth: 240,
      title: '备注',
    },
    {
      field: 'createTime',
      title: '创建时间',
      width: 200,
    },
    {
      align: 'center',
      field: 'operation',
      fixed: 'right',
      slots: { default: 'operation' },
      title: '操作',
      width: 150,
    },
  ];
}
