//! Route aggregation.

use axum::routing::{get, patch};
use axum::Router;

use crate::handlers;
use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/me", get(handlers::me))
        .route("/api/v1/me/context", get(handlers::me_context))
        .route(
            "/api/v1/marketplace/offers",
            get(handlers::marketplace_offers),
        )
        .route(
            "/api/v1/marketplace/offers/:offer_id",
            get(handlers::marketplace_offer_by_id),
        )
        .route(
            "/api/v1/marketplace/merchants/:merchant_id/reviews",
            get(handlers::merchant_reviews),
        )
        .route(
            "/api/v1/merchant/offers",
            get(handlers::merchant_offers).post(handlers::merchant_create_offer),
        )
        .route(
            "/api/v1/merchant/offers/:offer_id",
            patch(handlers::merchant_update_offer).delete(handlers::merchant_delete_offer),
        )
        .route("/api/v1/merchant/profile", get(handlers::merchant_profile))
        .route("/api/v1/merchant/orders", get(handlers::merchant_orders))
        .route("/api/v1/client/orders", get(handlers::client_orders))
        .route(
            "/api/v1/client/orders/:order_id",
            get(handlers::client_order_by_id),
        )
        .route(
            "/api/v1/metrics/monthly-sales",
            get(handlers::monthly_sales_metrics),
        )
        .with_state(state)
}
