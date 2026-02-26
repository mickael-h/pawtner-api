//! Application state (DB pool, JWT validator).

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;
use crate::middleware::JwtValidator;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_validator: Arc<JwtValidator>,
    /// Keycloak UserInfo URL for the pawtner realm.
    pub keycloak_userinfo_uri: String,
}

impl AppState {
    pub async fn from_config(config: &Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;
        let jwt_validator = Arc::new(JwtValidator::new(
            config.keycloak_jwks_uri.clone(),
            config.keycloak_issuer.clone(),
            config.keycloak_audience.clone(),
        ));
        let keycloak_userinfo_uri = format!(
            "{}/protocol/openid-connect/userinfo",
            config.keycloak_issuer.trim_end_matches('/')
        );
        Ok(Self {
            db,
            jwt_validator,
            keycloak_userinfo_uri,
        })
    }
}
