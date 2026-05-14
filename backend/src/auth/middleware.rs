use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::auth::jwt::verify_token;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jwt_secret = parts
            .extensions
            .get::<JwtSecret>()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("JWT secret not configured")))?;

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Auth("Invalid authorization format".into()))?;

        let claims = verify_token(token, &jwt_secret.0)
            .map_err(|e| AppError::Auth(format!("Invalid token: {e}")))?;

        Ok(AuthUser {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
        })
    }
}

#[derive(Clone)]
pub struct JwtSecret(pub String);
