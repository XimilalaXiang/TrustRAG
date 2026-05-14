use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces/{ws_id}/members",
            get(list_members).post(add_member),
        )
        .route(
            "/workspaces/{ws_id}/members/{member_id}",
            put(update_member_role).delete(remove_member),
        )
}

#[derive(Serialize, sqlx::FromRow)]
struct MemberResponse {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
    username: String,
    role: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct AddMemberRequest {
    email: String,
    role: Option<String>,
}

#[derive(Deserialize)]
struct UpdateRoleRequest {
    role: String,
}

async fn check_owner_or_editor(
    pool: &sqlx::PgPool,
    ws_id: Uuid,
    user_id: Uuid,
) -> Result<String, AppError> {
    let role = sqlx::query_scalar::<_, String>(
        "SELECT wm.role FROM workspace_members wm
         WHERE wm.workspace_id = $1 AND wm.user_id = $2",
    )
    .bind(ws_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(ref r) = role {
        if r == "owner" || r == "editor" {
            return Ok(r.clone());
        }
    }

    let is_creator = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM workspaces WHERE id = $1 AND user_id = $2)",
    )
    .bind(ws_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if is_creator {
        return Ok("owner".to_string());
    }

    Err(AppError::Forbidden(
        "You do not have permission to manage this workspace".into(),
    ))
}

async fn list_members(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let members = sqlx::query_as::<_, MemberResponse>(
        "SELECT wm.id, wm.workspace_id, wm.user_id, u.username, wm.role, wm.created_at
         FROM workspace_members wm
         JOIN users u ON wm.user_id = u.id
         WHERE wm.workspace_id = $1
         ORDER BY wm.created_at",
    )
    .bind(ws_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(members))
}

async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), AppError> {
    check_owner_or_editor(&state.pool, ws_id, auth.id).await?;

    let target_user_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = $1",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User with email '{}' not found", req.email)))?;

    let role = req.role.unwrap_or_else(|| "viewer".to_string());
    if !["owner", "editor", "viewer"].contains(&role.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid role: {}", role)));
    }

    tracing::info!(
        workspace_id = %ws_id,
        target_user = %target_user_id,
        role = %role,
        invited_by = %auth.id,
        "Adding workspace member"
    );

    let member = sqlx::query_as::<_, MemberResponse>(
        "INSERT INTO workspace_members (workspace_id, user_id, role, invited_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (workspace_id, user_id) DO UPDATE SET role = $3
         RETURNING id, workspace_id, user_id,
                   (SELECT username FROM users WHERE id = workspace_members.user_id) as username,
                   role, created_at",
    )
    .bind(ws_id)
    .bind(target_user_id)
    .bind(&role)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(member)))
}

async fn update_member_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, member_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_owner_or_editor(&state.pool, ws_id, auth.id).await?;

    if !["owner", "editor", "viewer"].contains(&req.role.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid role: {}", req.role)));
    }

    let result = sqlx::query(
        "UPDATE workspace_members SET role = $1 WHERE id = $2 AND workspace_id = $3",
    )
    .bind(&req.role)
    .bind(member_id)
    .bind(ws_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".into()));
    }

    Ok(Json(serde_json::json!({"status": "updated"})))
}

async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    check_owner_or_editor(&state.pool, ws_id, auth.id).await?;

    let result = sqlx::query(
        "DELETE FROM workspace_members WHERE id = $1 AND workspace_id = $2",
    )
    .bind(member_id)
    .bind(ws_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
