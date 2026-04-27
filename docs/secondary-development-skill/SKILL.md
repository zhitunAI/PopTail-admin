# PopTail-admin 二次开发 Skill

本项目基于 `vue-vben-admin` 框架构建：

- [https://github.com/vbenjs/vue-vben-admin](https://github.com/vbenjs/vue-vben-admin)

## 目标

在不破坏当前可用功能的前提下，快速完成业务二开：

- 明确前后端边界
- 在正确位置新增页面/API/数据结构
- 及时清理不可达和历史残留代码
- 保持可验证、可回滚

## 当前系统功能边界（以现状代码为准）

### 后端（Rust Axum）

入口与核心：

- `backend/src/main.rs`
- `backend/src/lib.rs`
- `backend/src/auth.rs`
- `backend/src/models.rs`
- `backend/src/storage.rs`
- `backend/src/core/http.rs`
- `backend/src/state/mod.rs`
- `backend/src/state/admin.rs`
- `backend/src/state/misc.rs`
- `backend/src/state/tooling.rs`
- `backend/src/state/examples.rs`
- `backend/src/state/platform.rs`

当前主要能力：

- 认证与会话：`/base/*`、`/user/*`
- 权限与菜单：`/authority/*`、`/menu/*`、`/casbin/*`
- 系统管理：用户/角色/菜单/API/字典/参数/日志/系统状态
- 系统工具：AI workflow、API token、LLM 配置、技能、系统配置、邮件、公告
- 前台管理：前台配置、导航、文章分类、文章、会员、控制台菜单

### 后端目录说明

- `backend/src/features/auth/`
  认证、登录态、用户信息、profile 保存、路由注册。
- `backend/src/features/authority/`
  用户、角色、菜单、按钮权限、菜单树、角色菜单关系。
- `backend/src/features/system/`
  API、字典、参数、日志、系统信息等系统管理能力。
- `backend/src/features/tooling/`
  AI workflow、skills、LLM、API token、邮件、公告等工具能力。
- `backend/src/features/frontend/`
  前台导航、文章、分类、会员、控制台菜单、前台设置。
- `backend/src/features/examples/`
  示例与演示接口。
- `backend/src/features/customer/`
  客户示例相关接口。
- `backend/src/state/`
  业务数据读写与种子，是真正的“当前系统能力来源”。
  新增模块时，优先在对应 `state/*.rs` 增加列表、详情、保存、删除操作。
- `backend/src/models.rs`
  前后端契约类型，包含请求 DTO、响应 DTO、列表结构、领域模型。
- `backend/src/storage.rs`
  持久化层与 sqlite/postgres 适配；涉及真正存储时在这里扩展。
- `backend/src/router_tests.rs`
  路由契约测试；新增接口后优先补这里的端到端验证。

### 后端新增模块放置规则

新增后端业务时，按下面顺序放置：

1. `backend/src/models.rs`
   增加请求/响应/列表 DTO
2. `backend/src/state/<domain>.rs` 或对应已有 state 文件
   增加列表、详情、保存、删除操作
3. `backend/src/features/<domain>/handlers.rs`
   增加 handler
4. `backend/src/features/<domain>/router.rs`
   注册路由
5. `backend/src/features/<domain>/mod.rs`
   导出模块
6. `backend/src/router_tests.rs`
   增加最小契约测试

### 前端（Vben + web-antd）

主要入口：

- `frontend/apps/web-antd/src/main.ts`
- `frontend/apps/web-antd/src/router/routes/modules/*.ts`
- `frontend/apps/web-antd/src/store/pop-tail/auth.ts`
- `frontend/apps/web-antd/src/api/pop-tail/*.ts`

当前主路由分组：

- 控制台：`pop-tail-dashboard.ts`
- 系统管理：`pop-tail-system.ts`
- 系统工具：`pop-tail-tools.ts`
- 前台管理：`pop-tail-frontend.ts`
- 示例中心：`pop-tail-examples.ts`
- 兼容重定向：`pop-tail-legacy-redirects.ts`

## UI 模板二开（Vben）

### 主题与视觉规范

- 主题与品牌色优先通过 `frontend/apps/web-antd/src/preferences.ts` 调整
- 颜色体系优先走 CSS 变量（与 Vben 主题机制一致）
- 不建议在业务页内写死大量颜色，优先复用主题 token

### 布局与导航规范

- 一级导航结构以 `src/router/routes/modules/pop-tail-*.ts` 为准
- 菜单显示行为优先使用 route `meta`（如 `order`、`icon`、`keepAlive`、`hideInMenu`）
- 需要“可访问但不进菜单”的页面使用 `hideInMenu + activePath`
- 旧地址兼容统一放到 `pop-tail-legacy-redirects.ts`，不要分散在页面组件里

### 页面模板与组件适配

- 新增页面优先参考 `src/views/pop-tail/admin/*` 与 `src/views/pop-tail/tools/*` 的现有实现
- 表单统一走 `src/adapter/form.ts`（Vben Form 适配层），避免页面内直接散落 UI 库差异
- 表格统一走 `src/adapter/vxe-table.ts`，保持筛选、分页、操作列交互一致
- 组件路径候选需同步 `src/views/pop-tail/admin/menu-template/modules/form.vue`

### 新建带列表模块的强制复用规则

只要是“新增模块包含列表”，必须优先复用用户管理这一套 UI 方案，不要自己重新拼表格：

- 参考页：
  - `src/views/pop-tail/admin/UsersView.vue`
- 必须复用：
  - `useVbenVxeGrid`
  - `useVbenForm`
  - `useVbenDrawer`
  - `Page auto-content-height`
  - `PageRefreshCellSkeleton`
  - `usePageRefreshLoading`
  - `useMenuButtonAccess`
- 列表统一具备：
  - 搜索表单
  - toolbar 刷新
  - 分页
  - 操作列
  - 列表单元格骨架屏
  - 按钮权限控制
- 骨架屏规则：
  - 只替换列表动态内容单元格
  - 不覆盖搜索区、工具栏、表头、静态说明
- 刷新链路规则：
  - `tab` 刷新
  - 页面内刷新按钮
  - 搜索/筛选
  - 翻页
  - 代码里的 `onRefresh`
  都要走同一套列表骨架刷新链
- 禁止事项：
  - 不要自己直接拼 `Table/TableColumn`
  - 不要新增整块遮罩式骨架
  - 不要跳过按钮权限
  - 不要在列表页里写第二套独立刷新逻辑

### UI 二开检查单

1. 路由是否已注册到正确模块，并设置了完整 `meta`
2. 页面是否复用了现有布局、表单和表格适配层
3. 菜单候选与后端 `seeded_menu_tree` 是否一致
4. 是否引入了无入口页面或失效重定向
5. 是否通过 typecheck 验证

## 二开标准流程

1. 新增后端能力

- 在 `backend/src/features/<domain>/handlers.rs` 增加 handler
- 在 `backend/src/features/<domain>/router.rs` 注册路由
- 在 `backend/src/state/*.rs` 增加 state 操作与种子数据（需要时）
- 在 `backend/src/models.rs` 增加 DTO

2. 新增前端 API

- 在 `src/api/pop-tail/admin.ts` 或拆分模块增加 API 方法
- 在 `src/types/pop-tail.ts` 增加类型

3. 新增页面与路由

- 页面放到 `src/views/pop-tail/<domain>/`
- 路由加到对应 `src/router/routes/modules/pop-tail-*.ts`
- 如需兼容旧路径，再补 `pop-tail-legacy-redirects.ts`

4. 权限与菜单联动

- 后端菜单种子：`backend/src/state/seed.rs` 的 `seeded_menu_tree`
- 前端默认映射：`src/store/pop-tail/auth.ts` 里的 legacy 归一化映射
- 菜单编辑组件候选：`src/views/pop-tail/admin/menu-template/modules/form.vue`

## 无用代码清理准则

满足以下全部条件可清理：

- 无路由入口（不在 `pop-tail-*.ts` / core routes）
- 无业务引用（`rg` 全局无 import / 调用）
- 非运行时必须资源（如全局 fallback、基础布局）

清理后必须同步：

- 删除失效 legacy redirect
- 删除失效 route/path normalize 映射
- 删除菜单组件候选中的失效页面

## 本次已清理范围（2026-04-23）

- 清理 `auth.ts` 中已下线功能的 legacy route/path 映射
- 清理 `pop-tail-legacy-redirects.ts` 中指向已下线页面的重定向
- 清理菜单模板中已下线组件候选
- 删除不可达页面（已下线工具页、旧 demo 页、旧 profile/about 页等）

## 验证命令

前端：

```bash
pnpm -C frontend --filter @vben/web-antd run typecheck
```

后端：

```bash
cd backend && cargo fmt --check
cd backend && cargo test
```

## 执行原则

- 小步提交，优先删除而非叠加兼容层
- 每次删除都要给出“可达性证据”
- 路由、映射、页面三处保持一致
