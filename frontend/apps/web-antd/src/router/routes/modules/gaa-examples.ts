import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:flask-conical',
      order: -8,
      title: '示例中心',
    },
    name: 'GaaExamples',
    path: '/examples',
    children: [
      {
        name: 'examplesRoot',
        path: '/examples',
        component: () => import('#/views/gin-ai-admin/examples/ExamplesView.vue'),
        meta: { icon: 'lucide:flask-conical', title: '示例中心' },
      },
      {
        name: 'customerExample',
        path: '/examples/customer',
        component: () => import('#/views/gin-ai-admin/examples/CustomerExampleView.vue'),
        meta: { icon: 'lucide:users-round', title: '客户示例' },
      },
      {
        name: 'uploadExample',
        path: '/examples/upload',
        component: () => import('#/views/gin-ai-admin/examples/UploadExampleView.vue'),
        meta: { icon: 'lucide:upload-cloud', title: '上传示例' },
      },
      {
        name: 'breakpointExample',
        path: '/examples/breakpoint',
        component: () => import('#/views/gin-ai-admin/examples/BreakpointExampleView.vue'),
        meta: { icon: 'lucide:separator-horizontal', title: '断点续传' },
      },
    ],
  },
];

export default routes;
