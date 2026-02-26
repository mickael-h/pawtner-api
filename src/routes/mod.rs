//! Route aggregation.

use axum::routing::get;
use axum::Router;

use crate::handlers;
use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/me", get(handlers::me))
        .with_state(state)
}
