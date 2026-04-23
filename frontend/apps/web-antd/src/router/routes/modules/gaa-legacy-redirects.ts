import type { RouteRecordRaw } from 'vue-router';

const hidden = {
  hideInBreadcrumb: true,
  hideInMenu: true,
  hideInTab: true,
  title: 'Legacy Redirect',
};

const publicHidden = {
  ...hidden,
  ignoreAccess: true,
};

const redirects: RouteRecordRaw[] = [
  { path: '/admin', redirect: '/system/overview', name: 'LegacyAdminRedirect', meta: hidden },
  { path: '/person', redirect: '/profile', name: 'LegacyPersonRedirect', meta: hidden },
  { path: '/example', redirect: '/examples', name: 'LegacyExampleRedirect', meta: hidden },
  { path: '/state', redirect: '/system/state', name: 'LegacyStateRedirect', meta: hidden },
  { path: '/plugin', redirect: '/system/tools', name: 'LegacyPluginRedirect', meta: hidden },
  { path: '/admin/authority', redirect: '/system/authorities', name: 'LegacyAuthorityRedirect', meta: hidden },
  { path: '/admin/menu', redirect: '/system/menus', name: 'LegacyMenuRedirect', meta: hidden },
  { path: '/admin/api', redirect: '/system/apis', name: 'LegacyApiRedirect', meta: hidden },
  { path: '/admin/user', redirect: '/system/users', name: 'LegacyUserRedirect', meta: hidden },
  { path: '/admin/dictionary', redirect: '/system/dictionaries', name: 'LegacyDictRedirect', meta: hidden },
  { path: '/admin/operation', redirect: '/system/operation-logs', name: 'LegacyOperationRedirect', meta: hidden },
  { path: '/admin/sysParams', redirect: '/system/params', name: 'LegacyParamsRedirect', meta: hidden },
  { path: '/admin/system', redirect: '/system/tools/config', name: 'LegacySystemRedirect', meta: hidden },
  { path: '/admin/apiToken', redirect: '/system/tools/api-tokens', name: 'LegacyApiTokenRedirect', meta: hidden },
  { path: '/admin/loginLog', redirect: '/system/login-logs', name: 'LegacyLoginLogRedirect', meta: hidden },
  { path: '/example/upload', redirect: '/examples/upload', name: 'LegacyExampleUploadRedirect', meta: hidden },
  { path: '/example/breakpoint', redirect: '/examples/breakpoint', name: 'LegacyExampleBreakpointRedirect', meta: hidden },
  { path: '/example/customer', redirect: '/examples/customer', name: 'LegacyExampleCustomerRedirect', meta: hidden },
  { path: '/systemTools', redirect: '/system/tools', name: 'LegacySystemToolsRedirect', meta: hidden },
  { path: '/systemTools/aiWorkflow', redirect: '/system/tools/ai-workflow', name: 'LegacyAiWorkflowRedirect', meta: hidden },
  { path: '/systemTools/skills', redirect: '/system/tools/skills', name: 'LegacySkillsRedirect', meta: hidden },
  { path: '/plugin-email', redirect: '/system/tools/plugin-email', name: 'LegacyPluginEmailRedirect', meta: hidden },
  { path: '/anInfo', redirect: '/system/tools/announcement', name: 'LegacyAnnouncementRedirect', meta: hidden },
  { path: '/scanUpload', redirect: '/scan-upload', name: 'LegacyScanUploadRedirect', meta: publicHidden },
];

export default redirects;
