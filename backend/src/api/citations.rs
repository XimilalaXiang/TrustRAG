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

#[derive(Serialize)]
pub struct CitationResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub document_id: Uuid,
    pub chunk_id: Uuid,
    pub citation_index: i32,
    pub quoted_text: Option<String>,
    pub page_number: Option<i32>,
    pub heading_path: Option<String>,
    pub relevance_score: Option<f64>,
    pub verified: Option<i32>,
    pub created_at: String,
}

type CitationRow = (String, String, String, String, i32, Option<String>, Option<i32>, Option<String>, Option<f64>, Option<i32>, String);

fn parse_citation_row(r: CitationRow) -> CitationResponse {
    CitationResponse {
        id: r.0.parse().unwrap_or_default(),
        message_id: r.1.parse().unwrap_or_default(),
        document_id: r.2.parse().unwrap_or_default(),
        chunk_id: r.3.parse().unwrap_or_default(),
        citation_index: r.4,
        quoted_text: r.5,
        page_number: r.6,
        heading_path: r.7,
        relevance_score: r.8,
        verified: r.9,
        created_at: r.10,
    }
}

async fn list_citations(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<CitationResponse>>, AppError> {
    let rows = sqlx::query_as::<_, CitationRow>(
        "SELECT id, message_id, document_id, chunk_id, citation_index, quoted_text,
                page_number, heading_path, relevance_score, CASE WHEN verified THEN 1 ELSE 0 END, CAST(created_at AS TEXT)
         FROM citations
         WHERE message_id = $1
         ORDER BY citation_index ASC",
    )
    .bind(message_id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let citations: Vec<CitationResponse> = rows.into_iter().map(parse_citation_row).collect();
    Ok(Json(citations))
}
