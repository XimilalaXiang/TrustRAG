use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put, delete},
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
        .route("/model-configs", get(list_configs).post(create_config))
        .route(
            "/model-configs/{id}",
            put(update_config).delete(delete_config),
        )
        .route("/model-configs/{id}/test", post(test_connection))
}

// ── Request / Response types ──

#[derive(Deserialize)]
pub struct CreateConfigRequest {
    pub name: String,
    pub provider: String,
    pub api_base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    pub model_name: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

fn default_temperature() -> f32 {
    0.1
}
fn default_max_tokens() -> i32 {
    4096
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_default: Option<bool>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ModelConfigResponse {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    pub api_base_url: String,
    pub has_api_key: bool,
    pub model_name: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_default: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

// ── Simple XOR-based obfuscation for API keys ──
// Production systems should use AES-256-GCM with a proper KMS.
fn encrypt_api_key(key: &str, secret: &str) -> String {
    let secret_bytes = secret.as_bytes();
    let encrypted: Vec<u8> = key
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ secret_bytes[i % secret_bytes.len()])
        .collect();
    hex::encode(encrypted)
}

pub fn decrypt_api_key(enc_hex: &str, secret: &str) -> Option<String> {
    let encrypted = hex::decode(enc_hex).ok()?;
    let secret_bytes = secret.as_bytes();
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ secret_bytes[i % secret_bytes.len()])
        .collect();
    String::from_utf8(decrypted).ok()
}

// ── Handlers ──

async fn list_configs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ModelConfigResponse>>, AppError> {
    let rows = sqlx::query_as::<_, (
        Uuid,
        Option<Uuid>,
        Uuid,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<f32>,
        Option<i32>,
        Option<bool>,
        DateTime<Utc>,
        DateTime<Utc>,
    )>(
        "SELECT id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                model_name, temperature, max_tokens, is_default, created_at, updated_at
         FROM model_configs
         WHERE user_id = $1
         ORDER BY is_default DESC NULLS LAST, created_at DESC",
    )
    .bind(auth.id)
    .fetch_all(&state.pool)
    .await?;

    let configs: Vec<ModelConfigResponse> = rows
        .into_iter()
        .map(|r| ModelConfigResponse {
            id: r.0,
            workspace_id: r.1,
            user_id: r.2,
            name: r.3,
            provider: r.4,
            api_base_url: r.5,
            has_api_key: r.6.is_some(),
            model_name: r.7,
            temperature: r.8,
            max_tokens: r.9,
            is_default: r.10,
            created_at: r.11,
            updated_at: r.12,
        })
        .collect();

    Ok(Json(configs))
}

async fn create_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateConfigRequest>,
) -> Result<(StatusCode, Json<ModelConfigResponse>), AppError> {
    let valid_providers = ["openai", "anthropic", "ollama", "custom"];
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

    // If is_default, clear other defaults for same user
    if req.is_default {
        sqlx::query("UPDATE model_configs SET is_default = false WHERE user_id = $1")
            .bind(auth.id)
            .execute(&state.pool)
            .await?;
    }

    let row = sqlx::query_as::<_, (
        Uuid,
        Option<Uuid>,
        Uuid,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<f32>,
        Option<i32>,
        Option<bool>,
        DateTime<Utc>,
        DateTime<Utc>,
    )>(
        "INSERT INTO model_configs
            (workspace_id, user_id, name, provider, api_base_url, api_key_enc,
             model_name, temperature, max_tokens, is_default)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                   model_name, temperature, max_tokens, is_default, created_at, updated_at",
    )
    .bind(req.workspace_id)
    .bind(auth.id)
    .bind(&req.name)
    .bind(&req.provider)
    .bind(&req.api_base_url)
    .bind(&api_key_enc)
    .bind(&req.model_name)
    .bind(req.temperature)
    .bind(req.max_tokens)
    .bind(req.is_default)
    .fetch_one(&state.pool)
    .await?;

    let resp = ModelConfigResponse {
        id: row.0,
        workspace_id: row.1,
        user_id: row.2,
        name: row.3,
        provider: row.4,
        api_base_url: row.5,
        has_api_key: row.6.is_some(),
        model_name: row.7,
        temperature: row.8,
        max_tokens: row.9,
        is_default: row.10,
        created_at: row.11,
        updated_at: row.12,
    };

    Ok((StatusCode::CREATED, Json(resp)))
}

