# Vue Rust Admin 二次开发 Skill

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
- `backend/src/handlers.rs`
- `backend/src/state.rs`

当前主要能力：

- 认证与会话：`/base/*`、`/user/*`
- 权限与菜单：`/authority/*`、`/menu/*`、`/casbin/*`
- 系统管理：用户/角色/菜单/API/字典/参数/日志/系统状态
- 系统工具：AI workflow、API token、LLM 配置、技能、系统配置、邮件、公告
- 前台管理：前台配置、导航、文章分类、文章、会员、控制台菜单

### 前端（Vben + web-antd）

主要入口：

- `frontend/apps/web-antd/src/main.ts`
- `frontend/apps/web-antd/src/router/routes/modules/*.ts`
- `frontend/apps/web-antd/src/store/gin-ai-admin/auth.ts`
- `frontend/apps/web-antd/src/api/gin-ai-admin/*.ts`

当前主路由分组：

- 控制台：`gaa-dashboard.ts`
- 系统管理：`gaa-system.ts`
- 系统工具：`gaa-tools.ts`
- 前台管理：`gaa-frontend.ts`
- 示例中心：`gaa-examples.ts`
- 兼容重定向：`gaa-legacy-redirects.ts`

## UI 模板二开（Vben）

### 主题与视觉规范

- 主题与品牌色优先通过 `frontend/apps/web-antd/src/preferences.ts` 调整
- 颜色体系优先走 CSS 变量（与 Vben 主题机制一致）
- 不建议在业务页内写死大量颜色，优先复用主题 token

### 布局与导航规范

- 一级导航结构以 `src/router/routes/modules/gaa-*.ts` 为准
- 菜单显示行为优先使用 route `meta`（如 `order`、`icon`、`keepAlive`、`hideInMenu`）
- 需要“可访问但不进菜单”的页面使用 `hideInMenu + activePath`
- 旧地址兼容统一放到 `gaa-legacy-redirects.ts`，不要分散在页面组件里

### 页面模板与组件适配

- 新增页面优先参考 `src/views/gin-ai-admin/admin/*` 与 `src/views/gin-ai-admin/tools/*` 的现有实现
- 表单统一走 `src/adapter/form.ts`（Vben Form 适配层），避免页面内直接散落 UI 库差异
- 表格统一走 `src/adapter/vxe-table.ts`，保持筛选、分页、操作列交互一致
- 组件路径候选需同步 `src/views/gin-ai-admin/admin/menu-template/modules/form.vue`

### UI 二开检查单

1. 路由是否已注册到正确模块，并设置了完整 `meta`
2. 页面是否复用了现有布局、表单和表格适配层
3. 菜单候选与后端 `seeded_menu_tree` 是否一致
4. 是否引入了无入口页面或失效重定向
5. 是否通过 typecheck 验证

## 二开标准流程

1. 新增后端能力

- 在 `handlers.rs` 增加 handler
- 在 `lib.rs` 注册路由
- 在 `state.rs` 增加 state 操作与种子数据（需要时）
- 在 `models.rs` 增加 DTO

2. 新增前端 API

- 在 `src/api/gin-ai-admin/admin.ts` 或拆分模块增加 API 方法
- 在 `src/types/gin-ai-admin.ts` 增加类型

3. 新增页面与路由

- 页面放到 `src/views/gin-ai-admin/<domain>/`
- 路由加到对应 `src/router/routes/modules/gaa-*.ts`
- 如需兼容旧路径，再补 `gaa-legacy-redirects.ts`

4. 权限与菜单联动

- 后端菜单种子：`backend/src/state.rs` 的 `seeded_menu_tree`
- 前端默认映射：`src/store/gin-ai-admin/auth.ts` 里的 legacy 归一化映射
- 菜单编辑组件候选：`src/views/gin-ai-admin/admin/menu-template/modules/form.vue`

## 无用代码清理准则

满足以下全部条件可清理：

- 无路由入口（不在 `gaa-*.ts` / core routes）
- 无业务引用（`rg` 全局无 import / 调用）
- 非运行时必须资源（如全局 fallback、基础布局）

清理后必须同步：

- 删除失效 legacy redirect
- 删除失效 route/path normalize 映射
- 删除菜单组件候选中的失效页面

## 本次已清理范围（2026-04-23）

- 清理 `auth.ts` 中已下线功能的 legacy route/path 映射
- 清理 `gaa-legacy-redirects.ts` 中指向已下线页面的重定向
- 清理菜单模板中已下线组件候选
- 删除不可达页面（已下线工具页、旧 demo 页、旧 profile/about 页等）

## 验证命令

前端：

```bash
pnpm -C frontend/apps/web-antd typecheck
```

后端：

```bash
cargo -C backend fmt --check
cargo -C backend test
```

## 执行原则

- 小步提交，优先删除而非叠加兼容层
- 每次删除都要给出“可达性证据”
- 路由、映射、页面三处保持一致
