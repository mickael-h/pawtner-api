use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{OfferStatus, UserRole};
use crate::error::ApiError;
use crate::middleware::KeycloakClaims;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct MarketplaceUser {
    pub id: String,
    pub keycloak_sub: Option<String>,
    pub keycloak_username: String,
    pub role: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Offer {
    pub id: String,
    pub offer_code: String,
    pub merchant_user_id: String,
    pub name: String,
    pub animal_type: String,
    pub breed: String,
    pub gender: String,
    pub birth_date: String,
    pub price_eur: f64,
    pub location: String,
    pub listing_type: String,
    pub image_url: String,
    pub cycle_status: Option<String>,
    pub is_available_for_club: bool,
    pub description: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Order {
    pub id: String,
    pub order_code: String,
    pub client_user_id: String,
    pub merchant_user_id: String,
    pub offer_id: String,
    pub status: String,
    pub amount_eur: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct MerchantReview {
    pub id: String,
    pub review_code: String,
    pub merchant_user_id: String,
    pub author_name: String,
    pub rating: i32,
    pub comment: String,
    pub reviewed_at: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct MerchantProfile {
    pub merchant_user_id: String,
    pub profile_code: String,
    pub label_score: i32,
    pub is_certified: bool,
    pub is_family_style: bool,
    pub location: String,
    pub specialties: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct MonthlySalesPoint {
    pub metric_year: i32,
    pub month_index: i32,
    pub amount_eur: f64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OffersQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub animal_type: Option<String>,
    pub listing_type: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewOffer {
    pub name: String,
    pub animal_type: String,
    pub breed: String,
    pub gender: String,
    pub birth_date: String,
    pub price_eur: f64,
    pub location: String,
    pub listing_type: String,
    pub image_url: String,
    pub cycle_status: Option<String>,
    pub is_available_for_club: bool,
    pub description: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateOffer {
    pub name: Option<String>,
    pub animal_type: Option<String>,
    pub breed: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub price_eur: Option<f64>,
    pub location: Option<String>,
    pub listing_type: Option<String>,
    pub image_url: Option<String>,
    pub cycle_status: Option<String>,
    pub is_available_for_club: Option<bool>,
    pub description: Option<String>,
    pub status: Option<String>,
}

const MERCHANT_DEMO_ID: &str = "11111111-1111-4111-8111-111111111111";
const MERCHANT_B2_ID: &str = "11111111-1111-4111-8111-111111111112";
const CLIENT_DEMO_ID: &str = "22222222-2222-4222-8222-222222222221";
const CLIENT_ALICE_ID: &str = "22222222-2222-4222-8222-222222222222";

fn use_mock_marketplace_data() -> bool {
    std::env::var("MOCK_MARKETPLACE_DATA")
        .ok()
        .as_deref()
        .is_some_and(|v| v == "1")
}

fn paginate<T: Clone>(items: Vec<T>, page: i64, page_size: i64) -> Paged<T> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 50);
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(items.len());
    let slice = if start >= items.len() {
        Vec::new()
    } else {
        items[start..end].to_vec()
    };
    Paged {
        total: items.len() as i64,
        items: slice,
        page,
        page_size,
    }
}

fn mock_offers() -> Vec<Offer> {
    vec![
        Offer {
            id: "33333333-3333-4333-8333-333333333331".to_string(),
            offer_code: "a1".to_string(),
            merchant_user_id: MERCHANT_DEMO_ID.to_string(),
            name: "Rudy".to_string(),
            animal_type: "dog".to_string(),
            breed: "Golden Retriever".to_string(),
            gender: "M".to_string(),
            birth_date: "2023-05-12".to_string(),
            price_eur: 1800.0,
            location: "Lyon, FR".to_string(),
            listing_type: "sale".to_string(),
            image_url: "https://example.org/rudy.jpg".to_string(),
            cycle_status: None,
            is_available_for_club: false,
            description: "Mock offer".to_string(),
            status: "published".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        },
        Offer {
            id: "33333333-3333-4333-8333-333333333333".to_string(),
            offer_code: "a3".to_string(),
            merchant_user_id: MERCHANT_B2_ID.to_string(),
            name: "Max".to_string(),
            animal_type: "dog".to_string(),
            breed: "Berger Australien".to_string(),
            gender: "M".to_string(),
            birth_date: "2021-02-15".to_string(),
            price_eur: 800.0,
            location: "Bordeaux, FR".to_string(),
            listing_type: "stud".to_string(),
            image_url: "https://example.org/max.jpg".to_string(),
            cycle_status: None,
            is_available_for_club: true,
            description: "Mock offer".to_string(),
            status: "published".to_string(),
            created_at: "2025-01-02T00:00:00Z".to_string(),
        },
    ]
}

fn mock_orders() -> Vec<Order> {
    vec![
        Order {
            id: "55555555-5555-4555-8555-555555555551".to_string(),
            order_code: "o1".to_string(),
            client_user_id: CLIENT_DEMO_ID.to_string(),
            merchant_user_id: MERCHANT_DEMO_ID.to_string(),
            offer_id: "33333333-3333-4333-8333-333333333331".to_string(),
            status: "completed".to_string(),
            amount_eur: 1800.0,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        },
        Order {
            id: "55555555-5555-4555-8555-555555555553".to_string(),
            order_code: "o3".to_string(),
            client_user_id: CLIENT_DEMO_ID.to_string(),
            merchant_user_id: MERCHANT_B2_ID.to_string(),
            offer_id: "33333333-3333-4333-8333-333333333333".to_string(),
            status: "pending".to_string(),
            amount_eur: 800.0,
            created_at: "2025-01-03T00:00:00Z".to_string(),
            updated_at: "2025-01-03T00:00:00Z".to_string(),
        },
    ]
}

fn mock_metrics() -> Vec<MonthlySalesPoint> {
    vec![
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 1,
            amount_eur: 8600.0,
        },
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 2,
            amount_eur: 9200.0,
        },
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 3,
            amount_eur: 10100.0,
        },
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 4,
            amount_eur: 9700.0,
        },
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 5,
            amount_eur: 11200.0,
        },
        MonthlySalesPoint {
            metric_year: 2025,
            month_index: 6,
            amount_eur: 12400.0,
        },
    ]
}

