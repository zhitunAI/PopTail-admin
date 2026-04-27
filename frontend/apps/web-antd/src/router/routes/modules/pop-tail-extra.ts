import type { RouteRecordRaw } from 'vue-router';

import { LOGIN_PATH } from '@vben/constants';

const routes: RouteRecordRaw[] = [
  {
    name: 'LegacyLoginRedirect',
    path: '/login',
    redirect: LOGIN_PATH,
    meta: {
      hideInBreadcrumb: true,
      hideInMenu: true,
      hideInTab: true,
      ignoreAccess: true,
      title: '登录跳转',
    },
  },
  {
    name: 'init',
    path: '/init',
    component: () => import('#/views/pop-tail/InitView.vue'),
    meta: {
      hideInMenu: true,
      icon: 'lucide:rocket',
      ignoreAccess: true,
      title: '初始化引导',
    },
  },
  {
    name: 'scanUpload',
    path: '/scan-upload',
    component: () => import('#/views/pop-tail/examples/ScanUploadView.vue'),
    meta: {
      hideInMenu: true,
      hideInTab: true,
      icon: 'lucide:scan-line',
      ignoreAccess: true,
      title: '扫码上传',
    },
  },
  {
    name: 'errorReload',
    path: '/error/reload',
    component: () => import('#/views/pop-tail/ErrorReloadView.vue'),
    meta: {
      hideInMenu: true,
      hideInTab: true,
      icon: 'lucide:refresh-cw',
      title: '页面恢复',
    },
  },
];

export default routes;
