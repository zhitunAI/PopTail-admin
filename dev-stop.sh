#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$ROOT_DIR/.run"
FRONTEND_PID_FILE="$RUN_DIR/frontend.pid"

if [[ -f "$FRONTEND_PID_FILE" ]]; then
  frontend_pid="$(cat "$FRONTEND_PID_FILE" 2>/dev/null || true)"
  if [[ -n "${frontend_pid:-}" ]] && kill -0 "$frontend_pid" >/dev/null 2>&1; then
    echo "Stopping frontend process $frontend_pid..."
    kill "$frontend_pid" >/dev/null 2>&1 || true
  fi
  rm -f "$FRONTEND_PID_FILE"
fi

echo "Stopping docker compose services..."
"$ROOT_DIR/dev-down.sh"
