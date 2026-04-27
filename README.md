# PopTail-admin

PopTail-admin is the extracted Rust backend plus the current Vben Vue frontend from the original workspace.

## Framework Base

This project is built on top of the `vue-vben-admin` framework.

- Framework repository:
  [https://github.com/vbenjs/vue-vben-admin](https://github.com/vbenjs/vue-vben-admin)

## Structure

- `backend/` - Rust Axum admin backend, including Dockerfile and tests.
- `frontend/` - Vben workspace containing the current `apps/web-antd` frontend and the workspace packages it depends on.
- `docker-compose.yml` - root compose file for the backend, Postgres, and Redis.

## Run Backend

```bash
cd /path/to/PopTail-admin
./dev-backend.sh
```

`dev-backend.sh` will automatically provide local development defaults for:

- `POP_TAIL_POSTGRES_PASSWORD`
- `POP_TAIL_DATABASE_URL`
- `POP_TAIL_REDIS_URL`
- `POP_TAIL_JWT_SECRET`
- `POP_TAIL_SERVICE_MODERATION_TOKEN`
- `POP_TAIL_BOOTSTRAP_ADMIN_PASSWORD`
- `POP_TAIL_DEFAULT_USER_PASSWORD`

So you can start the backend directly without exporting those variables first.
It also rebuilds the backend image before starting, so local code changes (including CORS fixes) are applied to the running container.

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

## Run Both Services

```bash
cd /path/to/PopTail-admin
./dev-up.sh
```

This starts:

- backend via `docker compose`
- frontend dev server in the background

`dev-up.sh` also applies the same local development defaults as `dev-backend.sh`, so it can boot the full stack without a separate `.env` export step.

Frontend logs are written to:

```text
./.run/frontend.log
```

## Service Status

```bash
cd /path/to/PopTail-admin
./dev-status.sh
```

## Stop Services

```bash
cd /path/to/PopTail-admin
./dev-stop.sh
```

## Restart Services

```bash
cd /path/to/PopTail-admin
./dev-restart.sh
```

## Verify Auth Modes

To verify both login behaviors end to end:

```bash
cd /path/to/PopTail-admin
./verify-auth-modes.sh
```

This checks that:

- `multipointEnabled=true` keeps concurrent sessions alive
- `multipointEnabled=false` invalidates the older session on a new login

## Run Frontend

```bash
cd /path/to/PopTail-admin/frontend
pnpm install
../dev-frontend.sh
```

Frontend dev server defaults to:

```text
http://127.0.0.1:5666
```

The frontend is configured to call:

```text
VITE_POP_TAIL_API_BASE=http://127.0.0.1:8888
```

## Default Login

```text
username: admin
password: 123456
```
