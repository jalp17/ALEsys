use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    IngestionRead,
    IngestionWrite,
}

pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: u64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                "alesys-dev-secret-change-in-production".to_string()
            }),
            expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }
}

pub fn create_token(user_id: &str, role: Role, config: &JwtConfig) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let expiry = now + (config.expiry_hours * 3600) as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        role,
        exp: expiry,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => return Err(StatusCode::UNAUTHORIZED),
        };

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            "alesys-dev-secret-change-in-production".to_string()
        });

        verify_token(token, &secret).map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    match verify_token(token, &state.jwt_config.secret) {
        Ok(_claims) => Ok(next.run(request).await),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub struct AuthState {
    pub jwt_config: JwtConfig,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            jwt_config: JwtConfig::from_env(),
        }
    }
}

pub fn has_permission(role: &Role, permission: Permission) -> bool {
    match role {
        Role::Admin => true,
        Role::User => matches!(permission, Permission::IngestionRead | Permission::IngestionWrite),
        Role::Agent => matches!(permission, Permission::IngestionRead),
    }
}
