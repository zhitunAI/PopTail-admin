# GAA Desktop Shell Bootstrap

本目录用于承接 **Rust + Tauri 2** 桌面端重构的最小可运行骨架。

当前目标不是替换现有 `rust-server` / `vben-web` 主线，而是把桌面端后续实施所需的结构、配置与联调入口提前落地，避免后续阶段重新拆分目录。

## 目录结构

- `src-tauri/`
  - `Cargo.toml`：Tauri 2 桌面壳 crate 定义
  - `tauri.conf.json`：桌面壳运行/构建配置
  - `src/main.rs`：最小桌面命令入口

## 当前约定

1. 前端复用 `../frontend/apps/web-antd`
2. 后端复用 `../backend`
3. SQL 与 Redis 通过 `../docker-compose.yml` 提供
4. 桌面端阶段不得回退到 legacy `server/` + `web/`

## 推荐开发顺序

### 1) 启动后端依赖

```bash
cd ..
./dev-backend.sh
```

### 2) 启动前端

```bash
cd ../frontend/apps/web-antd
pnpm dev --host 127.0.0.1 --port 5666
```

### 3) 检查桌面壳

```bash
cd ../desktop-shell/src-tauri
cargo check
```

### 4) 运行桌面壳

当前仓库已经改成指向 `PopTail-admin` 的前后端：

1. 开发时加载 `http://127.0.0.1:5666`
2. 构建时加载 `../frontend/apps/web-antd/dist`
3. 桌面壳当前仅保留运行态命令桥，不改变现有前端页面实现

## 当前命令桥

`desktop_runtime_snapshot`

返回桌面壳感知到的运行态信息：

- 后端地址
- 数据库地址
- Redis 地址
- 是否启用多点登录

这为后续桌面端“环境自检页 / 运行态页”打底。
