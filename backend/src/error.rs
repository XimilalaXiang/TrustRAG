use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            AppError::Auth(_) => "AUTH_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Database(e) => {
                tracing::error!(
                    error.kind = "database",
                    error.detail = %e,
                    "Database error occurred"
                );
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            AppError::Internal(e) => {
                tracing::error!(
                    error.kind = "internal",
                    error.detail = %e,
                    error.chain = ?e.chain().skip(1).collect::<Vec<_>>(),
                    "Internal error occurred"
                );
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into())
            }
        };

        let error_code = self.error_code();
        tracing::warn!(
            http.status = status.as_u16(),
            error.code = error_code,
            error.message = %message,
            "Request error response"
        );

        let body = serde_json::json!({
            "error": message,
            "code": error_code,
        });
        (status, axum::Json(body)).into_response()
    }
}
