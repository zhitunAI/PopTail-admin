# PopTail-admin

PopTail-admin 是一套后台管理与前台内容管理一体化项目，包含 Rust 后端、基于 Vue 的管理前端、桌面端壳以及公共前台站点。

本项目基于 `vue-vben-admin` 框架构建：

- 框架仓库：
  [https://github.com/vbenjs/vue-vben-admin](https://github.com/vbenjs/vue-vben-admin)

项目仓库：

- [https://github.com/zhitunAI/PopTail-admin.git](https://github.com/zhitunAI/PopTail-admin.git)

## 项目定位

PopTail-admin 当前主要服务于以下场景：

- 系统管理后台
- 权限、菜单、角色、用户管理
- 系统工具管理
- 前台内容与配置管理
- 桌面端管理壳接入

它不是纯模板仓库，而是在 `vue-vben-admin` 的基础上收敛出的当前业务项目。

## 技术说明

### 后端技术栈

- `Rust`
- `Axum`
- `Tokio`
- `Serde`
- `SQLx`
- `SQLite / PostgreSQL`
- `Redis`
- `Casbin`
- `Docker Compose`

后端职责：

- 登录鉴权与会话管理
- 权限控制与菜单树管理
- 系统管理数据接口
- 工具类模块接口
- 前台管理接口
- 持久化与种子数据初始化

### 前端技术栈

- `Vue 3`
- `TypeScript`
- `vue-vben-admin`
- `Ant Design Vue`
- `Vxe Grid`
- `Pinia`
- `Vue Router`
- `pnpm workspace`

前端职责：

- 后台管理 UI
- 菜单与权限联动
- 列表查询、分页、骨架屏刷新
- 抽屉表单、详情编辑、配置管理

### 桌面端技术栈

- `Tauri`
- `Rust`
- `WebView`

桌面端职责：

- 将后台前端封装为桌面应用
- 复用前端页面和后端接口
- 提供桌面环境下的运行与调试入口

### 公共前台技术栈

- `Next.js`
- `React`
- `TypeScript`

公共前台职责：

- 面向外部访问的展示与内容承载
- 与后台管理的配置/内容能力配合使用

## 目录说明

### 根目录

- `backend/`
  Rust Axum 后端，包含接口、状态层、模型、测试、Docker 支持。
- `frontend/`
  前端 monorepo，目前主业务应用是 `apps/web-antd`。
- `desktop-shell/`
  Tauri 桌面端壳。
- `public-frontend/`
  公共前台站点。
- `docs/`
  项目补充说明文档与二开 skill。
- `docker-compose.yml`
  根级联调编排文件。
- `dev-*.sh`
  本地启动、停止、重启、状态检查脚本。

### 后端目录说明

- `backend/src/main.rs`
  后端启动入口。
- `backend/src/lib.rs`
  应用装配入口。
- `backend/src/models.rs`
  前后端契约类型、请求/响应 DTO、领域模型。
- `backend/src/storage.rs`
  持久化适配层。
- `backend/src/auth.rs`
  登录态与鉴权相关能力。
- `backend/src/core/http.rs`
  HTTP 基础设施。
- `backend/src/state/`
  业务状态与读写逻辑，是当前系统能力的核心来源。
- `backend/src/features/auth/`
  认证、用户信息、profile 等。
- `backend/src/features/authority/`
  用户、角色、菜单、按钮权限、菜单树。
- `backend/src/features/system/`
  API、字典、参数、日志、系统状态等。
- `backend/src/features/tooling/`
  AI workflow、API token、技能、LLM、邮件、公告等工具模块。
- `backend/src/features/frontend/`
  前台配置、导航、文章、分类、会员等。
- `backend/tests/`
  后端契约与回归测试。

### 前端目录说明

- `frontend/apps/web-antd/`
  当前使用的后台前端应用。
- `frontend/apps/web-antd/src/api/pop-tail/`
  项目业务 API 调用层。
- `frontend/apps/web-antd/src/store/pop-tail/`
  项目业务 store。
- `frontend/apps/web-antd/src/views/pop-tail/`
  项目业务页面。
- `frontend/apps/web-antd/src/router/routes/modules/`
  后台主路由模块。
- `frontend/apps/web-antd/src/components/`
  项目级页面骨架、通用组件等。
- `frontend/packages/`
  `vue-vben-admin` 工作区依赖包。
- `frontend/internal/`
  内部构建、lint、配置支撑。
- `frontend/docs/`
  上游框架文档与工作区说明。

### 桌面端目录说明

- `desktop-shell/src-tauri/`
  桌面端壳、配置、Rust 启动逻辑、图标资源。

### 公共前台目录说明

- `public-frontend/app/`
  前台页面入口。
- `public-frontend/components/`
  前台组件。
- `public-frontend/lib/`
  前台工具与公共逻辑。

## 功能说明

### 系统管理

- 用户管理
- 角色管理
- 菜单管理
- API 管理
- 字典管理
- 参数管理
- 操作日志
- 登录日志
- 系统状态
- 系统工具入口

### 权限与菜单

- 登录鉴权
- 菜单树加载
- 角色菜单分配
- 按钮权限控制
- 菜单参数与可控按钮配置

### 工具模块

- AI workflow
- API Token 管理
- LLM 配置
- Skills 管理
- 系统配置
- 邮件管理
- 公告管理

### 前台管理

- 前台导航
- 文章分类
- 文章管理
- 会员管理
- 前台设置
- 控制台菜单

### 终端与桌面运行

- 浏览器后台前端开发模式
- Docker 后端本地联调
- Tauri 桌面端运行与发布链

## 开发脚本说明

### 启动后端

```bash
cd /path/to/PopTail-admin
./dev-backend.sh
```

后端默认地址：

```text
http://127.0.0.1:8888
```

### 启动前后端联调

```bash
cd /path/to/PopTail-admin
./dev-up.sh
```

这会启动：

- 后端 `docker compose`
- 前端开发服务

前端日志默认写入：

```text
./.run/frontend.log
```

### 启动前端

```bash
cd /path/to/PopTail-admin/frontend
pnpm install
pnpm -C apps/web-antd dev
```

前端默认地址：

```text
http://127.0.0.1:5666
```

默认前端 API 配置：

```text
VITE_POP_TAIL_API_BASE=http://127.0.0.1:8888
```

### 启动桌面端

```bash
cd /path/to/PopTail-admin
./dev-desktop.sh
```

### 服务状态

```bash
cd /path/to/PopTail-admin
./dev-status.sh
```

### 停止服务

```bash
cd /path/to/PopTail-admin
./dev-stop.sh
```

### 重启服务

```bash
cd /path/to/PopTail-admin
./dev-restart.sh
```

## 默认登录

```text
username: admin
password: 123456
```

## 验证说明

前端类型检查：

```bash
pnpm -C /path/to/PopTail-admin/frontend --filter @vben/web-antd run typecheck
```

后端检查：

```bash
cd /path/to/PopTail-admin/backend
cargo check
```

## 二次开发说明

如果你要继续在这个项目上二开，优先看：

- [docs/secondary-development-skill/SKILL.md](/Users/mac/Documents/gin-ai-admin/PopTail-admin/docs/secondary-development-skill/SKILL.md)

这份文档包含：

- 当前技术边界
- 后端模块放置规则
- 前端列表页复用规则
- 骨架屏、刷新、按钮权限的统一要求