async fn update_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<ModelConfigResponse>, AppError> {
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM model_configs WHERE id = $1 AND user_id = $2",
    )
    .bind(config_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_none() {
        return Err(AppError::NotFound("Model config not found".into()));
    }

    if let Some(provider) = &req.provider {
        let valid = ["openai", "anthropic", "ollama", "custom"];
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
            "UPDATE model_configs SET is_default = false WHERE user_id = $1 AND id != $2",
        )
        .bind(auth.id)
        .bind(config_id)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, (
        Uuid,
        Option<Uuid>,
        Uuid,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<f32>,
        Option<i32>,
        Option<bool>,
        DateTime<Utc>,
        DateTime<Utc>,
    )>(
        "UPDATE model_configs SET
            name = COALESCE($1, name),
            provider = COALESCE($2, provider),
            api_base_url = COALESCE($3, api_base_url),
            api_key_enc = COALESCE($4, api_key_enc),
            model_name = COALESCE($5, model_name),
            temperature = COALESCE($6, temperature),
            max_tokens = COALESCE($7, max_tokens),
            is_default = COALESCE($8, is_default)
         WHERE id = $9 AND user_id = $10
         RETURNING id, workspace_id, user_id, name, provider, api_base_url, api_key_enc,
                   model_name, temperature, max_tokens, is_default, created_at, updated_at",
    )
    .bind(req.name)
    .bind(req.provider)
    .bind(req.api_base_url)
    .bind(api_key_enc)
    .bind(req.model_name)
    .bind(req.temperature)
    .bind(req.max_tokens)
    .bind(req.is_default)
    .bind(config_id)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ModelConfigResponse {
        id: row.0,
        workspace_id: row.1,
        user_id: row.2,
        name: row.3,
        provider: row.4,
        api_base_url: row.5,
        has_api_key: row.6.is_some(),
        model_name: row.7,
        temperature: row.8,
        max_tokens: row.9,
        is_default: row.10,
        created_at: row.11,
        updated_at: row.12,
    }))
}

async fn delete_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM model_configs WHERE id = $1 AND user_id = $2")
        .bind(config_id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Model config not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn test_connection(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(config_id): Path<Uuid>,
) -> Result<Json<TestConnectionResponse>, AppError> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "SELECT provider, api_base_url, api_key_enc, model_name
         FROM model_configs
         WHERE id = $1 AND user_id = $2",
    )
    .bind(config_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?;

    let (provider, api_base_url, api_key_enc, model_name) = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Model config not found".into())),
    };

    let api_key = api_key_enc.and_then(|enc| decrypt_api_key(&enc, &state.jwt_secret));

    let start = std::time::Instant::now();
    let result = test_provider_connection(&provider, &api_base_url, api_key.as_deref(), &model_name).await;
    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(msg) => Ok(Json(TestConnectionResponse {
            success: true,
            message: msg,
            latency_ms: Some(latency),
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            message: format!("Connection failed: {}", e),
            latency_ms: Some(latency),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = "sk-test-12345-abcdef";
        let secret = "my-jwt-secret";
        let encrypted = encrypt_api_key(key, secret);
        assert_ne!(encrypted, key);
        let decrypted = decrypt_api_key(&encrypted, secret).unwrap();
        assert_eq!(decrypted, key);
    }

    #[test]
    fn test_encrypt_different_secrets_produce_different_results() {
        let key = "sk-test-12345";
        let enc1 = encrypt_api_key(key, "secret-a");
        let enc2 = encrypt_api_key(key, "secret-b");
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_decrypt_wrong_secret_fails() {
        let key = "sk-test-12345";
        let encrypted = encrypt_api_key(key, "correct-secret");
        let decrypted = decrypt_api_key(&encrypted, "wrong-secret").unwrap();
        assert_ne!(decrypted, key);
    }

    #[test]
    fn test_encrypt_empty_key() {
        let encrypted = encrypt_api_key("", "secret");
        assert_eq!(encrypted, "");
        let decrypted = decrypt_api_key(&encrypted, "secret").unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_valid_providers() {
        let valid = ["openai", "anthropic", "ollama", "custom"];
        for p in &valid {
            assert!(valid.contains(p));
        }
        assert!(!valid.contains(&"invalid_provider"));
    }
}

async fn test_provider_connection(
    provider: &str,
    api_base_url: &str,
    api_key: Option<&str>,
    model_name: &str,
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let base = api_base_url.trim_end_matches('/');

    let models_url = format!("{}/models", base);
    let mut req = client.get(&models_url);
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }

    let resp = req
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", base, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("API returned status {}: {}", status, body);
    }

    let chat_url = format!("{}/chat/completions", base);
    let chat_body = serde_json::json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 5,
    });

    let mut chat_req = client.post(&chat_url).json(&chat_body);
    if let Some(key) = api_key {
        chat_req = chat_req.bearer_auth(key);
    }

    let chat_resp = chat_req
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Chat request failed: {}", e))?;

    if !chat_resp.status().is_success() {
        let status = chat_resp.status();
        let body = chat_resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "Model '{}' chat test failed (status {}): {}",
            model_name,
            status,
            body
        );
    }

    Ok(format!(
        "Connected to {} and model '{}' responded successfully",
        provider, model_name
    ))
}
