import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:layout-dashboard',
      order: -10,
      title: '控制台',
    },
    name: 'GaaDashboard',
    path: '/dashboard',
    children: [
      {
        name: 'dashboard',
        path: '/dashboard',
        component: () => import('#/views/gin-ai-admin/DashboardView.vue'),
        meta: {
          affixTab: true,
          icon: 'lucide:layout-dashboard',
          title: '仪表盘',
        },
      },
      {
        name: 'profile',
        path: '/profile',
        component: () => import('#/views/gin-ai-admin/ProfileView.vue'),
        meta: {
          hideInMenu: true,
          icon: 'lucide:user-circle-2',
          title: '个人资料',
        },
      },
      {
        name: 'about',
        path: '/about',
        component: () => import('#/views/gin-ai-admin/AboutView.vue'),
        meta: {
          icon: 'lucide:info',
          title: '关于系统',
        },
      },
    ],
  },
];

export default routes;
