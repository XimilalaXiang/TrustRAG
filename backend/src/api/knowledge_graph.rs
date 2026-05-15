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
        .route("/workspaces/{ws_id}/knowledge-graph", get(get_graph))
        .route(
            "/workspaces/{ws_id}/knowledge-graph/entities",
            get(list_entities),
        )
}

#[derive(Serialize)]
struct EntityRow {
    id: Uuid,
    name: String,
    entity_type: String,
    document_id: Option<Uuid>,
    metadata: serde_json::Value,
    created_at: String,
}

struct RelationRow {
    id: Uuid,
    source_entity_id: Uuid,
    target_entity_id: Uuid,
    relation_type: String,
    weight: f64,
}

#[derive(Serialize)]
struct GraphResponse {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    label: String,
    entity_type: String,
    document_id: Option<Uuid>,
}

#[derive(Serialize)]
struct GraphEdge {
    source: String,
    target: String,
    relation: String,
    weight: f64,
}

async fn check_workspace_access(
    pool: &crate::db::DbPool,
    ws_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let access_count: i32 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM workspaces
        WHERE id = $1
          AND (owner_id = $2
               OR id IN (SELECT workspace_id FROM workspace_members WHERE user_id = $2)
               OR visibility = 'public')
        "#,
    )
    .bind(ws_id.to_string())
    .bind(user_id.to_string())
    .fetch_one(pool)
    .await?;

    if access_count == 0 {
        return Err(AppError::NotFound("Workspace not found".into()));
    }
    Ok(())
}

async fn get_graph(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<GraphResponse>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;
    tracing::info!(workspace_id = %ws_id, "Fetching knowledge graph");

    let entity_rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String)>(
        "SELECT id, name, entity_type, document_id, CAST(metadata AS TEXT), CAST(created_at AS TEXT)
         FROM entities WHERE workspace_id = $1 ORDER BY name",
    )
    .bind(ws_id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let entities: Vec<EntityRow> = entity_rows.into_iter().map(|r| {
        let metadata: serde_json::Value = r.4.as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::json!({}));
        EntityRow {
            id: r.0.parse().unwrap_or_default(),
            name: r.1,
            entity_type: r.2,
            document_id: r.3.as_deref().and_then(|s| s.parse().ok()),
            metadata,
            created_at: r.5,
        }
    }).collect();

    let relation_rows = sqlx::query_as::<_, (String, String, String, String, f64)>(
        "SELECT id, source_entity_id, target_entity_id, relation_type, weight
         FROM entity_relations WHERE workspace_id = $1",
    )
    .bind(ws_id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let relations: Vec<RelationRow> = relation_rows.into_iter().map(|r| {
        RelationRow {
            id: r.0.parse().unwrap_or_default(),
            source_entity_id: r.1.parse().unwrap_or_default(),
            target_entity_id: r.2.parse().unwrap_or_default(),
            relation_type: r.3,
            weight: r.4,
        }
    }).collect();

    let nodes: Vec<GraphNode> = entities
        .iter()
        .map(|e| GraphNode {
            id: e.id.to_string(),
            label: e.name.clone(),
            entity_type: e.entity_type.clone(),
            document_id: e.document_id,
        })
        .collect();

    let edges: Vec<GraphEdge> = relations
        .iter()
        .map(|r| GraphEdge {
            source: r.source_entity_id.to_string(),
            target: r.target_entity_id.to_string(),
            relation: r.relation_type.clone(),
            weight: r.weight,
        })
        .collect();

    Ok(Json(GraphResponse { nodes, edges }))
}

async fn list_entities(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Vec<EntityRow>>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String)>(
        "SELECT id, name, entity_type, document_id, CAST(metadata AS TEXT), CAST(created_at AS TEXT)
         FROM entities WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 200",
    )
    .bind(ws_id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let entities: Vec<EntityRow> = rows.into_iter().map(|r| {
        let metadata: serde_json::Value = r.4.as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::json!({}));
        EntityRow {
            id: r.0.parse().unwrap_or_default(),
            name: r.1,
            entity_type: r.2,
            document_id: r.3.as_deref().and_then(|s| s.parse().ok()),
            metadata,
            created_at: r.5,
        }
    }).collect();

    Ok(Json(entities))
}
