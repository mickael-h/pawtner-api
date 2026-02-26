#!/usr/bin/env bash
# Test GET /api/v1/me with user mickaelh (Keycloak pawtner realm).
# Requires: Keycloak running, a client with "Direct access grants" enabled.
#
# Usage:
#   export KEYCLOAK_USER=mickaelh
#   export KEYCLOAK_PASSWORD=your_password
#   ./scripts/test-me.sh
#
# Optional: KEYCLOAK_ISSUER (default realm root below; do NOT use .../account/),
#           KEYCLOAK_CLIENT_ID (must be a client with Direct access grants),
#           API_BASE (default http://localhost:3000)
#
# Note: "account" and the account console URL (../realms/pawtner/account/) are
# for the Keycloak web UI only. For this script you need a separate client
# (e.g. pawtner-mobile) with "Direct access grants" enabled in the pawtner realm.
# The API validates the JWT audience against KEYCLOAK_AUDIENCE (default
# "pawtner-mobile"), so keep KEYCLOAK_CLIENT_ID and the API's
# KEYCLOAK_AUDIENCE consistent.

set -e

# Issuer must be the realm root (token URL is issuer + /protocol/openid-connect/token)
KEYCLOAK_ISSUER="${KEYCLOAK_ISSUER:-http://localhost:8080/realms/pawtner}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-pawtner-mobile}"
KEYCLOAK_USER="${KEYCLOAK_USER:-mickaelh}"
KEYCLOAK_PASSWORD="${KEYCLOAK_PASSWORD:?Set KEYCLOAK_PASSWORD (password for mickaelh in Keycloak)}"
API_BASE="${API_BASE:-http://localhost:3000}"

TOKEN_URL="${KEYCLOAK_ISSUER}/protocol/openid-connect/token"

echo "Token URL: $TOKEN_URL"
echo "Getting token for user $KEYCLOAK_USER (client: $KEYCLOAK_CLIENT_ID)..."
RESP=$(curl -s -X POST "$TOKEN_URL" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=$KEYCLOAK_CLIENT_ID" \
  -d "username=$KEYCLOAK_USER" \
  -d "password=$KEYCLOAK_PASSWORD" \
  -d "scope=openid profile email")

if echo "$RESP" | grep -q "error"; then
  echo "Token request failed: $RESP"
  exit 1
fi

ACCESS_TOKEN=$(echo "$RESP" | jq -r '.access_token')
if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" = "null" ]; then
  echo "No access_token in response: $RESP"
  exit 1
fi

echo "Calling GET $API_BASE/api/v1/me..."
BODY=$(mktemp)
HTTP_CODE=$(curl -s -o "$BODY" -w "%{http_code}" -H "Authorization: Bearer $ACCESS_TOKEN" "$API_BASE/api/v1/me")
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
