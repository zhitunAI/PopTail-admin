#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$ROOT_DIR/.run"
FRONTEND_LOG="$RUN_DIR/frontend.log"
FRONTEND_PID_FILE="$RUN_DIR/frontend.pid"
BACKEND_HEALTH_URL="http://127.0.0.1:8888/healthz"
FRONTEND_URL="http://127.0.0.1:5666/"

mkdir -p "$RUN_DIR"

export POP_TAIL_POSTGRES_PASSWORD="${POP_TAIL_POSTGRES_PASSWORD:-pop_tail_dev_secret}"
export POP_TAIL_DATABASE_URL="${POP_TAIL_DATABASE_URL:-postgres://pop_tail:${POP_TAIL_POSTGRES_PASSWORD}@postgres:5432/pop_tail_auth}"
export POP_TAIL_REDIS_URL="${POP_TAIL_REDIS_URL:-redis://redis:6379/0}"
export POP_TAIL_JWT_SECRET="${POP_TAIL_JWT_SECRET:-pop-tail-dev-secret-change-me-1234567890}"
export POP_TAIL_SERVICE_MODERATION_TOKEN="${POP_TAIL_SERVICE_MODERATION_TOKEN:-pop-tail-moderation-token-dev-1234567890}"
export POP_TAIL_BOOTSTRAP_ADMIN_PASSWORD="${POP_TAIL_BOOTSTRAP_ADMIN_PASSWORD:-123456}"
export POP_TAIL_DEFAULT_USER_PASSWORD="${POP_TAIL_DEFAULT_USER_PASSWORD:-123456}"
export POP_TAIL_MULTIPOINT_ENABLED="${POP_TAIL_MULTIPOINT_ENABLED:-true}"
export POP_TAIL_COMPATIBILITY_REFRESH_HEADERS="${POP_TAIL_COMPATIBILITY_REFRESH_HEADERS:-true}"
export RUST_LOG="${RUST_LOG:-info}"

wait_for_url() {
  local url="$1"
  local label="$2"
  local attempts="${3:-60}"
  local delay="${4:-1}"

  for ((i = 1; i <= attempts; i += 1)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$delay"
  done

  echo "$label failed to become ready: $url" >&2
  return 1
}

frontend_pid=""
if [[ -f "$FRONTEND_PID_FILE" ]]; then
  frontend_pid="$(cat "$FRONTEND_PID_FILE" 2>/dev/null || true)"
fi

if curl -fsS "$BACKEND_HEALTH_URL" >/dev/null 2>&1; then
  echo "Backend already running at $BACKEND_HEALTH_URL"
else
  echo "Starting backend..."
  "$ROOT_DIR/dev-backend.sh"
  wait_for_url "$BACKEND_HEALTH_URL" "Backend health check"
fi

if curl -fsS "$FRONTEND_URL" >/dev/null 2>&1; then
  echo "Frontend already running at $FRONTEND_URL"
elif [[ -n "$frontend_pid" ]] && kill -0 "$frontend_pid" >/dev/null 2>&1; then
  echo "Frontend process already running (pid $frontend_pid), waiting for readiness..."
  wait_for_url "$FRONTEND_URL" "Frontend readiness"
else
  if [[ -n "$frontend_pid" ]]; then
    rm -f "$FRONTEND_PID_FILE"
  fi

  echo "Starting frontend..."
  (
    cd "$ROOT_DIR/frontend"
    nohup pnpm -C apps/web-antd dev --host 127.0.0.1 >"$FRONTEND_LOG" 2>&1 &
    echo $! >"$FRONTEND_PID_FILE"
  )
  wait_for_url "$FRONTEND_URL" "Frontend readiness"
fi

echo
echo "Services are ready:"
echo "  Frontend: $FRONTEND_URL"
echo "  Backend:  http://127.0.0.1:8888"
echo
echo "Frontend log: $FRONTEND_LOG"
