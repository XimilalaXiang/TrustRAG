use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, delete},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::services::embedding::OpenAIEmbeddingProvider;
use crate::traits::embedding_provider::EmbeddingProvider;

use super::AppState;
use super::models::{decrypt_api_key, encrypt_api_key};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/embedding-configs", get(list_configs).post(create_config))
        .route(
            "/embedding-configs/{id}",
            put(update_config).delete(delete_config),
        )
        .route("/embedding-configs/{id}/test", post(test_connection))
}

#[derive(Deserialize)]
pub struct CreateEmbeddingConfigRequest {
    pub name: String,
    pub provider: String,
    pub api_base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    pub model_name: String,
    #[serde(default = "default_dimensions")]
    pub dimensions: i32,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

fn default_dimensions() -> i32 {
    1536
}

#[derive(Deserialize)]
pub struct UpdateEmbeddingConfigRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub dimensions: Option<i32>,
    pub is_default: Option<bool>,
}

#[derive(Serialize)]
pub struct EmbeddingConfigResponse {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    pub api_base_url: Option<String>,
    pub has_api_key: bool,
    pub model_name: String,
    pub dimensions: i32,
    pub is_default: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct TestEmbeddingResponse {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

async fn list_configs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<EmbeddingConfigResponse>>, AppError> {
    let rows = sqlx::query_as::<_, (
        Uuid, Option<Uuid>, Uuid, String, String, Option<String>, Option<String>,
        String, i32, Option<bool>, DateTime<Utc>, DateTime<Utc>,
    )>(
        "SELECT id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                model_name, dimensions, is_default, created_at, updated_at
         FROM embedding_configs
         WHERE user_id = $1
         ORDER BY is_default DESC NULLS LAST, created_at DESC",
    )
    .bind(auth.id)
    .fetch_all(&state.pool)
    .await?;

    let configs: Vec<EmbeddingConfigResponse> = rows
        .into_iter()
        .map(|r| EmbeddingConfigResponse {
            id: r.0,
            workspace_id: r.1,
            user_id: r.2,
            name: r.3,
            provider: r.4,
            api_base_url: r.5,
            has_api_key: r.6.is_some(),
            model_name: r.7,
            dimensions: r.8,
            is_default: r.9,
            created_at: r.10,
            updated_at: r.11,
        })
        .collect();

    Ok(Json(configs))
}

async fn create_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateEmbeddingConfigRequest>,
) -> Result<(StatusCode, Json<EmbeddingConfigResponse>), AppError> {
    let valid_providers = ["openai", "local", "custom"];
    if !valid_providers.contains(&req.provider.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid provider '{}'. Must be one of: {}",
            req.provider,
            valid_providers.join(", ")
        )));
    }

    let api_key_enc = req
        .api_key
        .as_deref()
        .map(|k| encrypt_api_key(k, &state.jwt_secret));

    if req.is_default {
        sqlx::query("UPDATE embedding_configs SET is_default = false WHERE user_id = $1")
            .bind(auth.id)
            .execute(&state.pool)
            .await?;
    }

    let row = sqlx::query_as::<_, (
        Uuid, Option<Uuid>, Uuid, String, String, Option<String>, Option<String>,
        String, i32, Option<bool>, DateTime<Utc>, DateTime<Utc>,
    )>(
        "INSERT INTO embedding_configs
            (workspace_id, user_id, name, provider, api_base_url, api_key_enc,
             model_name, dimensions, is_default)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                   model_name, dimensions, is_default, created_at, updated_at",
    )
    .bind(req.workspace_id)
    .bind(auth.id)
    .bind(&req.name)
    .bind(&req.provider)
    .bind(&req.api_base_url)
    .bind(&api_key_enc)
    .bind(&req.model_name)
    .bind(req.dimensions)
    .bind(req.is_default)
    .fetch_one(&state.pool)
    .await?;

    let resp = EmbeddingConfigResponse {
        id: row.0,
        workspace_id: row.1,
        user_id: row.2,
        name: row.3,
        provider: row.4,
        api_base_url: row.5,
        has_api_key: row.6.is_some(),
        model_name: row.7,
        dimensions: row.8,
        is_default: row.9,
        created_at: row.10,
        updated_at: row.11,
    };

    if req.is_default {
        reload_embedding_provider(&state).await;
    }

    Ok((StatusCode::CREATED, Json(resp)))
}

