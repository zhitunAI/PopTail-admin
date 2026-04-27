import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:layout-dashboard',
      order: -10,
      title: '控制台',
    },
    name: 'PopTailDashboard',
    path: '/dashboard',
    children: [
      {
        name: 'dashboard',
        path: '/dashboard',
        component: () => import('#/views/pop-tail/DashboardView.vue'),
        meta: {
          affixTab: true,
          icon: 'lucide:layout-dashboard',
          title: '仪表盘',
        },
      },
      {
        name: 'profile',
        path: '/profile',
        component: () => import('#/views/pop-tail/ProfileView.vue'),
        meta: {
          hideInMenu: true,
          icon: 'lucide:user-circle-2',
          title: '个人资料',
        },
      },
      {
        name: 'about',
        path: '/about',
        component: () => import('#/views/pop-tail/AboutView.vue'),
        meta: {
          icon: 'lucide:info',
          title: '关于系统',
        },
      },
    ],
  },
];

export default routes;
