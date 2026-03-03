use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use pawtner_api::middleware::JwtValidator;
use pawtner_api::routes::api_router;
use pawtner_api::state::AppState;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::util::ServiceExt;

fn test_token(sub: &str, username: &str, email: &str, name: &str, roles: &[&str]) -> String {
    let claims = json!({
        "sub": sub,
        "exp": 4_000_000_000u64,
        "preferred_username": username,
        "email": email,
        "name": name,
        "realm_access": { "roles": roles },
    });
    let bytes = serde_json::to_vec(&claims).expect("claims should serialize");
    format!("test.{}", URL_SAFE_NO_PAD.encode(bytes))
}

async fn send_json(app: axum::Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp = app.oneshot(req).await.expect("request should be served");
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let parsed = serde_json::from_slice::<serde_json::Value>(&body)
        .unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&body) }));
    (status, parsed)
}

async fn build_test_app() -> (axum::Router, sqlx::PgPool) {
    std::env::set_var("ALLOW_TEST_TOKENS", "1");
    std::env::remove_var("MOCK_MARKETPLACE_DATA");

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for DB-backed endpoint tests");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("test database should be reachable");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("migrations should run");

    sqlx::query(
        r#"
        UPDATE marketplace_users
        SET keycloak_sub = NULL
        WHERE keycloak_username IN ('merchant_demo', 'client_demo', 'client_alice')
        "#,
    )
    .execute(&db)
    .await
    .expect("should reset demo identity links for deterministic tests");

    let state = AppState {
        db: db.clone(),
        jwt_validator: Arc::new(JwtValidator::new(
            "http://localhost/unused-jwks".to_string(),
            "http://localhost/realms/pawtner".to_string(),
            "pawtner-mobile".to_string(),
        )),
        keycloak_userinfo_uri: "http://localhost/unused-userinfo".to_string(),
    };
    (api_router(state), db)
}

