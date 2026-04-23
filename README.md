# Vue Rust Admin

Vue Rust Admin is the extracted Rust backend plus the current Vben Vue frontend from the original workspace.

## Structure

- `backend/` - Rust Axum admin backend, including Dockerfile and tests.
- `frontend/` - Vben workspace containing the current `apps/web-antd` frontend and the workspace packages it depends on.
- `docker-compose.yml` - root compose file for the backend, Postgres, and Redis.

## Run Backend

```bash
cd /Users/mac/Documents/gin-ai-admin/vue-rust-admin
./dev-backend.sh
```

Backend API:

```text
http://127.0.0.1:8888
```

Smoke check:

```bash
curl -i -X POST http://127.0.0.1:8888/base/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"123456"}'
```

## Run Frontend

```bash
cd /Users/mac/Documents/gin-ai-admin/vue-rust-admin/frontend
pnpm install
../dev-frontend.sh
```

Frontend dev server defaults to:

```text
http://127.0.0.1:5666
```

The frontend is configured to call:

```text
VITE_GAA_API_BASE=http://127.0.0.1:8888
```

## Default Login

```text
username: admin
password: 123456
```
