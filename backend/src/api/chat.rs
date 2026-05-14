use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::services::llm::OpenAILlmProvider;
use crate::services::rag::{self, AssembledSource, RagConfig};
use crate::traits::llm_provider::{LlmMessage, LlmProvider, StreamEvent};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces/{ws_id}/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/workspaces/{ws_id}/conversations/{conv_id}",
            get(get_conversation).delete(delete_conversation),
        )
        .route(
            "/workspaces/{ws_id}/conversations/{conv_id}/messages",
            get(list_messages).post(send_message),
        )
}

// ── Types ──

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
    pub model_config_id: Option<Uuid>,
    #[serde(default)]
    pub document_scope: Vec<Uuid>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub model_config_id: Option<Uuid>,
    pub document_scope: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub model_name: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub latency_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub document_scope: Vec<Uuid>,
    pub model_config_id: Option<Uuid>,
}

#[derive(Serialize)]
struct MessageStartEvent {
    message_id: Uuid,
    model: String,
}

#[derive(Serialize)]
struct TextDeltaEvent {
    delta: String,
}

#[derive(Serialize)]
struct CitationEvent {
    index: usize,
    chunk_id: Uuid,
    document_id: Uuid,
    heading: Option<String>,
    page: Option<i32>,
    score: f64,
    text: String,
}

#[derive(Serialize)]
struct MessageEndEvent {
    message_id: Uuid,
    prompt_tokens: u32,
    completion_tokens: u32,
    latency_ms: u64,
}

#[derive(Serialize)]
struct SuggestionsEvent {
    questions: Vec<String>,
}

#[derive(Serialize)]
struct NonStreamingResponse {
    message: MessageResponse,
    citations: Vec<CitationEvent>,
}

// ── Conversation CRUD ──

async fn create_conversation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ws_id): Path<Uuid>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), AppError> {
    let scope_json = serde_json::to_value(&req.document_scope).unwrap_or_default();

    let conv = sqlx::query_as::<_, ConversationResponse>(
        "INSERT INTO conversations (workspace_id, user_id, title, model_config_id, document_scope)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, workspace_id, user_id, title, model_config_id, document_scope, created_at, updated_at",
    )
    .bind(ws_id)
    .bind(auth.id)
    .bind(&req.title)
    .bind(req.model_config_id)
    .bind(&scope_json)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(conv)))
}

