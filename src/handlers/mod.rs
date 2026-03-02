//! HTTP handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::json;

use crate::domain::marketplace::{
    archive_merchant_offer, create_merchant_offer, ensure_marketplace_user, get_client_order,
    get_merchant_profile, get_merchant_reviews, get_offer_by_id, list_client_orders,
    list_merchant_offers, list_merchant_orders, list_monthly_sales_metrics, list_public_offers,
    require_role, NewOffer, OffersQuery, UpdateOffer,
};
use crate::domain::{
    validate_animal_type, validate_birth_date, validate_cycle_status, validate_gender,
    validate_listing_type, validate_offer_status, validate_uuid, UserRole,
};
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

#[derive(Debug, serde::Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// GET /api/v1/me/context — auth context + resolved marketplace user.
pub async fn me_context(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = ensure_marketplace_user(&state.db, &claims).await?;
    Ok(Json(json!({
        "sub": claims.sub,
        "exp": claims.exp,
        "iss": claims.iss,
        "roles": claims.realm_access.map(|ra| ra.roles).unwrap_or_default(),
        "marketplaceUser": user
    })))
}

/// GET /api/v1/marketplace/offers
pub async fn marketplace_offers(
    State(state): State<AppState>,
    Query(query): Query<OffersQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if let Some(animal_type) = query.animal_type.as_deref() {
        validate_animal_type(animal_type, "animal_type")?;
    }
    if let Some(listing_type) = query.listing_type.as_deref() {
        validate_listing_type(listing_type, "listing_type")?;
    }
    if let Some(status) = query.status.as_deref() {
        validate_offer_status(status, "status")?;
    }
    let paged = list_public_offers(&state.db, query).await?;
    Ok(Json(json!(paged)))
}

/// GET /api/v1/marketplace/offers/:offer_id
pub async fn marketplace_offer_by_id(
    State(state): State<AppState>,
    Path(offer_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_uuid(&offer_id, "offer_id")?;
    let offer = get_offer_by_id(&state.db, &offer_id).await?;
    Ok(Json(json!(offer)))
}

/// GET /api/v1/merchant/offers
pub async fn merchant_offers(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<OffersQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    if let Some(status) = query.status.as_deref() {
        validate_offer_status(status, "status")?;
    }
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let paged = list_merchant_offers(&state.db, &merchant.id, query).await?;
    Ok(Json(json!(paged)))
}

/// POST /api/v1/merchant/offers
pub async fn merchant_create_offer(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(input): Json<NewOffer>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    validate_animal_type(&input.animal_type, "animal_type")?;
    validate_listing_type(&input.listing_type, "listing_type")?;
    validate_gender(&input.gender, "gender")?;
    validate_birth_date(&input.birth_date, "birth_date")?;
    if let Some(cycle_status) = input.cycle_status.as_deref() {
        validate_cycle_status(cycle_status, "cycle_status")?;
    }
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let created = create_merchant_offer(&state.db, &merchant.id, input).await?;
    Ok(Json(json!(created)))
}

/// PATCH /api/v1/merchant/offers/:offer_id
pub async fn merchant_update_offer(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(offer_id): Path<String>,
    Json(input): Json<UpdateOffer>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    validate_uuid(&offer_id, "offer_id")?;
    if let Some(animal_type) = input.animal_type.as_deref() {
        validate_animal_type(animal_type, "animal_type")?;
    }
    if let Some(listing_type) = input.listing_type.as_deref() {
        validate_listing_type(listing_type, "listing_type")?;
    }
    if let Some(status) = input.status.as_deref() {
        validate_offer_status(status, "status")?;
    }
    if let Some(cycle_status) = input.cycle_status.as_deref() {
        validate_cycle_status(cycle_status, "cycle_status")?;
    }
    if let Some(gender) = input.gender.as_deref() {
        validate_gender(gender, "gender")?;
    }
    if let Some(birth_date) = input.birth_date.as_deref() {
        validate_birth_date(birth_date, "birth_date")?;
    }
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let updated = crate::domain::marketplace::update_merchant_offer(
        &state.db,
        &merchant.id,
        &offer_id,
        input,
    )
    .await?;
    Ok(Json(json!(updated)))
}

/// DELETE /api/v1/merchant/offers/:offer_id (soft delete -> archived)
pub async fn merchant_delete_offer(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(offer_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    validate_uuid(&offer_id, "offer_id")?;
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let archived = archive_merchant_offer(&state.db, &merchant.id, &offer_id).await?;
    Ok(Json(json!(archived)))
}

/// GET /api/v1/merchant/profile
pub async fn merchant_profile(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let profile = get_merchant_profile(&state.db, &merchant.id).await?;
    let reviews = get_merchant_reviews(&state.db, &merchant.id).await?;
    Ok(Json(json!({
        "profile": profile,
        "reviews": reviews
    })))
}

/// GET /api/v1/marketplace/merchants/:merchant_id/reviews
pub async fn merchant_reviews(
    State(state): State<AppState>,
    Path(merchant_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_uuid(&merchant_id, "merchant_id")?;
    let profile = get_merchant_profile(&state.db, &merchant_id).await?;
    let reviews = get_merchant_reviews(&state.db, &merchant_id).await?;
    Ok(Json(json!({
        "profile": profile,
        "reviews": reviews
    })))
}

/// GET /api/v1/client/orders
pub async fn client_orders(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Client)?;
    let client = ensure_marketplace_user(&state.db, &claims).await?;
    let paged = list_client_orders(
        &state.db,
        &client.id,
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(20),
    )
    .await?;
    Ok(Json(json!(paged)))
}

/// GET /api/v1/client/orders/:order_id
pub async fn client_order_by_id(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Client)?;
    validate_uuid(&order_id, "order_id")?;
    let client = ensure_marketplace_user(&state.db, &claims).await?;
    let order = get_client_order(&state.db, &client.id, &order_id).await?;
    Ok(Json(json!(order)))
}

/// GET /api/v1/merchant/orders
pub async fn merchant_orders(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Query(query): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_role(&claims, UserRole::Merchant)?;
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let paged = list_merchant_orders(
        &state.db,
        &merchant.id,
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(20),
    )
    .await?;
    Ok(Json(json!(paged)))
}

/// GET /api/v1/metrics/monthly-sales
pub async fn monthly_sales_metrics(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let merchant = ensure_marketplace_user(&state.db, &claims).await?;
    let merchant_scope = if claims.has_role("merchant") {
        Some(merchant.id.as_str())
    } else {
        None
    };
    let points = list_monthly_sales_metrics(&state.db, merchant_scope).await?;
    Ok(Json(json!({ "series": points })))
}
