import { createRouter, createWebHistory } from "vue-router";
import { useAuthStore } from "./stores/auth";

const legacyRouteRedirects = [
  { path: "/dashboard", redirect: "/dashboard" },
  { path: "/about", redirect: "/about" },
  { path: "/admin", redirect: "/system/overview" },
  { path: "/person", redirect: "/profile" },
  { path: "/example", redirect: "/examples" },
  { path: "/state", redirect: "/system/state" },
  { path: "/plugin", redirect: "/system/tools" },
  { path: "/admin/authority", redirect: "/system/authorities" },
  { path: "/admin/menu", redirect: "/system/menus" },
  { path: "/admin/api", redirect: "/system/apis" },
  { path: "/admin/user", redirect: "/system/users" },
  { path: "/admin/dictionary", redirect: "/system/dictionaries" },
  { path: "/admin/operation", redirect: "/system/operation-logs" },
  { path: "/admin/sysParams", redirect: "/system/params" },
  { path: "/admin/system", redirect: "/system/tools/config" },
  { path: "/admin/apiToken", redirect: "/system/tools/api-tokens" },
  { path: "/admin/loginLog", redirect: "/system/login-logs" },
  { path: "/admin/sysVersion", redirect: "/system/tools/version" },
  { path: "/admin/sysError", redirect: "/system/tools/errors" },
  { path: "/example/upload", redirect: "/examples/upload" },
  { path: "/example/breakpoint", redirect: "/examples/breakpoint" },
  { path: "/example/customer", redirect: "/examples/customer" },
  { path: "/systemTools", redirect: "/system/tools" },
  { path: "/systemTools/autoPkg", redirect: "/system/tools/auto-package" },
  { path: "/systemTools/autoCode", redirect: "/system/tools/autocode" },
  { path: "/systemTools/formCreate", redirect: "/system/tools/form-create" },
  { path: "/systemTools/aiWorkflow", redirect: "/system/tools/ai-workflow" },
  { path: "/systemTools/exportTemplate", redirect: "/system/tools/export-templates" },
  { path: "/systemTools/mcpTest", redirect: "/system/tools/autocode/mcp-test" },
  { path: "/systemTools/mcpTool", redirect: "/system/tools/autocode/mcp" },
  { path: "/systemTools/skills", redirect: "/system/tools/skills" },
  { path: "/systemTools/picture", redirect: "/system/tools/autocode/picture" },
  { path: "/systemTools/autoCodeAdmin", redirect: "/system/tools/autocode-admin" },
  { path: "/plugin/installPlugin", redirect: "/system/tools/install-plugin" },
  { path: "/plugin/pubPlug", redirect: "/system/tools/publish-plugin" },
  { path: "/plugin-email", redirect: "/system/tools/plugin-email" },
  { path: "/anInfo", redirect: "/system/tools/announcement" },
].map((item) => ({
  ...item,
  meta: { public: true, compatibility: true },
}));

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/login",
      name: "login",
      component: () => import("./views/LoginView.vue"),
      meta: { public: true },
    },
    {
      path: "/init",
      name: "init",
      component: () => import("./views/InitView.vue"),
      meta: { public: true },
    },
    {
      path: "/",
      component: () => import("./components/AppShell.vue"),
      children: [
        {
          path: "dashboard",
          name: "dashboard",
          component: () => import("./views/DashboardView.vue"),
        },
        {
          path: "profile",
          name: "profile",
          component: () => import("./views/ProfileView.vue"),
        },
        {
          path: "about",
          name: "about",
          component: () => import("./views/AboutView.vue"),
        },
        {
          path: "examples",
          name: "examplesRoot",
          component: () => import("./views/examples/ExamplesView.vue"),
        },
        {
          path: "examples/customer",
          name: "customerExample",
          component: () => import("./views/examples/CustomerExampleView.vue"),
        },
        {
          path: "examples/upload",
          name: "uploadExample",
          component: () => import("./views/examples/UploadExampleView.vue"),
        },
        {
          path: "examples/breakpoint",
          name: "breakpointExample",
          component: () => import("./views/examples/BreakpointExampleView.vue"),
        },
        {
          path: "system/overview",
          name: "systemOverview",
          component: () => import("./views/admin/SystemOverviewView.vue"),
        },
        {
          path: "system/users",
          name: "users",
          component: () => import("./views/admin/UsersView.vue"),
        },
        {
          path: "system/authorities",
          name: "authorities",
          component: () => import("./views/admin/AuthoritiesView.vue"),
        },
        {
          path: "system/menus",
          name: "menus",
          component: () => import("./views/admin/MenusView.vue"),
        },
        {
          path: "system/apis",
          name: "apis",
          component: () => import("./views/admin/ApisView.vue"),
        },
        {
          path: "system/icons",
          name: "menuIcons",
          component: () => import("./views/admin/IconGalleryView.vue"),
        },
        {
          path: "system/dictionaries",
          name: "dictionaries",
          component: () => import("./views/admin/DictionariesView.vue"),
        },
        {
          path: "system/dictionaries/:id",
          name: "dictionaryDetail",
          component: () => import("./views/admin/DictionaryDetailView.vue"),
        },
        {
          path: "system/params",
          name: "params",
          component: () => import("./views/admin/ParamsView.vue"),
        },
        {
          path: "system/operation-logs",
          name: "operationLogs",
          component: () => import("./views/admin/OperationLogsView.vue"),
        },
        {
          path: "system/login-logs",
          name: "loginLogs",
          component: () => import("./views/admin/LoginLogsView.vue"),
        },
        {
          path: "system/tools",
          name: "systemTools",
          component: () => import("./views/admin/SystemToolsView.vue"),
        },
        {
          path: "system/state",
          name: "systemState",
          component: () => import("./views/admin/SystemStateView.vue"),
        },
        {
          path: "system/tools/ai-workflow",
          name: "aiWorkflow",
          component: () => import("./views/tools/AiWorkflowView.vue"),
        },
        {
          path: "system/tools/autocode",
          name: "autoCode",
          component: () => import("./views/tools/AutoCodeView.vue"),
        },
        {
          path: "system/tools/autocode/mcp",
          name: "autoCodeMcp",
          component: () => import("./views/tools/AutoCodeMcpView.vue"),
        },
        {
          path: "system/tools/autocode/mcp-test",
          name: "autoCodeMcpTest",
          component: () => import("./views/tools/AutoCodeMcpTestView.vue"),
        },
        {
          path: "system/tools/autocode/picture",
          name: "autoCodePicture",
          component: () => import("./views/tools/AutoCodePictureView.vue"),
        },
        {
          path: "system/tools/autocode-admin",
          name: "autoCodeAdmin",
          component: () => import("./views/tools/AutoCodeAdminView.vue"),
        },
        {
          path: "system/tools/api-tokens",
          name: "apiTokens",
          component: () => import("./views/tools/ApiTokensView.vue"),
        },
        {
          path: "system/tools/form-create",
          name: "formCreate",
          component: () => import("./views/tools/FormDesignerView.vue"),
        },
        {
          path: "system/tools/publish-plugin",
          name: "publishPlugin",
          component: () => import("./views/tools/PluginPublisherView.vue"),
        },
        {
          path: "system/tools/skills",
          name: "skills",
          component: () => import("./views/tools/SkillsView.vue"),
        },
        {
          path: "system/tools/version",
          name: "versionInfo",
          component: () => import("./views/tools/VersionView.vue"),
        },
        {
          path: "system/tools/config",
          name: "systemConfig",
          component: () => import("./views/tools/SystemConfigView.vue"),
        },
        {
          path: "system/tools/errors",
          name: "sysErrors",
          component: () => import("./views/tools/ErrorLogsView.vue"),
        },
        {
          path: "system/tools/export-templates",
          name: "exportTemplates",
          component: () => import("./views/tools/ExportTemplatesView.vue"),
        },
        {
          path: "system/tools/install-plugin",
          name: "installPlugins",
          component: () => import("./views/tools/InstallPluginView.vue"),
        },
        {
          path: "system/tools/auto-package",
          name: "autoPackages",
          component: () => import("./views/tools/AutoPackageView.vue"),
        },
        {
          path: "system/tools/runtime-state",
          name: "runtimeState",
          component: () => import("./views/tools/RuntimeStateView.vue"),
        },
        {
          path: "system/tools/plugin-email",
          name: "pluginEmail",
          component: () => import("./views/tools/EmailPluginView.vue"),
        },
        {
          path: "system/tools/announcement",
          name: "announcementInfo",
          component: () => import("./views/tools/AnnouncementView.vue"),
        },
      ],
    },
    {
      path: "/",
      redirect: "/dashboard",
    },
    {
      path: "/scan-upload",
      name: "scanUpload",
      component: () => import("./views/examples/ScanUploadView.vue"),
      meta: { public: true },
    },
    {
      path: "/scanUpload",
      redirect: "/scan-upload",
      meta: { public: true },
    },
    {
      path: "/systemTools/autoCodeEdit/:id",
      redirect: (to) => ({
        path: "/system/tools/autocode",
        query: { ...to.query, legacyId: String(to.params.id ?? "") },
      }),
      meta: { public: true, compatibility: true },
    },
    ...legacyRouteRedirects,
    {
      path: "/error/reload",
      name: "errorReload",
      component: () => import("./views/ErrorReloadView.vue"),
      meta: { public: true },
    },
    {
      path: "/:pathMatch(.*)*",
      name: "error",
      component: () => import("./views/ErrorView.vue"),
      meta: { public: true },
    },
  ],
});

router.beforeEach(async (to) => {
  const auth = useAuthStore();
  if (!auth.bootstrapped) {
    await auth.bootstrap();
  }

  if (to.meta.public) {
    if (auth.isLoggedIn) {
      const defaultName = auth.defaultRouterName;
      if (defaultName && router.hasRoute(defaultName)) {
        return { name: defaultName };
      }
    }
    return true;
  }

  if (!auth.isLoggedIn) {
    return { name: "login", query: { redirect: to.fullPath } };
  }
  return true;
});
