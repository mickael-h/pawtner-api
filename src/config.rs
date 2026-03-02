//! Configuration from environment (see docs/ARCHITECTURE.md).

use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub keycloak_issuer: String,
    pub keycloak_jwks_uri: String,
    pub keycloak_audience: String,
    pub bind: SocketAddr,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        let keycloak_issuer = std::env::var("KEYCLOAK_ISSUER")
            .map_err(|_| anyhow::anyhow!("KEYCLOAK_ISSUER must be set"))?;
        let keycloak_jwks_uri = std::env::var("KEYCLOAK_JWKS_URI").unwrap_or_else(|_| {
            format!(
                "{}/protocol/openid-connect/certs",
                keycloak_issuer.trim_end_matches('/')
            )
        });
        let keycloak_audience = std::env::var("KEYCLOAK_AUDIENCE")
            .map_err(|_| anyhow::anyhow!("KEYCLOAK_AUDIENCE must be set"))?;
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        let bind: SocketAddr = std::env::var("BIND_ADDR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| ([0, 0, 0, 0], port).into());

        Ok(Self {
            database_url,
            keycloak_issuer,
            keycloak_jwks_uri,
            keycloak_audience,
            bind,
        })
    }
}
