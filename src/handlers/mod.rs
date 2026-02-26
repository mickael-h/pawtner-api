//! HTTP handlers.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::error::ApiError;
use crate::middleware::{AuthUser, RawBearerToken};
use crate::state::AppState;

/// GET /health — no auth, for load balancers and Docker healthcheck.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// GET /api/v1/me — protected; returns current user data from Keycloak UserInfo (pawtner realm).
pub async fn me(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    RawBearerToken(token): RawBearerToken,
) -> Result<Json<serde_json::Value>, ApiError> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&state.keycloak_userinfo_uri)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("Keycloak UserInfo request failed: {:?}", e);
            ApiError::Internal(anyhow::anyhow!("userinfo unavailable"))
        })?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("Keycloak UserInfo error {}: {}", status, body);
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(ApiError::Unauthorized("userinfo unauthorized".to_string()));
        }
        return Err(ApiError::Internal(anyhow::anyhow!(
            "userinfo returned {}",
            status
        )));
    }
    let user: serde_json::Value = resp.json().await.map_err(|e| {
        tracing::warn!("Keycloak UserInfo parse error: {:?}", e);
        ApiError::Internal(anyhow::anyhow!("invalid userinfo response"))
    })?;
    Ok(Json(user))
}
