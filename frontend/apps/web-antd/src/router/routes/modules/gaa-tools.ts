import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    meta: {
      icon: 'lucide:wrench',
      order: -7,
      title: '系统工具',
    },
    name: 'GaaTools',
    path: '/system/tools',
    children: [
      {
        name: 'systemTools',
        path: '/system/tools',
        component: () => import('#/views/gin-ai-admin/admin/SystemToolsView.vue'),
        meta: { icon: 'lucide:wrench', title: '系统工具' },
      },
      {
        name: 'aiWorkflow',
        path: '/system/tools/ai-workflow',
        component: () => import('#/views/gin-ai-admin/tools/AiWorkflowView.vue'),
        meta: { icon: 'lucide:sparkles', keepAlive: true, title: 'AI 工作流' },
      },
      {
        name: 'apiTokens',
        path: '/system/tools/api-tokens',
        component: () => import('#/views/gin-ai-admin/tools/ApiTokensView.vue'),
        meta: { icon: 'lucide:key', title: 'API Token' },
      },
      {
        name: 'llmConfig',
        path: '/system/tools/llm-config',
        component: () => import('#/views/gin-ai-admin/tools/LlmConfigView.vue'),
        meta: { icon: 'lucide:brain', title: '大模型配置' },
      },
      {
        name: 'skills',
        path: '/system/tools/skills',
        component: () => import('#/views/gin-ai-admin/tools/SkillsView.vue'),
        meta: { icon: 'lucide:brain-circuit', title: '技能管理' },
      },
      {
        name: 'systemConfig',
        path: '/system/tools/config',
        component: () => import('#/views/gin-ai-admin/tools/SystemConfigView.vue'),
        meta: { icon: 'lucide:settings-2', title: '系统配置' },
      },
      {
        name: 'pluginEmail',
        path: '/system/tools/plugin-email',
        component: () => import('#/views/gin-ai-admin/tools/EmailPluginView.vue'),
        meta: { icon: 'lucide:mail', title: '邮件管理' },
      },
      {
        name: 'announcementInfo',
        path: '/system/tools/announcement',
        component: () => import('#/views/gin-ai-admin/tools/AnnouncementView.vue'),
        meta: { icon: 'lucide:megaphone', title: '公告管理' },
      },
    ],
  },
];

export default routes;