fn mock_user_from_claims(claims: &KeycloakClaims) -> Result<MarketplaceUser, ApiError> {
    let role = claim_role(claims)?;
    let username = claims
        .preferred_username
        .clone()
        .unwrap_or_else(|| claims.sub.clone());
    let id = match username.as_str() {
        "merchant_demo" => MERCHANT_DEMO_ID.to_string(),
        "client_demo" => CLIENT_DEMO_ID.to_string(),
        "client_alice" => CLIENT_ALICE_ID.to_string(),
        _ => Uuid::new_v5(&Uuid::NAMESPACE_OID, claims.sub.as_bytes()).to_string(),
    };
    Ok(MarketplaceUser {
        id,
        keycloak_sub: Some(claims.sub.clone()),
        keycloak_username: username.clone(),
        role: role.as_str().to_string(),
        email: claims
            .email
            .clone()
            .unwrap_or_else(|| format!("{}@pawtner.local", username)),
        display_name: claims.name.clone().unwrap_or(username),
    })
}

pub fn claim_role(claims: &KeycloakClaims) -> Result<UserRole, ApiError> {
    if claims.has_role("merchant") {
        return Ok(UserRole::Merchant);
    }
    if claims.has_role("client") {
        return Ok(UserRole::Client);
    }
    Err(ApiError::Forbidden(
        "token has no marketplace role".to_string(),
    ))
}

pub fn require_role(claims: &KeycloakClaims, role: UserRole) -> Result<(), ApiError> {
    let ok = match role {
        UserRole::Merchant => claims.has_role("merchant"),
        UserRole::Client => claims.has_role("client"),
    };
    if ok {
        Ok(())
    } else {
        Err(ApiError::Forbidden(format!(
            "required role {}",
            role.as_str()
        )))
    }
}

