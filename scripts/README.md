# Test script: GET /api/v1/me

The `test-me.sh` script gets a Keycloak token for user **mickaelh** (or `KEYCLOAK_USER`) and calls the API to retrieve that user's data from the pawtner realm (Keycloak UserInfo).

## Keycloak setup

**Do not use the "account" client** — that is Keycloak's Account Console (the UI at `.../realms/pawtner/account/`) and does not support the password grant used by this script.

1. In realm **pawtner**, create a **new** client (e.g. `pawtner-mobile`):
   - **Client ID:** `pawtner-mobile` (or any name; set `KEYCLOAK_CLIENT_ID` to match).
   - **Client authentication:** OFF (public client).
   - **Direct access grants:** ON (under "Capability config" or "Authentication flow").
   - Save.
2. Ensure user **mickaelh** exists and has a password set.

## Run

```bash
export KEYCLOAK_PASSWORD=the_password_for_mickaelh
./scripts/test-me.sh
```

Optional env vars: `KEYCLOAK_ISSUER` (default `http://localhost:8080/realms/pawtner`), `KEYCLOAK_CLIENT_ID` (default `pawtner-mobile`), `API_BASE` (default `http://localhost:3000`).

Requires `curl` and `jq`. The API and Keycloak must be running.
