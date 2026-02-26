//! JWT auth: validate Bearer token using Keycloak JWKS.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, header::AUTHORIZATION},
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::ApiError;
use crate::state::AppState;

/// Keycloak JWT claims (minimal: sub = user id).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakClaims {
    pub sub: String,
    pub exp: i64,
    #[serde(default)]
    pub iss: Option<String>,
}

/// Minimal JWKS structure for Keycloak (RSA keys with n, e).
#[derive(Debug, Clone, Deserialize)]
struct JwkSet {
    keys: Vec<RsaJwk>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct RsaJwk {
    kid: Option<String>,
    kty: String,
    n: String,
    e: String,
}

impl JwkSet {
    fn find(&self, kid: &str) -> Option<&RsaJwk> {
        self.keys.iter().find(|k| k.kid.as_deref() == Some(kid))
    }
}

async fn fetch_jwks(uri: &str) -> anyhow::Result<JwkSet> {
    let res = reqwest::get(uri).await?;
    let jwks: JwkSet = res.json().await?;
    Ok(jwks)
}

/// Validates JWT using Keycloak JWKS (cached).
pub struct JwtValidator {
    jwks_uri: String,
    cache: RwLock<Option<JwkSet>>,
    issuer: String,
    audience: String,
}

impl JwtValidator {
    pub fn new(jwks_uri: String, issuer: String, audience: String) -> Self {
        Self {
            jwks_uri,
            cache: RwLock::new(None),
            issuer,
            audience,
        }
    }

    async fn get_jwks(&self) -> anyhow::Result<JwkSet> {
        {
            let guard = self.cache.read().await;
            if let Some(ref jwks) = *guard {
                return Ok(jwks.clone());
            }
        }
        let jwks = fetch_jwks(&self.jwks_uri).await?;
        *self.cache.write().await = Some(jwks.clone());
        Ok(jwks)
    }

    pub async fn validate(&self, token: &str) -> Result<TokenData<KeycloakClaims>, ApiError> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| ApiError::Unauthorized(format!("invalid token header: {}", e)))?;
        let kid = header
            .kid
            .ok_or_else(|| ApiError::Unauthorized("token missing kid".to_string()))?;
        let jwks = self.get_jwks().await.map_err(|e| {
            tracing::warn!("jwks fetch failed: {:?}", e);
            ApiError::Unauthorized("auth server unavailable".to_string())
        })?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| ApiError::Unauthorized("unknown key id".to_string()))?;
        let _ = &jwk.kty;
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| ApiError::Unauthorized(format!("key error: {}", e)))?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.trim_end_matches('/')]);
        validation.set_audience(&[self.audience.as_str()]);
        validation.set_required_spec_claims(&["exp", "sub"]);
        let token_data = decode::<KeycloakClaims>(token, &decoding_key, &validation)
            .map_err(|e| ApiError::Unauthorized(format!("invalid token: {}", e)))?;
        // Touch claims fields so they are considered used (they are logically important
        // even if not yet used in handlers).
        let _ = (&token_data.claims.sub, &token_data.claims.exp, &token_data.claims.iss);
        Ok(token_data)
    }
}

/// Extractor: require valid Bearer JWT and yield Keycloak claims.
pub struct AuthUser(pub KeycloakClaims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing Authorization header".to_string()))?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::Unauthorized("expected Bearer token".to_string()))?;
        let token_data = app_state.jwt_validator.validate(token).await?;
        Ok(AuthUser(token_data.claims))
    }
}

/// Trait to get AppState from router state (axum 0.7).
pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for AppState {
    fn from_ref(input: &AppState) -> Self {
        input.clone()
    }
}

/// Extractor: raw Bearer token string (for forwarding to Keycloak UserInfo etc.).
pub struct RawBearerToken(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for RawBearerToken
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing Authorization header".to_string()))?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::Unauthorized("expected Bearer token".to_string()))?
            .to_string();
        Ok(RawBearerToken(token))
    }
}
