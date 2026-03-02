# Scripts

## `smoke-keycloak-me.sh` (manual Keycloak smoke test)

The `smoke-keycloak-me.sh` script gets a Keycloak token for a dev user (default `merchant_demo`) and calls an authenticated API endpoint (default `/api/v1/me/context`).

## Keycloak setup

**Do not use the "account" client** — that is Keycloak's Account Console (the UI at `.../realms/pawtner/account/`) and does not support the password grant used by this script.

1. In realm **pawtner**, create a **new** client (e.g. `pawtner-mobile`):
   - **Client ID:** `pawtner-mobile` (or any name; set `KEYCLOAK_CLIENT_ID` to match).
   - **Client authentication:** OFF (public client).
   - **Direct access grants:** ON (under "Capability config" or "Authentication flow").
   - Save.
2. Ensure a dev user exists and has a password set (for example `merchant_demo` / `dev-merchant-123`).

## Run

```bash
./scripts/smoke-keycloak-me.sh
```

Optional env vars: `KEYCLOAK_ISSUER` (default `http://localhost:18080/realms/pawtner`), `KEYCLOAK_CLIENT_ID` (default `pawtner-mobile`), `KEYCLOAK_USER` (default `merchant_demo`), `KEYCLOAK_PASSWORD` (default `dev-merchant-123`), `API_BASE` (default `http://localhost:3000`), `API_PATH` (default `/api/v1/me/context`).

Requires `curl` and `jq`. Keycloak must be running. The script checks API health and starts `cargo run` automatically if the API is not already running; it tears that process down when done.

## Endpoint integration tests (real DB)

The `tests/endpoints.rs` suite runs against a real Postgres database (no mocked data mode).

Set one of these variables before `cargo test`:

- `TEST_DATABASE_URL` (preferred for tests)
- fallback: `DATABASE_URL`

Example:

```bash
export TEST_DATABASE_URL="postgres://pawtner:dev-secret@localhost:5432/pawtner_db"
cargo test
```

## `test-with-db.sh` (automated local DB lifecycle)

This script starts infra Postgres, waits for readiness, runs tests, then stops Postgres.

Default behavior:

- starts `postgres` from `../infra/docker-compose.yml`
- runs `cargo test --test endpoints`
- stops postgres afterward

Options:

- `--all`: run full `cargo test`
- `--keep-db`: keep postgres running after tests

Examples:

```bash
./scripts/test-with-db.sh
./scripts/test-with-db.sh --all
./scripts/test-with-db.sh --keep-db
```
