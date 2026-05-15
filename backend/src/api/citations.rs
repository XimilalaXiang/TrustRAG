use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/messages/{message_id}/citations",
            get(list_citations),
        )
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CitationResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub document_id: Uuid,
    pub chunk_id: Uuid,
    pub citation_index: i16,
    pub quoted_text: Option<String>,
    pub page_number: Option<i32>,
    pub heading_path: Option<String>,
    pub relevance_score: Option<f32>,
    pub verified: Option<bool>,
    pub created_at: DateTime<Utc>,
}

async fn list_citations(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<CitationResponse>>, AppError> {
    let citations = sqlx::query_as::<_, CitationResponse>(
        "SELECT id, message_id, document_id, chunk_id, citation_index, quoted_text,
                page_number, heading_path, relevance_score, verified, created_at
         FROM citations
         WHERE message_id = $1
         ORDER BY citation_index ASC",
    )
    .bind(message_id.to_string())
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(citations))
}
