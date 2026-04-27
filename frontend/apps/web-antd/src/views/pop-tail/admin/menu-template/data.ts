import type { VxeTableGridColumns } from '#/adapter/vxe-table';

import type { MenuRow } from './types';

export function getMenuTypeOptions() {
  return [
    { color: 'processing', label: '目录', value: 'catalog' },
    { color: 'default', label: '菜单', value: 'menu' },
    { color: 'error', label: '按钮', value: 'button' },
    { color: 'success', label: '内嵌', value: 'embedded' },
    { color: 'warning', label: '外链', value: 'link' },
  ];
}

export function useColumns(): VxeTableGridColumns<MenuRow> {
  return [
    {
      align: 'left',
      field: 'meta.title',
      fixed: 'left',
      slots: { default: 'title' },
      title: '菜单名称',
      treeNode: true,
      width: 260,
    },
    {
      align: 'center',
      field: 'type',
      slots: { default: 'type' },
      title: '类型',
      width: 100,
    },
    {
      field: 'authCode',
      slots: { default: 'authCode' },
      title: '权限标识',
      width: 200,
    },
    {
      align: 'left',
      field: 'path',
      slots: { default: 'path' },
      title: '路由路径',
      width: 200,
    },
    {
      align: 'center',
      field: 'sort',
      slots: { default: 'sort' },
      title: '排序',
      width: 180,
    },
    {
      align: 'left',
      field: 'component',
      minWidth: 320,
      slots: { default: 'component' },
      title: '组件/地址',
    },
    {
      align: 'center',
      field: 'status',
      slots: { default: 'status' },
      title: '状态',
      width: 100,
    },
    {
      align: 'right',
      field: 'operation',
      fixed: 'right',
      headerAlign: 'center',
      showOverflow: false,
      slots: { default: 'operation' },
      title: '操作',
      width: 260,
    },
  ];
}
