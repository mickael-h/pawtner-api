#!/usr/bin/env bash
# Manual smoke test for authenticated API endpoints (Keycloak pawtner realm).
# Requires: Keycloak running, a client with "Direct access grants" enabled.
#
# Usage:
#   export KEYCLOAK_USER=merchant_demo
#   export KEYCLOAK_PASSWORD=your_password
#   ./scripts/smoke-keycloak-me.sh
#
# Optional: KEYCLOAK_ISSUER (default realm root below; do NOT use .../account/),
#           KEYCLOAK_CLIENT_ID (must be a client with Direct access grants),
#           API_BASE (default http://localhost:3000),
#           API_PATH (default /api/v1/me/context),
#           KEYCLOAK_PASSWORD (default dev seed password for merchant_demo)
#
# Note: "account" and the account console URL (../realms/pawtner/account/) are
# for the Keycloak web UI only. For this script you need a separate client
# (e.g. pawtner-mobile) with "Direct access grants" enabled in the pawtner realm.
# The API validates the JWT audience against KEYCLOAK_AUDIENCE (default
# "pawtner-mobile"), so keep KEYCLOAK_CLIENT_ID and the API's
# KEYCLOAK_AUDIENCE consistent.

set -euo pipefail

# Resolve project root (script lives in scripts/).
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Issuer must be the realm root (token URL is issuer + /protocol/openid-connect/token)
KEYCLOAK_ISSUER="${KEYCLOAK_ISSUER:-http://localhost:18080/realms/pawtner}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-pawtner-mobile}"
KEYCLOAK_USER="${KEYCLOAK_USER:-merchant_demo}"
KEYCLOAK_PASSWORD="${KEYCLOAK_PASSWORD:-dev-merchant-123}"
API_BASE="${API_BASE:-http://localhost:3000}"
API_PATH="${API_PATH:-/api/v1/me/context}"
HEALTH_URL="${API_BASE}/health"
CURL_CONNECT_TIMEOUT="${CURL_CONNECT_TIMEOUT:-2}"
CURL_MAX_TIME="${CURL_MAX_TIME:-8}"

API_STARTED_BY_SCRIPT=0
API_PID=""

cleanup() {
  if [ "$API_STARTED_BY_SCRIPT" -eq 1 ] && [ -n "$API_PID" ]; then
    echo "Stopping API process started by script (pid: $API_PID)..."
    kill "$API_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

TOKEN_URL="${KEYCLOAK_ISSUER}/protocol/openid-connect/token"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required"
  exit 1
fi

echo "Checking API health at $HEALTH_URL..."
if ! curl -fsS --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" "$HEALTH_URL" >/dev/null 2>&1; then
  echo "API is not reachable at $HEALTH_URL; starting API with cargo run..."
  (
    cd "$ROOT_DIR"
    cargo run >/tmp/pawtner-api-smoke.log 2>&1
  ) &
  API_PID=$!
  API_STARTED_BY_SCRIPT=1

  for _ in {1..20}; do
    if curl -fsS --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" "$HEALTH_URL" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  if ! curl -fsS --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" "$HEALTH_URL" >/dev/null 2>&1; then
    echo "API did not become ready. Last logs:"
    tail -n 50 /tmp/pawtner-api-smoke.log || true
    exit 1
  fi
fi

echo "Token URL: $TOKEN_URL"
echo "Getting token for user $KEYCLOAK_USER (client: $KEYCLOAK_CLIENT_ID)..."
RESP=$(curl -sS --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" -X POST "$TOKEN_URL" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=$KEYCLOAK_CLIENT_ID" \
  -d "username=$KEYCLOAK_USER" \
  -d "password=$KEYCLOAK_PASSWORD" \
  -d "scope=openid profile email")

if [ "$(echo "$RESP" | jq -r '.error // empty')" != "" ]; then
  echo "Token request failed: $RESP"
  exit 1
fi

ACCESS_TOKEN=$(echo "$RESP" | jq -r '.access_token')
if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" = "null" ]; then
  echo "No access_token in response: $RESP"
  exit 1
fi

JWT_PAYLOAD_B64=$(echo "$ACCESS_TOKEN" | cut -d '.' -f2)
if [ -z "$JWT_PAYLOAD_B64" ]; then
  echo "Access token does not look like a JWT"
  exit 1
fi
case $((${#JWT_PAYLOAD_B64} % 4)) in
  2) JWT_PAYLOAD_B64="${JWT_PAYLOAD_B64}==" ;;
  3) JWT_PAYLOAD_B64="${JWT_PAYLOAD_B64}=" ;;
esac
JWT_PAYLOAD=$(echo "$JWT_PAYLOAD_B64" | tr '_-' '/+' | base64 -d 2>/dev/null || true)
if [ -z "$JWT_PAYLOAD" ]; then
  echo "Could not decode JWT payload"
  exit 1
fi

SUB=$(echo "$JWT_PAYLOAD" | jq -r '.sub // empty')
if [ -z "$SUB" ]; then
  echo "Token payload is missing 'sub'."
  echo "Decoded payload:"
  echo "$JWT_PAYLOAD" | jq .
  echo
  echo "Check Keycloak token/client settings and make sure this is a normal user access token."
  exit 1
fi

echo "Token subject: $SUB"
echo "Calling GET $API_BASE$API_PATH..."
BODY=$(mktemp)
HTTP_CODE=$(curl -sS --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" -o "$BODY" -w "%{http_code}" -H "Authorization: Bearer $ACCESS_TOKEN" "$API_BASE$API_PATH")
echo "HTTP status: $HTTP_CODE"
if [ "$HTTP_CODE" = "000" ]; then
  echo "Could not connect to API. Is it running at $API_BASE? (e.g. cargo run in pawtner-api)"
  rm -f "$BODY"
  exit 1
fi
if [ "$HTTP_CODE" -ge 400 ]; then
  echo "Error response:"
  cat "$BODY" | jq . 2>/dev/null || cat "$BODY"
  rm -f "$BODY"
  exit 1
fi
echo "User data:"
cat "$BODY" | jq . 2>/dev/null || cat "$BODY"
rm -f "$BODY"
