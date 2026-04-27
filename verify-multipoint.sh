#!/usr/bin/env bash
set -euo pipefail

BACKEND_URL="${BACKEND_URL:-http://127.0.0.1:8888}"
LOGIN_URL="$BACKEND_URL/base/login"
USER_INFO_URL="$BACKEND_URL/user/getUserInfo"
SYSTEM_CONFIG_URL="$BACKEND_URL/system/setSystemConfig"
HEALTH_URL="$BACKEND_URL/healthz"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_cmd curl
require_cmd python3

for _ in $(seq 1 20); do
  if curl -fsS "$HEALTH_URL" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -fsS "$HEALTH_URL" >/dev/null 2>&1; then
  echo "backend is not reachable at $HEALTH_URL" >&2
  exit 1
fi

json_field() {
  local payload="$1"
  local field="$2"
  PAYLOAD_JSON="$payload" python3 -c '
import json, os, sys
field = sys.argv[1]
payload = json.loads(os.environ["PAYLOAD_JSON"])
value = payload
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
  curl -fsS "$USER_INFO_URL" \
    -H "x-token: $token"
}

echo "1/4 Enabling multipoint login in system config..."
bootstrap_payload="$(login)"
bootstrap_token="$(json_field "$bootstrap_payload" 'data.token')"
curl -fsS "$SYSTEM_CONFIG_URL" \
  -H 'Content-Type: application/json' \
  -H "x-token: $bootstrap_token" \
  -d '{
    "bindAddress":"0.0.0.0:8888",
    "databaseUrl":"postgres://gaa:gaa@postgres:5432/gaa",
    "redisUrl":"redis://redis:6379",
    "multipointEnabled":true,
    "compatibilityRefreshHeaders":true
  }' >/dev/null

echo "2/4 Logging in session A..."
payload_a="$(login)"
token_a="$(json_field "$payload_a" 'data.token')"

echo "3/4 Logging in session B..."
payload_b="$(login)"
token_b="$(json_field "$payload_b" 'data.token')"

echo "4/4 Verifying both sessions remain valid..."
user_a="$(call_user_info "$token_a")"
user_b="$(call_user_info "$token_b")"

user_name_a="$(json_field "$user_a" 'data.userInfo.userName')"
user_name_b="$(json_field "$user_b" 'data.userInfo.userName')"

cat <<EOF
Multipoint verification passed.
- Session A user: $user_name_a
- Session B user: $user_name_b
- Both tokens remained valid after concurrent login.
EOF
