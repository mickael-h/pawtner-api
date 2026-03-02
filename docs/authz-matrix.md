# AuthZ Matrix

Identity source:

- user id: JWT `sub`
- roles: JWT `realm_access.roles`

## Routes

| Method | Path | Allowed roles | Ownership rule |
|---|---|---|---|
| GET | `/health` | anonymous | none |
| GET | `/api/v1/me` | authenticated | none |
| GET | `/api/v1/me/context` | merchant, client | resolves caller from `sub` into `marketplace_users` |
| GET | `/api/v1/marketplace/offers` | anonymous | published offers by default |
| GET | `/api/v1/marketplace/offers/:offer_id` | anonymous | none |
| GET | `/api/v1/marketplace/merchants/:merchant_id/reviews` | anonymous | none |
| GET | `/api/v1/merchant/offers` | merchant | `offer.merchant_user_id == current_merchant.id` |
| POST | `/api/v1/merchant/offers` | merchant | owner is forced to current merchant |
| PATCH | `/api/v1/merchant/offers/:offer_id` | merchant | merchant can mutate only own offers |
| DELETE | `/api/v1/merchant/offers/:offer_id` | merchant | soft-delete only own offers |
| GET | `/api/v1/merchant/profile` | merchant | current merchant only |
| GET | `/api/v1/merchant/orders` | merchant | `order.merchant_user_id == current_merchant.id` |
| GET | `/api/v1/client/orders` | client | `order.client_user_id == current_client.id` |
| GET | `/api/v1/client/orders/:order_id` | client | only own order id is accessible |
| GET | `/api/v1/metrics/monthly-sales` | merchant, client | merchant reads merchant-scoped metrics; client reads global series |

