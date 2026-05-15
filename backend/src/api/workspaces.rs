use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::db::compat;
use crate::error::AppError;

use super::AppState;

#[derive(Serialize)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
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
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String, String, String, String)>(
        r#"
        SELECT w.id, w.name, w.description, w.owner_id, w.visibility,
               CAST(w.created_at AS TEXT) as created_at, CAST(w.updated_at AS TEXT) as updated_at
        FROM workspaces w
        WHERE w.owner_id = $1
           OR w.id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $1)
        ORDER BY w.updated_at DESC
        "#,
    )
    .bind(auth.id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let mut workspaces = Vec::with_capacity(rows.len());
    for r in rows {
        workspaces.push(WorkspaceResponse {
            id: compat::parse_uuid(&r.0).map_err(|e| AppError::Internal(e.into()))?,
            name: r.1,
            description: r.2,
            owner_id: compat::parse_uuid(&r.3).map_err(|e| AppError::Internal(e.into()))?,
            visibility: r.4,
            created_at: r.5,
            updated_at: r.6,
        });
    }

    Ok(Json(workspaces))
}

async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<(axum::http::StatusCode, Json<WorkspaceResponse>), AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required".into()));
    }

    let visibility = req.visibility.unwrap_or_else(|| "private".to_string());
    if !matches!(visibility.as_str(), "private" | "public") {
        return Err(AppError::BadRequest("Visibility must be 'private' or 'public'".into()));
    }

    let r = sqlx::query_as::<_, (String, String, Option<String>, String, String, String, String)>(
        "INSERT INTO workspaces (name, description, owner_id, visibility) VALUES ($1, $2, $3, $4) RETURNING id, name, description, owner_id, visibility, CAST(created_at AS TEXT), CAST(updated_at AS TEXT)",
    )
    .bind(&name)
    .bind(&req.description)
    .bind(auth.id.to_string())
    .bind(&visibility)
    .fetch_one(&state.pool)
    .await?;

    let ws = WorkspaceResponse {
        id: compat::parse_uuid(&r.0).map_err(|e| AppError::Internal(e.into()))?,
        name: r.1,
        description: r.2,
        owner_id: compat::parse_uuid(&r.3).map_err(|e| AppError::Internal(e.into()))?,
        visibility: r.4,
        created_at: r.5,
        updated_at: r.6,
    };

    Ok((axum::http::StatusCode::CREATED, Json(ws)))
}

async fn get_one(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let r = sqlx::query_as::<_, (String, String, Option<String>, String, String, String, String)>(
        r#"
        SELECT w.id, w.name, w.description, w.owner_id, w.visibility,
               CAST(w.created_at AS TEXT), CAST(w.updated_at AS TEXT)
        FROM workspaces w
        WHERE w.id = $1
          AND (w.owner_id = $2
               OR w.id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $2)
               OR w.visibility = 'public')
        "#,
    )
    .bind(id.to_string())
    .bind(auth.id.to_string())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Workspace not found".into()))?;

    Ok(Json(WorkspaceResponse {
        id: compat::parse_uuid(&r.0).map_err(|e| AppError::Internal(e.into()))?,
        name: r.1,
        description: r.2,
        owner_id: compat::parse_uuid(&r.3).map_err(|e| AppError::Internal(e.into()))?,
        visibility: r.4,
        created_at: r.5,
        updated_at: r.6,
    }))
}

async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, AppError> {
    let existing = sqlx::query_as::<_, (String, Option<String>, String)>(
        "SELECT name, description, visibility FROM workspaces WHERE id = $1 AND owner_id = $2",
    )
    .bind(id.to_string())
    .bind(auth.id.to_string())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Workspace not found or not owned by you".into()))?;

    let name = req.name.map(|n| n.trim().to_string()).unwrap_or(existing.0);
    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".into()));
    }
    let description = req.description.or(existing.1);
    let visibility = req.visibility.unwrap_or(existing.2);
    if !matches!(visibility.as_str(), "private" | "public") {
        return Err(AppError::BadRequest("Visibility must be 'private' or 'public'".into()));
    }

    let r = sqlx::query_as::<_, (String, String, Option<String>, String, String, String, String)>(
        "UPDATE workspaces SET name = $1, description = $2, visibility = $3 WHERE id = $4 RETURNING id, name, description, owner_id, visibility, CAST(created_at AS TEXT), CAST(updated_at AS TEXT)",
    )
    .bind(&name)
    .bind(&description)
    .bind(&visibility)
    .bind(id.to_string())
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(WorkspaceResponse {
        id: compat::parse_uuid(&r.0).map_err(|e| AppError::Internal(e.into()))?,
        name: r.1,
        description: r.2,
        owner_id: compat::parse_uuid(&r.3).map_err(|e| AppError::Internal(e.into()))?,
        visibility: r.4,
        created_at: r.5,
        updated_at: r.6,
    }))
}

async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM workspaces WHERE id = $1 AND owner_id = $2")
        .bind(id.to_string())
        .bind(auth.id.to_string())
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Workspace not found or not owned by you".into(),
        ));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
