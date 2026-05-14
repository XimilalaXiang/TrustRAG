use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;

use super::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workspaces", get(list).post(create))
        .route(
            "/workspaces/{id}",
            get(get_one).put(update).delete(remove),
        )
}

async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    let workspaces = sqlx::query_as::<_, WorkspaceResponse>(
        r#"
        SELECT w.* FROM workspaces w
        WHERE w.owner_id = $1
           OR w.id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $1)
        ORDER BY w.updated_at DESC
        "#,
    )
    .bind(auth.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(workspaces))
}

async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<(axum::http::StatusCode, Json<WorkspaceResponse>), AppError> {
    if req.name.is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }

    let visibility = req.visibility.unwrap_or_else(|| "private".to_string());

    let ws = sqlx::query_as::<_, WorkspaceResponse>(
        "INSERT INTO workspaces (name, description, owner_id, visibility) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(auth.id)
    .bind(&visibility)
    .fetch_one(&state.pool)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(ws)))
}

async fn get_one(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = sqlx::query_as::<_, WorkspaceResponse>(
        r#"
        SELECT w.* FROM workspaces w
        WHERE w.id = $1
          AND (w.owner_id = $2
               OR w.id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $2)
               OR w.visibility = 'public')
        "#,
    )
    .bind(id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Workspace not found".into()))?;

    Ok(Json(ws))
}

async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let ws = sqlx::query_as::<_, WorkspaceResponse>(
        "SELECT * FROM workspaces WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Workspace not found or not owned by you".into()))?;

    let name = req.name.unwrap_or(ws.name);
    let description = req.description.or(ws.description);
    let visibility = req.visibility.unwrap_or(ws.visibility);

    let updated = sqlx::query_as::<_, WorkspaceResponse>(
        "UPDATE workspaces SET name = $1, description = $2, visibility = $3 WHERE id = $4 RETURNING *",
    )
    .bind(&name)
    .bind(&description)
    .bind(&visibility)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}

async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM workspaces WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Workspace not found or not owned by you".into(),
        ));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
