# Domain Schema Dictionary

## `marketplace_users`

- Primary key: `id` (UUID)
- External identity: `keycloak_sub` (unique, nullable for legacy seed rows)
- Human identity fields: `keycloak_username`, `email`, `display_name`
- Role: `role` (`merchant` or `client`)

## `merchant_profiles`

- Primary key / FK: `merchant_user_id -> marketplace_users.id`
- Merchant-facing fields: `label_score`, `is_certified`, `is_family_style`, `location`, `specialties[]`

## `marketplace_offers`

- Primary key: `id`
- Owner FK: `merchant_user_id -> marketplace_users.id`
- Domain fields: animal, breed, listing type, price, lifecycle status, text description
- State: `status` (`draft`, `published`, `archived`)

## `merchant_reviews`

- Primary key: `id`
- FK: `merchant_user_id -> marketplace_users.id`
- Rating invariant: 1..=5

## `marketplace_orders`

- Primary key: `id`
- FKs:
  - `client_user_id -> marketplace_users.id`
  - `merchant_user_id -> marketplace_users.id`
  - `offer_id -> marketplace_offers.id`
- Status invariant: `pending`, `confirmed`, `completed`, `cancelled`

## `marketplace_monthly_sales_metrics`

- Primary key: `id`
- Optional FK: `merchant_user_id -> marketplace_users.id`
- Time axis: `metric_year`, `month_index` (1..=12)

## Ownership invariants

- Offer owner is `marketplace_offers.merchant_user_id`.
- Client owner is `marketplace_orders.client_user_id`.
- Merchant scope for orders is `marketplace_orders.merchant_user_id`.
- API derives caller identity from JWT `sub`; owner IDs in payload are ignored.

