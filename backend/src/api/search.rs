use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::services::search::{SearchConfig, SearchMode, SearchResponse};

use super::AppState;

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
    pub mode: Option<String>,
    pub document_ids: Option<Vec<Uuid>>,
    pub min_score: Option<f64>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/workspaces/{ws_id}/search", post(search))
}

async fn search(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, AppError> {
    if req.query.trim().is_empty() {
        return Err(AppError::BadRequest("Query must not be empty".into()));
    }

    let has_access = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM workspaces
            WHERE id = $1
              AND (owner_id = $2
                   OR id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $2)
                   OR visibility = 'public')
        )
        "#,
    )
    .bind(ws_id)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    if !has_access {
        return Err(AppError::NotFound("Workspace not found".into()));
    }

    let mode = match req.mode.as_deref() {
        Some("vector") => SearchMode::Vector,
        Some("fulltext") => SearchMode::Fulltext,
        _ => SearchMode::Hybrid,
    };

    let config = SearchConfig {
        mode,
        top_k: req.top_k.unwrap_or(10).min(100),
        min_score: req.min_score.unwrap_or(0.3),
        ..Default::default()
    };

    let doc_ids = req.document_ids.as_deref();

    let embedding_guard = state.embedding_provider.read().await;
    let embedding_provider = embedding_guard
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("No embedding provider configured".into()))?;

    let response = crate::services::search::hybrid_search(
        &state.pool,
        embedding_provider.as_ref(),
        ws_id,
        &req.query,
        &config,
        doc_ids,
    )
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Search failed: {e}")))?;

    Ok(Json(response))
}
