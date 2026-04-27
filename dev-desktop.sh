#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_URL="http://127.0.0.1:5666/"
BACKEND_HEALTH_URL="http://127.0.0.1:8888/healthz"

if ! curl -fsS "$FRONTEND_URL" >/dev/null 2>&1 || ! curl -fsS "$BACKEND_HEALTH_URL" >/dev/null 2>&1; then
  echo "Frontend or backend is not ready, starting full stack first..."
  "$ROOT_DIR/dev-up.sh"
fi

echo "Launching desktop shell..."
cd "$ROOT_DIR/desktop-shell/src-tauri"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export POP_TAIL_DESKTOP_LOG="${POP_TAIL_DESKTOP_LOG:-1}"
export POP_TAIL_DESKTOP_OPEN_DEVTOOLS="${POP_TAIL_DESKTOP_OPEN_DEVTOOLS:-1}"
cargo run