pub async fn ensure_marketplace_user(
    db: &PgPool,
    claims: &KeycloakClaims,
) -> Result<MarketplaceUser, ApiError> {
    if use_mock_marketplace_data() {
        return mock_user_from_claims(claims);
    }
    if let Some(found) = sqlx::query_as::<_, MarketplaceUser>(
        r#"
        SELECT
          id::text AS id,
          keycloak_sub,
          keycloak_username,
          role,
          email,
          display_name
        FROM marketplace_users
        WHERE keycloak_sub = $1
        "#,
    )
    .bind(&claims.sub)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db error: {}", e)))?
    {
        return Ok(found);
    }

    let role = claim_role(claims)?;
    let role_str = role.as_str();
    let username = claims
        .preferred_username
        .clone()
        .unwrap_or_else(|| claims.sub.clone());
    let email = claims
        .email
        .clone()
        .unwrap_or_else(|| format!("{}@pawtner.local", claims.sub));
    let display_name = claims.name.clone().unwrap_or_else(|| username.clone());

    let existing_sub_for_username = sqlx::query_scalar::<_, String>(
        r#"
        SELECT keycloak_sub
        FROM marketplace_users
        WHERE keycloak_username = $1
          AND keycloak_sub IS NOT NULL
        LIMIT 1
        "#,
    )
    .bind(&username)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db error: {}", e)))?;

    if let Some(existing_sub) = existing_sub_for_username {
        if existing_sub != claims.sub {
            return Err(ApiError::Forbidden(
                "identity conflict: username already linked to another subject".to_string(),
            ));
        }
    }

    if let Some(linked) = sqlx::query_as::<_, MarketplaceUser>(
        r#"
        UPDATE marketplace_users
        SET keycloak_sub = $1, role = $2, email = $3, display_name = $4
        WHERE keycloak_username = $5 AND keycloak_sub IS NULL
        RETURNING
          id::text AS id,
          keycloak_sub,
          keycloak_username,
          role,
          email,
          display_name
        "#,
    )
    .bind(&claims.sub)
    .bind(role_str)
    .bind(&email)
    .bind(&display_name)
    .bind(&username)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db error: {}", e)))?
    {
        return Ok(linked);
    }

    let user_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, claims.sub.as_bytes()).to_string();
    sqlx::query_as::<_, MarketplaceUser>(
        r#"
        INSERT INTO marketplace_users (id, keycloak_sub, keycloak_username, role, email, display_name)
        VALUES ($1::uuid, $2, $3, $4, $5, $6)
        RETURNING
          id::text AS id,
          keycloak_sub,
          keycloak_username,
          role,
          email,
          display_name
        "#,
    )
    .bind(&user_id)
    .bind(&claims.sub)
    .bind(&username)
    .bind(role_str)
    .bind(&email)
    .bind(&display_name)
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db insert error: {}", e)))
}

