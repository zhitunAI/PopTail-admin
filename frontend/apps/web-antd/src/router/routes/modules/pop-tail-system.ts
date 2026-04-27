import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:shield',
      order: -9,
      title: '系统管理',
    },
    name: 'GaaSystem',
    path: '/system',
    children: [
      {
        name: 'systemOverview',
        path: '/system/overview',
        component: () => import('#/views/pop-tail/admin/SystemOverviewView.vue'),
        meta: { icon: 'lucide:monitor-smartphone', title: '管理总览' },
      },
      {
        name: 'users',
        path: '/system/users',
        component: () => import('#/views/pop-tail/admin/UsersView.vue'),
        meta: { icon: 'lucide:users', title: '用户管理' },
      },
      {
        name: 'authorities',
        path: '/system/authorities',
        component: () => import('#/views/pop-tail/admin/AuthoritiesView.vue'),
        meta: { icon: 'lucide:key-round', title: '角色管理' },
      },
      {
        name: 'menus',
        path: '/system/menus',
        component: () => import('#/views/pop-tail/admin/MenusView.vue'),
        meta: { icon: 'lucide:menu-square', keepAlive: true, title: '菜单管理' },
      },
      {
        name: 'apis',
        path: '/system/apis',
        component: () => import('#/views/pop-tail/admin/ApisView.vue'),
        meta: { icon: 'lucide:waypoints', keepAlive: true, title: '接口管理' },
      },
      {
        name: 'menuIcons',
        path: '/system/icons',
        component: () => import('#/views/pop-tail/admin/IconGalleryView.vue'),
        meta: { icon: 'lucide:app-window', title: '图标库' },
      },
      {
        name: 'dictionaries',
        path: '/system/dictionaries',
        component: () => import('#/views/pop-tail/admin/DictionariesView.vue'),
        meta: { icon: 'lucide:book-a', title: '字典管理' },
      },
      {
        name: 'dictionaryDetail',
        path: '/system/dictionaries/:id',
        component: () => import('#/views/pop-tail/admin/DictionaryDetailView.vue'),
        meta: {
          activePath: '/system/dictionaries',
          hideInMenu: true,
          icon: 'lucide:list-tree',
          title: '字典详情',
        },
      },
      {
        name: 'params',
        path: '/system/params',
        component: () => import('#/views/pop-tail/admin/ParamsView.vue'),
        meta: { icon: 'lucide:sliders-horizontal', title: '系统参数' },
      },
      {
        name: 'operationLogs',
        path: '/system/operation-logs',
        component: () => import('#/views/pop-tail/admin/OperationLogsView.vue'),
        meta: { icon: 'lucide:clipboard-list', title: '操作日志' },
      },
      {
        name: 'loginLogs',
        path: '/system/login-logs',
        component: () => import('#/views/pop-tail/admin/LoginLogsView.vue'),
        meta: { icon: 'lucide:logs', title: '登录日志' },
      },
      {
        name: 'systemState',
        path: '/system/state',
        component: () => import('#/views/pop-tail/admin/SystemStateView.vue'),
        meta: { icon: 'lucide:activity-square', title: '系统状态' },
      },
    ],
  },
];

export default routes;