#[tokio::test]
async fn endpoints_happy_paths_and_common_errors() {
    let (app, db) = build_test_app().await;

    let merchant_token = test_token(
        "sub-merchant-demo",
        "merchant_demo",
        "merchant.demo@pawtner.local",
        "Merchant Demo",
        &["merchant"],
    );
    let client_token = test_token(
        "sub-client-demo",
        "client_demo",
        "client.demo@pawtner.local",
        "Client Demo",
        &["client"],
    );
    let client_alice_token = test_token(
        "sub-client-alice",
        "client_alice",
        "alice.client@pawtner.local",
        "Alice Martin",
        &["client"],
    );
    let concurrent_link_token = test_token(
        "sub-concurrent-link",
        "concurrent_link_user",
        "concurrent.link@pawtner.local",
        "Concurrent Link",
        &["merchant"],
    );
    let conflict_token = test_token(
        "sub-conflict-new",
        "conflict_identity",
        "conflict.new@pawtner.local",
        "Conflict User",
        &["merchant"],
    );

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/marketplace/offers/not-a-uuid")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/me")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/me/context")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let first_user_id = body["marketplaceUser"]["id"]
        .as_str()
        .expect("marketplace user id should exist")
        .to_string();

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/me/context")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["marketplaceUser"]["id"], first_user_id);

    sqlx::query("DELETE FROM marketplace_users WHERE keycloak_username = $1")
        .bind("concurrent_link_user")
        .execute(&db)
        .await
        .expect("should cleanup concurrent link user before test");

    let request_a = Request::builder()
        .method("GET")
        .uri("/api/v1/me/context")
        .header("Authorization", format!("Bearer {}", concurrent_link_token))
        .body(Body::empty())
        .expect("request should build");
    let request_b = Request::builder()
        .method("GET")
        .uri("/api/v1/me/context")
        .header("Authorization", format!("Bearer {}", concurrent_link_token))
        .body(Body::empty())
        .expect("request should build");

    let (resp_a, resp_b) = tokio::join!(
        send_json(app.clone(), request_a),
        send_json(app.clone(), request_b)
    );
    assert_eq!(resp_a.0, StatusCode::OK);
    assert_eq!(resp_b.0, StatusCode::OK);
    assert_eq!(
        resp_a.1["marketplaceUser"]["id"],
        resp_b.1["marketplaceUser"]["id"]
    );

    let invalid_enum_offer_body = json!({
        "name": "Invalid Enum Offer",
        "animal_type": "bird",
        "breed": "N/A",
        "gender": "M",
        "birth_date": "2024-01-01",
        "price_eur": 100.0,
        "location": "Lyon, FR",
        "listing_type": "sale",
        "image_url": "https://example.org/invalid.jpg",
        "cycle_status": null,
        "is_available_for_club": false,
        "description": "Invalid enum payload"
    });
    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/v1/merchant/offers")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .header("Content-Type", "application/json")
            .body(Body::from(invalid_enum_offer_body.to_string()))
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let invalid_date_offer_body = json!({
        "name": "Invalid Date Offer",
        "animal_type": "dog",
        "breed": "N/A",
        "gender": "M",
        "birth_date": "2024-99-01",
        "price_eur": 100.0,
        "location": "Lyon, FR",
        "listing_type": "sale",
        "image_url": "https://example.org/invalid-date.jpg",
        "cycle_status": null,
        "is_available_for_club": false,
        "description": "Invalid date payload"
    });
    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/v1/merchant/offers")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .header("Content-Type", "application/json")
            .body(Body::from(invalid_date_offer_body.to_string()))
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["marketplaceUser"]["keycloak_username"],
        "merchant_demo"
    );

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/marketplace/offers")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["items"]
            .as_array()
            .expect("items should be array")
            .len()
            >= 1
    );

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/marketplace/offers/33333333-3333-4333-8333-333333333331")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/merchant/offers")
            .header("Authorization", format!("Bearer {}", client_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    sqlx::query(
        r#"
        INSERT INTO marketplace_users (id, keycloak_sub, keycloak_username, role, email, display_name)
        VALUES ($1::uuid, $2, $3, $4, $5, $6)
        ON CONFLICT (keycloak_username) DO UPDATE
        SET keycloak_sub = EXCLUDED.keycloak_sub,
            role = EXCLUDED.role,
            email = EXCLUDED.email,
            display_name = EXCLUDED.display_name
        "#,
    )
    .bind("99999999-9999-4999-8999-999999999999")
    .bind("sub-conflict-existing")
    .bind("conflict_identity")
    .bind("merchant")
    .bind("conflict.existing@pawtner.local")
    .bind("Conflict Existing")
    .execute(&db)
    .await
    .expect("should setup conflict identity row");

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/me/context")
            .header("Authorization", format!("Bearer {}", conflict_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "FORBIDDEN");

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/client/orders")
            .header("Authorization", "Bearer malformed-test-token")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/merchant/offers")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let new_offer_body = json!({
        "name": "Test Offer",
        "animal_type": "dog",
        "breed": "Golden Retriever",
        "gender": "M",
        "birth_date": "2024-01-01",
        "price_eur": 1200.0,
        "location": "Lyon, FR",
        "listing_type": "sale",
        "image_url": "https://example.org/image.jpg",
        "cycle_status": null,
        "is_available_for_club": false,
        "description": "Offer created by endpoint test"
    });
    let (status, _body) = send_json(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/v1/merchant/offers")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .header("Content-Type", "application/json")
            .body(Body::from(new_offer_body.to_string()))
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let patch_body = json!({ "status": "published", "price_eur": 1300.0 });
    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("PATCH")
            .uri("/api/v1/merchant/offers/33333333-3333-4333-8333-333333333331")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .header("Content-Type", "application/json")
            .body(Body::from(patch_body.to_string()))
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "published");

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("DELETE")
            .uri("/api/v1/merchant/offers/33333333-3333-4333-8333-333333333331")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "archived");

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/merchant/profile")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["reviews"].is_array());

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/merchant/orders")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/client/orders")
            .header("Authorization", format!("Bearer {}", merchant_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/client/orders")
            .header("Authorization", format!("Bearer {}", client_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/client/orders/55555555-5555-4555-8555-555555555551")
            .header("Authorization", format!("Bearer {}", client_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/client/orders/55555555-5555-4555-8555-555555555551")
            .header("Authorization", format!("Bearer {}", client_alice_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/marketplace/merchants/11111111-1111-4111-8111-111111111111/reviews")
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/v1/metrics/monthly-sales")
            .header("Authorization", format!("Bearer {}", client_token))
            .body(Body::empty())
            .expect("request should build"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["series"]
            .as_array()
            .expect("series should be an array")
            .len()
            >= 6
    );
}
