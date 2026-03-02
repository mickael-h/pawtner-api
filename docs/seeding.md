# Data Seeding Contract

## Local bootstrap

1. Start infra stack (from infra project):
   - `docker compose up -d`
2. Set API `.env` values (`DATABASE_URL`, Keycloak issuer/JWKS/audience).
3. Start API:
   - `cargo run`

On startup, API executes `sqlx::migrate!()`:

- schema migration
- demo data migration
- identity mapping migration (`keycloak_sub`)

## Idempotency

- Seed migration uses `ON CONFLICT` updates.
- Running migrations repeatedly should not duplicate rows.

## Expected seeded entities

- users: merchants + clients demo rows
- profiles: merchant profiles for demo merchants
- offers: marketplace demo offers
- reviews: merchant reviews
- orders: demo client orders
- metrics: 6 months of sales points minimum

## Reset instructions

Use the infra Postgres reset flow, then restart API so migrations and seeds run again.

