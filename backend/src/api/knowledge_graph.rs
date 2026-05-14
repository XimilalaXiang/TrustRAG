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

#[derive(Serialize, sqlx::FromRow)]
struct EntityRow {
    id: Uuid,
    name: String,
    entity_type: String,
    document_id: Option<Uuid>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
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

async fn get_graph(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<GraphResponse>, AppError> {
    tracing::info!(workspace_id = %ws_id, "Fetching knowledge graph");

    let entities = sqlx::query_as::<_, EntityRow>(
        "SELECT id, name, entity_type, document_id, metadata, created_at
         FROM entities WHERE workspace_id = $1 ORDER BY name",
    )
    .bind(ws_id)
    .fetch_all(&state.pool)
    .await?;

    let relations = sqlx::query_as::<_, RelationRow>(
        "SELECT id, source_entity_id, target_entity_id, relation_type, weight
         FROM entity_relations WHERE workspace_id = $1",
    )
    .bind(ws_id)
    .fetch_all(&state.pool)
    .await?;

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
    _auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Vec<EntityRow>>, AppError> {
    let entities = sqlx::query_as::<_, EntityRow>(
        "SELECT id, name, entity_type, document_id, metadata, created_at
         FROM entities WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 200",
    )
    .bind(ws_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(entities))
}
