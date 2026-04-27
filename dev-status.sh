#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$ROOT_DIR/.run"
FRONTEND_PID_FILE="$RUN_DIR/frontend.pid"
BACKEND_HEALTH_URL="http://127.0.0.1:8888/healthz"
FRONTEND_URL="http://127.0.0.1:5666/"

check_url() {
  local url="$1"
  if curl -fsS "$url" >/dev/null 2>&1; then
    echo "up"
  else
    echo "down"
  fi
}

frontend_pid=""
frontend_pid_status="missing"
if [[ -f "$FRONTEND_PID_FILE" ]]; then
  frontend_pid="$(cat "$FRONTEND_PID_FILE" 2>/dev/null || true)"
  if [[ -n "$frontend_pid" ]] && kill -0 "$frontend_pid" >/dev/null 2>&1; then
    frontend_pid_status="running"
  else
    frontend_pid_status="stale"
  fi
fi

backend_status="$(check_url "$BACKEND_HEALTH_URL")"
frontend_status="$(check_url "$FRONTEND_URL")"

echo "Backend:"
echo "  URL:    $BACKEND_HEALTH_URL"
echo "  State:  $backend_status"
echo
echo "Frontend:"
echo "  URL:    $FRONTEND_URL"
echo "  State:  $frontend_status"
echo "  PID:    ${frontend_pid:-"-"} ($frontend_pid_status)"