async fn list_conversations(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<Json<Vec<ConversationResponse>>, AppError> {
    let convs = sqlx::query_as::<_, ConversationResponse>(
        "SELECT id, workspace_id, user_id, title, model_config_id, document_scope, created_at, updated_at
         FROM conversations
         WHERE workspace_id = $1 AND user_id = $2
         ORDER BY updated_at DESC",
    )
    .bind(ws_id)
    .bind(auth.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(convs))
}

async fn get_conversation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, conv_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ConversationResponse>, AppError> {
    let conv = sqlx::query_as::<_, ConversationResponse>(
        "SELECT id, workspace_id, user_id, title, model_config_id, document_scope, created_at, updated_at
         FROM conversations
         WHERE id = $1 AND workspace_id = $2 AND user_id = $3",
    )
    .bind(conv_id)
    .bind(ws_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation not found".into()))?;

    Ok(Json(conv))
}

async fn delete_conversation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, conv_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "DELETE FROM conversations WHERE id = $1 AND workspace_id = $2 AND user_id = $3",
    )
    .bind(conv_id)
    .bind(ws_id)
    .bind(auth.id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Conversation not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn list_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, conv_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    // Verify ownership
    let _conv = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM conversations WHERE id = $1 AND workspace_id = $2 AND user_id = $3",
    )
    .bind(conv_id)
    .bind(ws_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation not found".into()))?;

    let msgs = sqlx::query_as::<_, MessageResponse>(
        "SELECT id, conversation_id, role, content, model_name, prompt_tokens,
                completion_tokens, latency_ms, created_at
         FROM messages
         WHERE conversation_id = $1
         ORDER BY created_at ASC",
    )
    .bind(conv_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(msgs))
}

// ── Send Message (main RAG endpoint) ──

async fn send_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((ws_id, conv_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    let content = req.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::BadRequest("Message content cannot be empty".into()));
    }

    tracing::info!(
        user_id = %auth.id,
        workspace_id = %ws_id,
        conversation_id = %conv_id,
        query_len = content.len(),
        "Chat message received"
    );

    let conv = sqlx::query_as::<_, (Option<Uuid>, serde_json::Value)>(
        "SELECT model_config_id, document_scope
         FROM conversations WHERE id = $1 AND workspace_id = $2 AND user_id = $3",
    )
    .bind(conv_id)
    .bind(ws_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation not found".into()))?;

    let (conv_model_config_id, conv_doc_scope) = conv;

    // Save user message and touch conversation updated_at
    sqlx::query(
        "INSERT INTO messages (conversation_id, role, content) VALUES ($1, 'user', $2)",
    )
    .bind(conv_id)
    .bind(&content)
    .execute(&state.pool)
    .await?;

    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conv_id)
        .execute(&state.pool)
        .await?;

    // Determine model config
    let model_config_id = req.model_config_id.or(conv_model_config_id);

    // Load model config
    let (provider_name, api_base_url, api_key_enc, model_name, temperature, max_tokens) =
        if let Some(mc_id) = model_config_id {
            sqlx::query_as::<_, (String, String, Option<String>, String, Option<f32>, Option<i32>)>(
                "SELECT provider, api_base_url, api_key_enc, model_name, temperature, max_tokens
                 FROM model_configs WHERE id = $1 AND user_id = $2",
            )
            .bind(mc_id)
            .bind(auth.id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Model config not found".into()))?
        } else {
            // Try user's default model
            sqlx::query_as::<_, (String, String, Option<String>, String, Option<f32>, Option<i32>)>(
                "SELECT provider, api_base_url, api_key_enc, model_name, temperature, max_tokens
                 FROM model_configs WHERE user_id = $1 AND is_default = true LIMIT 1",
            )
            .bind(auth.id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::BadRequest("No model configured. Please create a model config first.".into()))?
        };

    let api_key = api_key_enc.and_then(|enc| {
        crate::api::models::decrypt_api_key(&enc, &state.jwt_secret)
    });

    let llm_provider = OpenAILlmProvider::new(
        &api_base_url,
        api_key.as_deref(),
        &model_name,
    );

    // Merge document scope
    let mut doc_scope: Vec<Uuid> = req.document_scope.clone();
    if doc_scope.is_empty() {
        if let Ok(scope_vec) = serde_json::from_value::<Vec<Uuid>>(conv_doc_scope) {
            doc_scope = scope_vec;
        }
    }

    // Load prior conversation history (exclude the user message we just inserted)
    let history_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT role, content FROM (
            SELECT role, content, created_at FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at DESC
            OFFSET 1
         ) sub ORDER BY created_at ASC",
    )
    .bind(conv_id)
    .fetch_all(&state.pool)
    .await?;

    let history: Vec<LlmMessage> = history_rows
        .into_iter()
        .map(|(role, content)| LlmMessage { role, content })
        .collect();

    let rag_config = RagConfig {
        temperature: temperature.unwrap_or(0.1),
        max_tokens: max_tokens.unwrap_or(4096) as u32,
        ..RagConfig::default()
    };

    let embedding_provider = state.embedding_provider.read().await.clone();

    if req.stream {
        // Return SSE stream
        let pool = state.pool.clone();
        let query = content.clone();

        let stream = build_sse_stream(
            pool,
            embedding_provider,
            Arc::new(llm_provider),
            ws_id,
            conv_id,
            query,
            history,
            doc_scope,
            rag_config,
        );

        Ok(Sse::new(stream).keep_alive(KeepAlive::default()).into_response())
    } else {
        // Non-streaming
        let embedding_provider = embedding_provider
            .ok_or_else(|| AppError::BadRequest("Embedding provider not configured".into()))?;

        let start = std::time::Instant::now();
        let result = rag::run_rag_pipeline(
            &state.pool,
            embedding_provider.as_ref(),
            &llm_provider,
            ws_id,
            &content,
            &history,
            &doc_scope,
            &rag_config,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("RAG pipeline error: {}", e)))?;

        let latency = start.elapsed().as_millis() as i32;

        // Save assistant message
        let msg = sqlx::query_as::<_, MessageResponse>(
            "INSERT INTO messages (conversation_id, role, content, model_name, prompt_tokens, completion_tokens, latency_ms)
             VALUES ($1, 'assistant', $2, $3, $4, $5, $6)
             RETURNING id, conversation_id, role, content, model_name, prompt_tokens, completion_tokens, latency_ms, created_at",
        )
        .bind(conv_id)
        .bind(&result.answer)
        .bind(&result.model)
        .bind(result.prompt_tokens as i32)
        .bind(result.completion_tokens as i32)
        .bind(latency)
        .fetch_one(&state.pool)
        .await?;

        let citations: Vec<CitationEvent> = result
            .sources
            .iter()
            .map(|s| CitationEvent {
                index: s.index,
                chunk_id: s.chunk_id,
                document_id: s.document_id,
                heading: s.heading_path.clone(),
                page: s.page_start,
                score: s.score,
                text: s.content.chars().take(200).collect(),
            })
            .collect();

        Ok(Json(NonStreamingResponse {
            message: msg,
            citations,
        })
        .into_response())
    }
}