pub async fn list_public_offers(db: &PgPool, query: OffersQuery) -> Result<Paged<Offer>, ApiError> {
    if use_mock_marketplace_data() {
        let mut offers: Vec<Offer> = mock_offers()
            .into_iter()
            .filter(|o| query.status.as_deref().is_none_or(|s| o.status == s))
            .filter(|o| {
                query
                    .animal_type
                    .as_deref()
                    .is_none_or(|s| o.animal_type == s)
            })
            .filter(|o| {
                query
                    .listing_type
                    .as_deref()
                    .is_none_or(|s| o.listing_type == s)
            })
            .filter(|o| {
                query
                    .location
                    .as_deref()
                    .is_none_or(|s| o.location.to_lowercase().contains(&s.to_lowercase()))
            })
            .collect();
        if query.status.is_none() {
            offers.retain(|o| o.status == OfferStatus::Published.as_str());
        }
        return Ok(paginate(
            offers,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        ));
    }
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * page_size;

    let status = query
        .status
        .or_else(|| Some(OfferStatus::Published.as_str().to_string()));

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM marketplace_offers
        WHERE ($1::text IS NULL OR animal_type = $1)
          AND ($2::text IS NULL OR listing_type = $2)
          AND ($3::text IS NULL OR location ILIKE '%' || $3 || '%')
          AND ($4::text IS NULL OR status = $4)
        "#,
    )
    .bind(query.animal_type.as_deref())
    .bind(query.listing_type.as_deref())
    .bind(query.location.as_deref())
    .bind(status.as_deref())
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db count error: {}", e)))?;

    let items = sqlx::query_as::<_, Offer>(
        r#"
        SELECT
          id::text AS id,
          offer_code,
          merchant_user_id::text AS merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date::text AS birth_date,
          price_eur::float8 AS price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status,
          created_at::text AS created_at
        FROM marketplace_offers
        WHERE ($1::text IS NULL OR animal_type = $1)
          AND ($2::text IS NULL OR listing_type = $2)
          AND ($3::text IS NULL OR location ILIKE '%' || $3 || '%')
          AND ($4::text IS NULL OR status = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(query.animal_type.as_deref())
    .bind(query.listing_type.as_deref())
    .bind(query.location.as_deref())
    .bind(status.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db list error: {}", e)))?;

    Ok(Paged {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_offer_by_id(db: &PgPool, offer_id: &str) -> Result<Offer, ApiError> {
    if use_mock_marketplace_data() {
        return mock_offers()
            .into_iter()
            .find(|o| o.id == offer_id)
            .ok_or_else(|| ApiError::NotFound("offer not found".to_string()));
    }
    sqlx::query_as::<_, Offer>(
        r#"
        SELECT
          id::text AS id,
          offer_code,
          merchant_user_id::text AS merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date::text AS birth_date,
          price_eur::float8 AS price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status,
          created_at::text AS created_at
        FROM marketplace_offers
        WHERE id = $1::uuid
        "#,
    )
    .bind(offer_id)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db get error: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("offer not found".to_string()))
}

pub async fn list_merchant_offers(
    db: &PgPool,
    merchant_user_id: &str,
    query: OffersQuery,
) -> Result<Paged<Offer>, ApiError> {
    if use_mock_marketplace_data() {
        let offers: Vec<Offer> = mock_offers()
            .into_iter()
            .filter(|o| o.merchant_user_id == merchant_user_id)
            .filter(|o| query.status.as_deref().is_none_or(|s| o.status == s))
            .collect();
        return Ok(paginate(
            offers,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
        ));
    }
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM marketplace_offers
        WHERE merchant_user_id = $1::uuid
          AND ($2::text IS NULL OR status = $2)
        "#,
    )
    .bind(merchant_user_id)
    .bind(query.status.as_deref())
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db count error: {}", e)))?;

    let items = sqlx::query_as::<_, Offer>(
        r#"
        SELECT
          id::text AS id,
          offer_code,
          merchant_user_id::text AS merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date::text AS birth_date,
          price_eur::float8 AS price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status,
          created_at::text AS created_at
        FROM marketplace_offers
        WHERE merchant_user_id = $1::uuid
          AND ($2::text IS NULL OR status = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(merchant_user_id)
    .bind(query.status.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db list error: {}", e)))?;

    Ok(Paged {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn create_merchant_offer(
    db: &PgPool,
    merchant_user_id: &str,
    input: NewOffer,
) -> Result<Offer, ApiError> {
    if use_mock_marketplace_data() {
        if input.price_eur < 0.0 {
            return Err(ApiError::BadRequest("price_eur must be >= 0".to_string()));
        }
        let offer_id = Uuid::new_v4().to_string();
        return Ok(Offer {
            id: offer_id.clone(),
            offer_code: format!("offer-{}", &offer_id[..8]),
            merchant_user_id: merchant_user_id.to_string(),
            name: input.name,
            animal_type: input.animal_type,
            breed: input.breed,
            gender: input.gender,
            birth_date: input.birth_date,
            price_eur: input.price_eur,
            location: input.location,
            listing_type: input.listing_type,
            image_url: input.image_url,
            cycle_status: input.cycle_status,
            is_available_for_club: input.is_available_for_club,
            description: input.description,
            status: OfferStatus::Draft.as_str().to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        });
    }
    if input.price_eur < 0.0 {
        return Err(ApiError::BadRequest("price_eur must be >= 0".to_string()));
    }
    let offer_id = Uuid::new_v4().to_string();
    let offer_code = format!("offer-{}", &offer_id[..8]);

    sqlx::query_as::<_, Offer>(
        r#"
        INSERT INTO marketplace_offers (
          id,
          offer_code,
          merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date,
          price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status
        )
        VALUES (
          $1::uuid, $2, $3::uuid, $4, $5, $6, $7, $8::date, $9, $10, $11, $12, NULLIF($13, ''),
          $14, $15, 'draft'
        )
        RETURNING
          id::text AS id,
          offer_code,
          merchant_user_id::text AS merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date::text AS birth_date,
          price_eur::float8 AS price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status,
          created_at::text AS created_at
        "#,
    )
    .bind(&offer_id)
    .bind(&offer_code)
    .bind(merchant_user_id)
    .bind(&input.name)
    .bind(&input.animal_type)
    .bind(&input.breed)
    .bind(&input.gender)
    .bind(&input.birth_date)
    .bind(input.price_eur)
    .bind(&input.location)
    .bind(&input.listing_type)
    .bind(&input.image_url)
    .bind(input.cycle_status.unwrap_or_default())
    .bind(input.is_available_for_club)
    .bind(&input.description)
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db create error: {}", e)))
}

pub async fn update_merchant_offer(
    db: &PgPool,
    merchant_user_id: &str,
    offer_id: &str,
    input: UpdateOffer,
) -> Result<Offer, ApiError> {
    if use_mock_marketplace_data() {
        let existing = get_offer_by_id(db, offer_id).await?;
        if existing.merchant_user_id != merchant_user_id {
            return Err(ApiError::Forbidden(
                "cannot mutate another merchant's offer".to_string(),
            ));
        }
        if input.price_eur.is_some_and(|price| price < 0.0) {
            return Err(ApiError::BadRequest("price_eur must be >= 0".to_string()));
        }
        return Ok(Offer {
            id: existing.id,
            offer_code: existing.offer_code,
            merchant_user_id: existing.merchant_user_id,
            name: input.name.unwrap_or(existing.name),
            animal_type: input.animal_type.unwrap_or(existing.animal_type),
            breed: input.breed.unwrap_or(existing.breed),
            gender: input.gender.unwrap_or(existing.gender),
            birth_date: input.birth_date.unwrap_or(existing.birth_date),
            price_eur: input.price_eur.unwrap_or(existing.price_eur),
            location: input.location.unwrap_or(existing.location),
            listing_type: input.listing_type.unwrap_or(existing.listing_type),
            image_url: input.image_url.unwrap_or(existing.image_url),
            cycle_status: input.cycle_status.or(existing.cycle_status),
            is_available_for_club: input
                .is_available_for_club
                .unwrap_or(existing.is_available_for_club),
            description: input.description.unwrap_or(existing.description),
            status: input.status.unwrap_or(existing.status),
            created_at: existing.created_at,
        });
    }
    let existing = get_offer_by_id(db, offer_id).await?;
    if existing.merchant_user_id != merchant_user_id {
        return Err(ApiError::Forbidden(
            "cannot mutate another merchant's offer".to_string(),
        ));
    }

    if let Some(price) = input.price_eur {
        if price < 0.0 {
            return Err(ApiError::BadRequest("price_eur must be >= 0".to_string()));
        }
    }

    sqlx::query_as::<_, Offer>(
        r#"
        UPDATE marketplace_offers
        SET
          name = COALESCE($1, name),
          animal_type = COALESCE($2, animal_type),
          breed = COALESCE($3, breed),
          gender = COALESCE($4, gender),
          birth_date = COALESCE($5::date, birth_date),
          price_eur = COALESCE($6, price_eur),
          location = COALESCE($7, location),
          listing_type = COALESCE($8, listing_type),
          image_url = COALESCE($9, image_url),
          cycle_status = CASE WHEN $10 IS NULL THEN cycle_status ELSE NULLIF($10, '') END,
          is_available_for_club = COALESCE($11, is_available_for_club),
          description = COALESCE($12, description),
          status = COALESCE($13, status)
        WHERE id = $14::uuid AND merchant_user_id = $15::uuid
        RETURNING
          id::text AS id,
          offer_code,
          merchant_user_id::text AS merchant_user_id,
          name,
          animal_type,
          breed,
          gender,
          birth_date::text AS birth_date,
          price_eur::float8 AS price_eur,
          location,
          listing_type,
          image_url,
          cycle_status,
          is_available_for_club,
          description,
          status,
          created_at::text AS created_at
        "#,
    )
    .bind(input.name)
    .bind(input.animal_type)
    .bind(input.breed)
    .bind(input.gender)
    .bind(input.birth_date)
    .bind(input.price_eur)
    .bind(input.location)
    .bind(input.listing_type)
    .bind(input.image_url)
    .bind(input.cycle_status)
    .bind(input.is_available_for_club)
    .bind(input.description)
    .bind(input.status)
    .bind(offer_id)
    .bind(merchant_user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db update error: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("offer not found".to_string()))
}

pub async fn archive_merchant_offer(
    db: &PgPool,
    merchant_user_id: &str,
    offer_id: &str,
) -> Result<Offer, ApiError> {
    update_merchant_offer(
        db,
        merchant_user_id,
        offer_id,
        UpdateOffer {
            name: None,
            animal_type: None,
            breed: None,
            gender: None,
            birth_date: None,
            price_eur: None,
            location: None,
            listing_type: None,
            image_url: None,
            cycle_status: None,
            is_available_for_club: None,
            description: None,
            status: Some(OfferStatus::Archived.as_str().to_string()),
        },
    )
    .await
}

pub async fn get_merchant_reviews(
    db: &PgPool,
    merchant_user_id: &str,
) -> Result<Vec<MerchantReview>, ApiError> {
    if use_mock_marketplace_data() {
        let reviews = if merchant_user_id == MERCHANT_DEMO_ID {
            vec![MerchantReview {
                id: "44444444-4444-4444-8444-444444444441".to_string(),
                review_code: "r1".to_string(),
                merchant_user_id: MERCHANT_DEMO_ID.to_string(),
                author_name: "Jean D.".to_string(),
                rating: 5,
                comment: "Top breeder".to_string(),
                reviewed_at: "2024-01-15".to_string(),
            }]
        } else {
            vec![]
        };
        return Ok(reviews);
    }
    sqlx::query_as::<_, MerchantReview>(
        r#"
        SELECT
          id::text AS id,
          review_code,
          merchant_user_id::text AS merchant_user_id,
          author_name,
          rating,
          comment,
          reviewed_at::text AS reviewed_at
        FROM merchant_reviews
        WHERE merchant_user_id = $1::uuid
        ORDER BY reviewed_at DESC
        "#,
    )
    .bind(merchant_user_id)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db reviews error: {}", e)))
}

pub async fn get_merchant_profile(
    db: &PgPool,
    merchant_user_id: &str,
) -> Result<MerchantProfile, ApiError> {
    if use_mock_marketplace_data() {
        if merchant_user_id == MERCHANT_DEMO_ID {
            return Ok(MerchantProfile {
                merchant_user_id: MERCHANT_DEMO_ID.to_string(),
                profile_code: "b1".to_string(),
                label_score: 95,
                is_certified: true,
                is_family_style: true,
                location: "Lyon, FR".to_string(),
                specialties: vec!["Golden Retriever".to_string(), "Maine Coon".to_string()],
            });
        }
        return Err(ApiError::NotFound("merchant profile not found".to_string()));
    }
    sqlx::query_as::<_, MerchantProfile>(
        r#"
        SELECT
          merchant_user_id::text AS merchant_user_id,
          profile_code,
          label_score,
          is_certified,
          is_family_style,
          location,
          specialties
        FROM merchant_profiles
        WHERE merchant_user_id = $1::uuid
        "#,
    )
    .bind(merchant_user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db profile error: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("merchant profile not found".to_string()))
}

pub async fn list_client_orders(
    db: &PgPool,
    client_user_id: &str,
    page: i64,
    page_size: i64,
) -> Result<Paged<Order>, ApiError> {
    if use_mock_marketplace_data() {
        let orders: Vec<Order> = mock_orders()
            .into_iter()
            .filter(|o| o.client_user_id == client_user_id)
            .collect();
        return Ok(paginate(orders, page, page_size));
    }
    let page = page.max(1);
    let page_size = page_size.clamp(1, 50);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM marketplace_orders WHERE client_user_id = $1::uuid",
    )
    .bind(client_user_id)
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db orders count error: {}", e)))?;

    let items = sqlx::query_as::<_, Order>(
        r#"
        SELECT
          id::text AS id,
          order_code,
          client_user_id::text AS client_user_id,
          merchant_user_id::text AS merchant_user_id,
          offer_id::text AS offer_id,
          status,
          amount_eur::float8 AS amount_eur,
          created_at::text AS created_at,
          updated_at::text AS updated_at
        FROM marketplace_orders
        WHERE client_user_id = $1::uuid
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(client_user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db orders list error: {}", e)))?;

    Ok(Paged {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_client_order(
    db: &PgPool,
    client_user_id: &str,
    order_id: &str,
) -> Result<Order, ApiError> {
    if use_mock_marketplace_data() {
        return mock_orders()
            .into_iter()
            .find(|o| o.id == order_id && o.client_user_id == client_user_id)
            .ok_or_else(|| {
                ApiError::Forbidden("order does not belong to current client".to_string())
            });
    }
    sqlx::query_as::<_, Order>(
        r#"
        SELECT
          id::text AS id,
          order_code,
          client_user_id::text AS client_user_id,
          merchant_user_id::text AS merchant_user_id,
          offer_id::text AS offer_id,
          status,
          amount_eur::float8 AS amount_eur,
          created_at::text AS created_at,
          updated_at::text AS updated_at
        FROM marketplace_orders
        WHERE id = $1::uuid AND client_user_id = $2::uuid
        "#,
    )
    .bind(order_id)
    .bind(client_user_id)
    .fetch_optional(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db order get error: {}", e)))?
    .ok_or_else(|| ApiError::Forbidden("order does not belong to current client".to_string()))
}

pub async fn list_merchant_orders(
    db: &PgPool,
    merchant_user_id: &str,
    page: i64,
    page_size: i64,
) -> Result<Paged<Order>, ApiError> {
    if use_mock_marketplace_data() {
        let orders: Vec<Order> = mock_orders()
            .into_iter()
            .filter(|o| o.merchant_user_id == merchant_user_id)
            .collect();
        return Ok(paginate(orders, page, page_size));
    }
    let page = page.max(1);
    let page_size = page_size.clamp(1, 50);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM marketplace_orders WHERE merchant_user_id = $1::uuid",
    )
    .bind(merchant_user_id)
    .fetch_one(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db orders count error: {}", e)))?;

    let items = sqlx::query_as::<_, Order>(
        r#"
        SELECT
          id::text AS id,
          order_code,
          client_user_id::text AS client_user_id,
          merchant_user_id::text AS merchant_user_id,
          offer_id::text AS offer_id,
          status,
          amount_eur::float8 AS amount_eur,
          created_at::text AS created_at,
          updated_at::text AS updated_at
        FROM marketplace_orders
        WHERE merchant_user_id = $1::uuid
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(merchant_user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db orders list error: {}", e)))?;

    Ok(Paged {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn list_monthly_sales_metrics(
    db: &PgPool,
    merchant_user_id: Option<&str>,
) -> Result<Vec<MonthlySalesPoint>, ApiError> {
    if use_mock_marketplace_data() {
        return Ok(mock_metrics());
    }
    sqlx::query_as::<_, MonthlySalesPoint>(
        r#"
        SELECT metric_year, month_index, amount_eur::float8 AS amount_eur
        FROM marketplace_monthly_sales_metrics
        WHERE
          ($1::uuid IS NULL AND merchant_user_id IS NULL)
          OR merchant_user_id = $1::uuid
        ORDER BY metric_year, month_index
        "#,
    )
    .bind(merchant_user_id)
    .fetch_all(db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("db metrics error: {}", e)))
}

#[cfg(test)]
mod tests {
    use crate::middleware::KeycloakClaims;

    use super::{claim_role, require_role};
    use crate::domain::UserRole;

    fn claims_with_roles(roles: &[&str]) -> KeycloakClaims {
        serde_json::from_value(serde_json::json!({
            "sub": "subject-1",
            "exp": 1_900_000_000,
            "realm_access": {
                "roles": roles
            }
        }))
        .expect("claims should parse")
    }

    #[test]
    fn claim_role_prefers_merchant_then_client() {
        let merchant = claims_with_roles(&["merchant"]);
        let client = claims_with_roles(&["client"]);
        let both = claims_with_roles(&["merchant", "client"]);
        let none = claims_with_roles(&["viewer"]);

        assert_eq!(claim_role(&merchant).unwrap(), UserRole::Merchant);
        assert_eq!(claim_role(&client).unwrap(), UserRole::Client);
        assert_eq!(claim_role(&both).unwrap(), UserRole::Merchant);
        assert!(claim_role(&none).is_err());
    }

    #[test]
    fn require_role_checks_expected_role() {
        let merchant = claims_with_roles(&["merchant"]);
        let client = claims_with_roles(&["client"]);

        assert!(require_role(&merchant, UserRole::Merchant).is_ok());
        assert!(require_role(&merchant, UserRole::Client).is_err());
        assert!(require_role(&client, UserRole::Client).is_ok());
        assert!(require_role(&client, UserRole::Merchant).is_err());
    }
}
