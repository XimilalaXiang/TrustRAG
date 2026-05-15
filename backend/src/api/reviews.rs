use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::services::review::{self, CreateReviewInput, ReviewRecord, ReviewStats};

pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route(
            "/citations/{citation_id}/reviews",
            get(list_reviews).post(create_review),
        )
        .route(
            "/conversations/{conv_id}/review-stats",
            get(get_conversation_review_stats),
        )
}

async fn create_review(
    auth: AuthUser,
    State(state): State<crate::api::AppState>,
    Path(citation_id): Path<Uuid>,
    Json(input): Json<CreateReviewInput>,
) -> Result<Json<ReviewRecord>, AppError> {
    tracing::info!(
        citation_id = %citation_id,
        reviewer_id = %auth.id,
        status = %input.status,
        "Creating citation review"
    );
    let record = match review::create_review(&state.pool, citation_id, auth.id, &input).await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Review creation failed for citation {}: {}", citation_id, e);
            tracing::error!("{}", err_msg);
            return Err(AppError::BadRequest(err_msg));
        }
    };
    Ok(Json(record))
}

async fn list_reviews(
    _auth: AuthUser,
    State(state): State<crate::api::AppState>,
    Path(citation_id): Path<Uuid>,
) -> Result<Json<Vec<ReviewRecord>>, AppError> {
    let reviews = review::list_reviews_for_citation(&state.pool, citation_id).await?;
    Ok(Json(reviews))
}

async fn get_conversation_review_stats(
    _auth: AuthUser,
    State(state): State<crate::api::AppState>,
    Path(conv_id): Path<Uuid>,
) -> Result<Json<ReviewStats>, AppError> {
    let stats = review::get_review_stats_for_conversation(&state.pool, conv_id).await?;
    Ok(Json(stats))
}
