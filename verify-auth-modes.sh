#!/usr/bin/env bash
set -euo pipefail

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:8888}"
HEALTH_URL="$BACKEND_URL/healthz"
LOGIN_URL="$BACKEND_URL/base/login"
USER_INFO_URL="$BACKEND_URL/user/getUserInfo"
SYSTEM_CONFIG_URL="$BACKEND_URL/system/setSystemConfig"
FINAL_MULTIPOINT_ENABLED="${FINAL_MULTIPOINT_ENABLED:-true}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_cmd curl
require_cmd python3

wait_for_backend() {
  for _ in $(seq 1 20); do
    if curl -fsS "$HEALTH_URL" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "backend is not reachable at $HEALTH_URL" >&2
  exit 1
}

json_field() {
  local payload="$1"
  local field="$2"
  PAYLOAD_JSON="$payload" python3 -c '
import json, os, sys
field = sys.argv[1]
value = json.loads(os.environ["PAYLOAD_JSON"])
for part in field.split("."):
    value = value[part]
print(value)
' "$field"
}

login() {
  curl -fsS "$LOGIN_URL" \
    -H 'Content-Type: application/json' \
    -d '{"username":"admin","password":"123456"}'
}

call_user_info() {
  local token="$1"
  curl -sS -o - -w $'\n%{http_code}' "$USER_INFO_URL" \
    -H "x-token: $token"
}

assert_user_status() {
  local response="$1"
  local expected_status="$2"
  local label="$3"
  local body="${response%$'\n'*}"
  local status="${response##*$'\n'}"

  if [[ "$status" != "$expected_status" ]]; then
    echo "$label expected HTTP $expected_status, got $status" >&2
    echo "$body" >&2
    exit 1
  fi
}

set_multipoint_mode() {
  local enabled="$1"
  local bootstrap_payload
  local bootstrap_token

  bootstrap_payload="$(login)"
  bootstrap_token="$(json_field "$bootstrap_payload" 'data.token')"

  curl -fsS "$SYSTEM_CONFIG_URL" \
    -H 'Content-Type: application/json' \
    -H "x-token: $bootstrap_token" \
    -d "{
      \"bindAddress\":\"0.0.0.0:8888\",
      \"databaseUrl\":\"postgres://gaa:gaa@postgres:5432/gaa\",
      \"redisUrl\":\"redis://redis:6379\",
      \"multipointEnabled\":$enabled,
      \"compatibilityRefreshHeaders\":true
    }" >/dev/null
}

verify_multipoint_enabled() {
  echo "1/6 Enabling multipoint login..."
  set_multipoint_mode true

  echo "2/6 Logging in session A..."
  local payload_a
  local token_a
  payload_a="$(login)"
  token_a="$(json_field "$payload_a" 'data.token')"

  echo "3/6 Logging in session B..."
  local payload_b
  local token_b
  payload_b="$(login)"
  token_b="$(json_field "$payload_b" 'data.token')"

  echo "4/6 Verifying both sessions remain valid..."
  local user_a
  local user_b
  user_a="$(call_user_info "$token_a")"
  user_b="$(call_user_info "$token_b")"
  assert_user_status "$user_a" 200 "multipoint session A"
  assert_user_status "$user_b" 200 "multipoint session B"
}

verify_multipoint_disabled() {
  echo "5/6 Disabling multipoint login..."
  set_multipoint_mode false

  local payload_a
  local token_a
  payload_a="$(login)"
  token_a="$(json_field "$payload_a" 'data.token')"

  local payload_b
  local token_b
  payload_b="$(login)"
  token_b="$(json_field "$payload_b" 'data.token')"

  echo "6/6 Verifying the older session is invalidated..."
  local user_a
  local user_b
  user_a="$(call_user_info "$token_a")"
  user_b="$(call_user_info "$token_b")"
  assert_user_status "$user_a" 401 "single-point session A"
  assert_user_status "$user_b" 200 "single-point session B"
}

restore_final_mode() {
  echo "Restoring multipoint login to $FINAL_MULTIPOINT_ENABLED..."
  set_multipoint_mode "$FINAL_MULTIPOINT_ENABLED"
}

wait_for_backend
trap 'restore_final_mode >/dev/null 2>&1 || true' EXIT
verify_multipoint_enabled
verify_multipoint_disabled

cat <<'EOF'
Auth mode verification passed.
- multipointEnabled=true keeps concurrent sessions alive
- multipointEnabled=false invalidates the older session on new login
EOF