async fn update_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
    Json(req): Json<UpdateEmbeddingConfigRequest>,
) -> Result<Json<EmbeddingConfigResponse>, AppError> {
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM embedding_configs WHERE id = $1 AND user_id = $2",
    )
    .bind(config_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_none() {
        return Err(AppError::NotFound("Embedding config not found".into()));
    }

    if let Some(provider) = &req.provider {
        let valid = ["openai", "local", "custom"];
        if !valid.contains(&provider.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid provider '{}'",
                provider
            )));
        }
    }

    let api_key_enc = req
        .api_key
        .as_deref()
        .map(|k| encrypt_api_key(k, &state.jwt_secret));

    if req.is_default == Some(true) {
        sqlx::query(
            "UPDATE embedding_configs SET is_default = false WHERE user_id = $1 AND id != $2",
        )
        .bind(auth.id)
        .bind(config_id)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, (
        Uuid, Option<Uuid>, Uuid, String, String, Option<String>, Option<String>,
        String, i32, Option<bool>, DateTime<Utc>, DateTime<Utc>,
    )>(
        "UPDATE embedding_configs SET
            name = COALESCE($1, name),
            provider = COALESCE($2, provider),
            api_base_url = COALESCE($3, api_base_url),
            api_key_enc = COALESCE($4, api_key_enc),
            model_name = COALESCE($5, model_name),
            dimensions = COALESCE($6, dimensions),
            is_default = COALESCE($7, is_default)
         WHERE id = $8 AND user_id = $9
         RETURNING id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                   model_name, dimensions, is_default, created_at, updated_at",
    )
    .bind(req.name)
    .bind(req.provider)
    .bind(req.api_base_url)
    .bind(api_key_enc)
    .bind(req.model_name)
    .bind(req.dimensions)
    .bind(req.is_default)
    .bind(config_id)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    let resp = EmbeddingConfigResponse {
        id: row.0,
        workspace_id: row.1,
        user_id: row.2,
        name: row.3,
        provider: row.4,
        api_base_url: row.5,
        has_api_key: row.6.is_some(),
        model_name: row.7,
        dimensions: row.8,
        is_default: row.9,
        created_at: row.10,
        updated_at: row.11,
    };

    reload_embedding_provider(&state).await;

    Ok(Json(resp))
}

async fn delete_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let was_default = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_default, false) FROM embedding_configs WHERE id = $1 AND user_id = $2",
    )
    .bind(config_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?;

    let result = sqlx::query("DELETE FROM embedding_configs WHERE id = $1 AND user_id = $2")
        .bind(config_id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Embedding config not found".into()));
    }

    if was_default == Some(true) {
        reload_embedding_provider(&state).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn test_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
) -> Result<Json<TestEmbeddingResponse>, AppError> {
    let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, String, i32)>(
        "SELECT provider, api_base_url, api_key_enc, model_name, dimensions
         FROM embedding_configs
         WHERE id = $1 AND user_id = $2",
    )
    .bind(config_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?;

    let (provider, api_base_url, api_key_enc, model_name, dimensions) = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Embedding config not found".into())),
    };

    let api_key = api_key_enc.and_then(|enc| decrypt_api_key(&enc, &state.jwt_secret));
    let base_url = api_base_url.unwrap_or_default();

    let start = std::time::Instant::now();

    let emb_provider = OpenAIEmbeddingProvider::new(
        &base_url,
        api_key.as_deref(),
        &model_name,
        dimensions as usize,
    );

    match emb_provider.embed_texts(&["Hello, this is a test.".to_string()]).await {
        Ok(embeddings) => {
            let latency = start.elapsed().as_millis() as u64;
            if let Some(emb) = embeddings.first() {
                Ok(Json(TestEmbeddingResponse {
                    success: true,
                    message: format!(
                        "Connected to {} model '{}'. Embedding dimension: {} (expected: {})",
                        provider, model_name, emb.len(), dimensions
                    ),
                    latency_ms: Some(latency),
                }))
            } else {
                Ok(Json(TestEmbeddingResponse {
                    success: false,
                    message: "API returned empty embeddings".into(),
                    latency_ms: Some(latency),
                }))
            }
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            Ok(Json(TestEmbeddingResponse {
                success: false,
                message: format!("Connection failed: {}", e),
                latency_ms: Some(latency),
            }))
        }
    }
}

async fn reload_embedding_provider(state: &AppState) {
    let row = sqlx::query_as::<_, (Option<String>, Option<String>, String, i32)>(
        "SELECT api_base_url, api_key_enc, model_name, dimensions
         FROM embedding_configs
         WHERE is_default = true
         ORDER BY updated_at DESC
         LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(Some((api_base_url, api_key_enc, model_name, dimensions))) => {
            let api_key = api_key_enc.and_then(|enc| decrypt_api_key(&enc, &state.jwt_secret));
            let base_url = api_base_url.unwrap_or_default();
            let provider = Arc::new(OpenAIEmbeddingProvider::new(
                &base_url,
                api_key.as_deref(),
                &model_name,
                dimensions as usize,
            )) as Arc<dyn EmbeddingProvider>;

            let mut guard = state.embedding_provider.write().await;
            *guard = Some(provider);
            tracing::info!(model = %model_name, dimensions, "Embedding provider reloaded");
        }
        Ok(None) => {
            let mut guard = state.embedding_provider.write().await;
            *guard = None;
            tracing::info!("No default embedding config found, embedding provider cleared");
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to reload embedding provider");
        }
    }
}

pub async fn init_embedding_provider(state: &AppState) {
    reload_embedding_provider(state).await;
}
