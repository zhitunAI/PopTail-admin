import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:globe',
      order: -6,
      title: '前台管理',
    },
    name: 'GaaFrontend',
    path: '/frontend',
    redirect: '/frontend/nav',
    children: [
      {
        name: 'frontendSettings',
        path: '/frontend/settings',
        component: () => import('#/views/pop-tail/frontend/FrontendSettingsView.vue'),
        meta: { icon: 'lucide:settings', title: '前台设置' },
      },
      {
        name: 'frontendNav',
        path: '/frontend/nav',
        component: () => import('#/views/pop-tail/frontend/FrontendNavView.vue'),
        meta: { icon: 'lucide:navigation', title: '前端导航管理' },
      },
      {
        name: 'articleCategories',
        path: '/frontend/article-categories',
        component: () => import('#/views/pop-tail/frontend/ArticleCategoriesView.vue'),
        meta: { icon: 'lucide:folders', title: '文章分类管理' },
      },
      {
        name: 'articles',
        path: '/frontend/articles',
        component: () => import('#/views/pop-tail/frontend/ArticlesView.vue'),
        meta: { icon: 'lucide:file-text', title: '文章管理' },
      },
      {
        name: 'members',
        path: '/frontend/members',
        component: () => import('#/views/pop-tail/frontend/MembersView.vue'),
        meta: { icon: 'lucide:users-round', title: '会员管理' },
      },
      {
        name: 'frontendConsoleMenus',
        path: '/frontend/console-menus',
        component: () => import('#/views/pop-tail/frontend/ConsoleMenusView.vue'),
        meta: { icon: 'lucide:layout-panel-top', title: '前端控制台菜单' },
      },
    ],
  },
];

export default routes;