fn build_sse_stream(
    pool: sqlx::PgPool,
    embedding_provider: Option<Arc<dyn crate::traits::embedding_provider::EmbeddingProvider>>,
    llm_provider: Arc<OpenAILlmProvider>,
    workspace_id: Uuid,
    conv_id: Uuid,
    query: String,
    history: Vec<LlmMessage>,
    doc_scope: Vec<Uuid>,
    rag_config: RagConfig,
) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        let message_id = Uuid::new_v4();
        let start = std::time::Instant::now();

        tracing::info!(
            message_id = %message_id,
            workspace_id = %workspace_id,
            conversation_id = %conv_id,
            query_len = query.len(),
            doc_scope_count = doc_scope.len(),
            "SSE stream started"
        );

        let start_data = serde_json::to_string(&MessageStartEvent {
            message_id,
            model: llm_provider.model_name().to_string(),
        }).unwrap_or_default();
        yield Ok(Event::default().event("message_start").data(start_data));

        let analysis = rag::analyze_query(&query, &history);
        tracing::debug!(
            needs_retrieval = analysis.needs_retrieval,
            rewritten_query = %analysis.rewritten_query,
            "Query analyzed"
        );

        let sources: Vec<AssembledSource> = if analysis.needs_retrieval {
            let Some(emb_provider) = embedding_provider else {
                tracing::error!(message_id = %message_id, "Embedding provider not configured");
                yield Ok(Event::default().event("error").data("Embedding provider not configured. Please configure an embedding model."));
                return;
            };

            let search_config = crate::services::search::SearchConfig {
                mode: rag_config.search_mode.clone(),
                top_k: rag_config.search_top_k,
                min_score: rag_config.search_min_score,
                use_mmr: false,
                mmr_lambda: 0.7,
                rrf_k: 60.0,
            };

            match crate::services::search::hybrid_search(
                &pool,
                emb_provider.as_ref(),
                workspace_id,
                &analysis.rewritten_query,
                &search_config,
                if doc_scope.is_empty() { None } else { Some(&doc_scope) },
            ).await {
                Ok(resp) => {
                    let (context, sources) = rag::assemble_context(&resp.results, rag_config.max_context_chars);

                    // Emit citations
                    for s in &sources {
                        let citation = CitationEvent {
                            index: s.index,
                            chunk_id: s.chunk_id,
                            document_id: s.document_id,
                            heading: s.heading_path.clone(),
                            page: s.page_start,
                            score: s.score,
                            text: s.content.chars().take(200).collect(),
                        };
                        if let Ok(data) = serde_json::to_string(&citation) {
                            yield Ok(Event::default().event("citation").data(data));
                        }
                    }

                    // Stream LLM
                    let messages = if sources.is_empty() {
                        rag::build_chitchat_prompt(&query, &history)
                    } else {
                        rag::build_prompt(&query, &context, &history, &rag_config.language)
                    };

                    let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);
                    let llm = llm_provider.clone();
                    let llm_req = crate::traits::llm_provider::LlmRequest {
                        messages,
                        temperature: rag_config.temperature,
                        max_tokens: rag_config.max_tokens,
                        stream: true,
                    };

                    tokio::spawn(async move {
                        if let Err(e) = llm.stream(&llm_req, tx.clone()).await {
                            let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                        }
                    });

                    let mut full_content = String::new();
                    let mut final_prompt_tokens = 0u32;
                    let mut final_completion_tokens = 0u32;

                    while let Some(event) = rx.recv().await {
                        match event {
                            StreamEvent::Delta(text) => {
                                full_content.push_str(&text);
                                let delta = serde_json::to_string(&TextDeltaEvent { delta: text }).unwrap_or_default();
                                yield Ok(Event::default().event("text_delta").data(delta));
                            }
                            StreamEvent::Done(resp) => {
                                full_content = resp.content;
                                final_prompt_tokens = resp.prompt_tokens;
                                final_completion_tokens = resp.completion_tokens;
                                break;
                            }
                            StreamEvent::Error(e) => {
                                yield Ok(Event::default().event("error").data(e));
                                break;
                            }
                        }
                    }

                    let latency = start.elapsed().as_millis() as i32;
                    if let Err(e) = sqlx::query(
                        "INSERT INTO messages (id, conversation_id, role, content, model_name, prompt_tokens, completion_tokens, latency_ms)
                         VALUES ($1, $2, 'assistant', $3, $4, $5, $6, $7)",
                    )
                    .bind(message_id)
                    .bind(conv_id)
                    .bind(&full_content)
                    .bind(llm_provider.model_name())
                    .bind(final_prompt_tokens as i32)
                    .bind(final_completion_tokens as i32)
                    .bind(latency)
                    .execute(&pool)
                    .await {
                        tracing::error!(
                            message_id = %message_id,
                            error = %e,
                            "Failed to save assistant message to database"
                        );
                    }

                    let end_data = serde_json::to_string(&MessageEndEvent {
                        message_id,
                        prompt_tokens: final_prompt_tokens,
                        completion_tokens: final_completion_tokens,
                        latency_ms: start.elapsed().as_millis() as u64,
                    }).unwrap_or_default();
                    yield Ok(Event::default().event("message_end").data(end_data));

                    // Generate follow-up suggestions
                    let suggestions = rag::generate_follow_up_questions(
                        llm_provider.as_ref(), &query, &full_content
                    ).await;
                    if !suggestions.is_empty() {
                        if let Ok(data) = serde_json::to_string(&SuggestionsEvent { questions: suggestions }) {
                            yield Ok(Event::default().event("suggestions").data(data));
                        }
                    }

                    sources
                }
                Err(e) => {
                    tracing::error!(message_id = %message_id, error = %e, "Search failed");
                    yield Ok(Event::default().event("error").data(format!("Search error: {}", e)));
                    vec![]
                }
            }
        } else {
            // Chitchat path
            let messages = rag::build_chitchat_prompt(&query, &history);
            let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);
            let llm = llm_provider.clone();
            let llm_req = crate::traits::llm_provider::LlmRequest {
                messages,
                temperature: rag_config.temperature.max(0.5),
                max_tokens: rag_config.max_tokens,
                stream: true,
            };

            tokio::spawn(async move {
                if let Err(e) = llm.stream(&llm_req, tx.clone()).await {
                    let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                }
            });

            let mut full_content = String::new();
            let mut final_prompt_tokens = 0u32;
            let mut final_completion_tokens = 0u32;

            while let Some(event) = rx.recv().await {
                match event {
                    StreamEvent::Delta(text) => {
                        full_content.push_str(&text);
                        let delta = serde_json::to_string(&TextDeltaEvent { delta: text }).unwrap_or_default();
                        yield Ok(Event::default().event("text_delta").data(delta));
                    }
                    StreamEvent::Done(resp) => {
                        full_content = resp.content;
                        final_prompt_tokens = resp.prompt_tokens;
                        final_completion_tokens = resp.completion_tokens;
                        break;
                    }
                    StreamEvent::Error(e) => {
                        yield Ok(Event::default().event("error").data(e));
                        break;
                    }
                }
            }

            let latency = start.elapsed().as_millis() as i32;
            if let Err(e) = sqlx::query(
                "INSERT INTO messages (id, conversation_id, role, content, model_name, prompt_tokens, completion_tokens, latency_ms)
                 VALUES ($1, $2, 'assistant', $3, $4, $5, $6, $7)",
            )
            .bind(message_id)
            .bind(conv_id)
            .bind(&full_content)
            .bind(llm_provider.model_name())
            .bind(final_prompt_tokens as i32)
            .bind(final_completion_tokens as i32)
            .bind(latency)
            .execute(&pool)
            .await {
                tracing::error!(
                    message_id = %message_id,
                    error = %e,
                    "Failed to save chitchat message to database"
                );
            }

            let end_data = serde_json::to_string(&MessageEndEvent {
                message_id,
                prompt_tokens: final_prompt_tokens,
                completion_tokens: final_completion_tokens,
                latency_ms: start.elapsed().as_millis() as u64,
            }).unwrap_or_default();
            yield Ok(Event::default().event("message_end").data(end_data));

            // Generate follow-up suggestions for chitchat too
            let suggestions = rag::generate_follow_up_questions(
                llm_provider.as_ref(), &query, &full_content
            ).await;
            if !suggestions.is_empty() {
                if let Ok(data) = serde_json::to_string(&SuggestionsEvent { questions: suggestions }) {
                    yield Ok(Event::default().event("suggestions").data(data));
                }
            }

            vec![]
        };

        let _ = sources;
    }
}
