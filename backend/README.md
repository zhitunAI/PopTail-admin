# PopTail Auth Server (Rust)

## Overview

This server provides a compatibility-focused auth layer for PopTail-admin.

Implemented compatibility contracts:

- `POST /base/login`
- `POST /base/refresh`
- `POST /base/logout`
- `GET /user/getUserInfo`
- `POST /casbin/getPolicyPathByAuthorityId`
- `POST /user/switchAuthority` / `POST /user/setUserAuthority`

Implemented AI operation endpoint:

- `POST /ai/moderation/decision`

## Runtime behavior

- Access + refresh token session model
- Rolling refresh compatibility headers:
  - `new-token`
  - `new-expires-at`
- Session revocation and logout invalidation
- Casbin-first authorization adapter
- AI service identity and delegated execution checks
- Audit records persisted for moderation decisions
- Multipoint session coordination can use Redis when `POP_TAIL_MULTIPOINT_ENABLED=true` and `POP_TAIL_REDIS_URL` is configured

## Persistence and environment

The server prefers `POP_TAIL_DATABASE_URL` and falls back to `POP_TAIL_SQLITE_URL`. If neither is set, it defaults to local SQLite:

```bash
export POP_TAIL_SQLITE_URL="sqlite://pop_tail_auth.db"
```

Supported persistence targets:

- SQLite via `sqlite://...` URLs
- PostgreSQL via `postgres://...` or `postgresql://...` URLs
- In-memory fallback if database initialization fails

On startup, persisted sessions and audits are loaded back into memory from the configured SQLite/PostgreSQL store.

Common environment variables:

```bash
export POP_TAIL_BIND_ADDR="0.0.0.0:8888"
export POP_TAIL_POSTGRES_PASSWORD="replace-with-a-strong-postgres-password"
export POP_TAIL_DATABASE_URL="postgres://pop_tail:${POP_TAIL_POSTGRES_PASSWORD}@127.0.0.1:15432/pop_tail_auth"
export POP_TAIL_REDIS_URL="redis://127.0.0.1:16379/0"
export POP_TAIL_JWT_SECRET="replace-with-at-least-32-random-characters"
export POP_TAIL_SERVICE_MODERATION_TOKEN="replace-with-at-least-32-random-characters"
export POP_TAIL_BOOTSTRAP_ADMIN_PASSWORD="replace-with-at-least-12-random-characters"
export POP_TAIL_DEFAULT_USER_PASSWORD="replace-with-at-least-12-random-characters"
export POP_TAIL_MULTIPOINT_ENABLED="true"
export POP_TAIL_COMPATIBILITY_REFRESH_HEADERS="true"
export RUST_LOG="info"
```

## Local run

Run directly with the default SQLite database:

```bash
cargo run
```

Server listens on `0.0.0.0:8888`.

## Docker Compose

`docker-compose.yml` provisions the Rust auth service together with PostgreSQL and Redis. Copy `.env.compose.example`, then replace every placeholder secret before starting the stack:

- app: `http://127.0.0.1:8888`
- postgres: `127.0.0.1:15432`
- redis: `127.0.0.1:16379`

Quick start:

```bash
cp .env.compose.example .env
docker compose --env-file .env up --build
```

## Verification

Primary verification commands:

```bash
cargo fmt --check
cargo test
```

These cover:

- auth flow tests
- revoke/refresh/switch-authority tests
- SQLite persistence write coverage
- parity contract tests for legacy-compatible response shape
