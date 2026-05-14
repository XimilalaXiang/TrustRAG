use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::services::storage::StorageService;

use super::AppState;

const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "docx", "md", "txt", "html"];

#[derive(Serialize, sqlx::FromRow)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub original_filename: String,
    pub file_type: String,
    pub file_size_bytes: Option<i64>,
    pub page_count: Option<i32>,
    pub language: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub processing_status: String,
    pub processing_error: Option<String>,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub file_type: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedDocuments {
    pub items: Vec<DocumentResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/workspaces/{ws_id}/documents",
            get(list_documents).post(upload_document),
        )
        .route(
            "/workspaces/{ws_id}/documents/{doc_id}",
            get(get_document).delete(delete_document),
        )
        .route(
            "/workspaces/{ws_id}/documents/{doc_id}/download",
            get(download_file),
        )
        .route(
            "/workspaces/{ws_id}/documents/{doc_id}/markdown",
            get(get_markdown),
        )
        .route(
            "/workspaces/{ws_id}/documents/{doc_id}/chunks",
            get(list_chunks),
        )
        .route(
            "/workspaces/{ws_id}/documents/{doc_id}/reprocess",
            post(reprocess_document),
        )
}

fn extract_extension(filename: &str) -> Option<String> {
    filename
        .rsplit('.')
        .next()
        .map(|ext| ext.to_lowercase())
}

fn validate_file_type(filename: &str) -> Result<String, AppError> {
    let ext = extract_extension(filename)
        .ok_or_else(|| AppError::BadRequest("File must have an extension".into()))?;

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file type '{}'. Allowed: {}",
            ext,
            ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    Ok(ext)
}

/// Verify user has access to workspace
async fn check_workspace_access(
    pool: &sqlx::PgPool,
    ws_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
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
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if !has_access {
        return Err(AppError::NotFound("Workspace not found".into()));
    }
    Ok(())
}

async fn list_documents(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(ws_id): Path<Uuid>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<Json<PaginatedDocuments>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM documents
        WHERE workspace_id = $1
          AND ($2::text IS NULL OR processing_status = $2)
          AND ($3::text IS NULL OR file_type = $3)
        "#,
    )
    .bind(ws_id)
    .bind(&params.status)
    .bind(&params.file_type)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, DocumentResponse>(
        r#"
        SELECT id, workspace_id, title, original_filename, file_type,
               file_size_bytes, page_count, language, tags,
               processing_status, processing_error, uploaded_by,
               created_at, updated_at
        FROM documents
        WHERE workspace_id = $1
          AND ($2::text IS NULL OR processing_status = $2)
          AND ($3::text IS NULL OR file_type = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(ws_id)
    .bind(&params.status)
    .bind(&params.file_type)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedDocuments {
        items,
        total,
        page,
        per_page,
    }))
}

async fn upload_document(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(ws_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DocumentResponse>), AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let mut file_data: Option<bytes::Bytes> = None;
    let mut file_name: Option<String> = None;
    let mut title: Option<String> = None;
    let mut tags: Option<serde_json::Value> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

                let max_bytes = state.max_upload_size * 1024 * 1024;
                if data.len() as u64 > max_bytes {
                    return Err(AppError::BadRequest(format!(
                        "File too large. Maximum size: {} MB",
                        state.max_upload_size
                    )));
                }

                file_data = Some(data);
            }
            "title" => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Invalid title: {e}")))?,
                );
            }
            "tags" => {
                let raw = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid tags: {e}")))?;
                tags = Some(
                    serde_json::from_str(&raw)
                        .unwrap_or_else(|_| serde_json::json!([])),
                );
            }
            _ => {}
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError::BadRequest("No file provided".into()))?;
    let original_filename =
        file_name.ok_or_else(|| AppError::BadRequest("File must have a name".into()))?;

    let file_type = validate_file_type(&original_filename)?;
    let file_size = file_data.len() as i64;
    let doc_title = title.unwrap_or_else(|| original_filename.clone());

    let doc_id = Uuid::new_v4();
    let storage_path = StorageService::document_path(&ws_id, &doc_id, &original_filename);

    state
        .storage
        .upload(&storage_path, file_data)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Storage upload failed: {e}")))?;

    let doc = sqlx::query_as::<_, DocumentResponse>(
        r#"
        INSERT INTO documents (
            id, workspace_id, title, original_filename, file_type,
            file_size_bytes, tags, original_file_path, uploaded_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, workspace_id, title, original_filename, file_type,
                  file_size_bytes, page_count, language, tags,
                  processing_status, processing_error, uploaded_by,
                  created_at, updated_at
        "#,
    )
    .bind(doc_id)
    .bind(ws_id)
    .bind(&doc_title)
    .bind(&original_filename)
    .bind(&file_type)
    .bind(file_size)
    .bind(&tags.unwrap_or_else(|| serde_json::json!([])))
    .bind(&storage_path)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(doc)))
}

async fn get_document(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DocumentResponse>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let doc = sqlx::query_as::<_, DocumentResponse>(
        r#"
        SELECT id, workspace_id, title, original_filename, file_type,
               file_size_bytes, page_count, language, tags,
               processing_status, processing_error, uploaded_by,
               created_at, updated_at
        FROM documents
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    Ok(Json(doc))
}

async fn delete_document(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let _doc = sqlx::query_as::<_, (String,)>(
        "SELECT original_file_path FROM documents WHERE id = $1 AND workspace_id = $2",
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    let dir_prefix = format!("workspaces/{}/documents/{}/", ws_id, doc_id);
    if let Err(e) = state.storage.delete_dir(&dir_prefix).await {
        tracing::warn!("Failed to delete storage files for doc {}: {}", doc_id, e);
    }

    sqlx::query("DELETE FROM documents WHERE id = $1 AND workspace_id = $2")
        .bind(doc_id)
        .bind(ws_id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn download_file(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let doc = sqlx::query_as::<_, (String, String)>(
        "SELECT original_file_path, original_filename FROM documents WHERE id = $1 AND workspace_id = $2",
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    let (file_path, filename) = doc;

    let data = state
        .storage
        .download(&file_path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Storage download failed: {e}")))?;

    let content_type = match extract_extension(&filename).as_deref() {
        Some("pdf") => "application/pdf",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("md") => "text/markdown",
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        _ => "application/octet-stream",
    };

    let disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    ))
}

async fn reprocess_document(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DocumentResponse>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let doc = sqlx::query_as::<_, DocumentResponse>(
        r#"
        UPDATE documents
        SET processing_status = 'pending', processing_error = NULL
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, title, original_filename, file_type,
                  file_size_bytes, page_count, language, tags,
                  processing_status, processing_error, uploaded_by,
                  created_at, updated_at
        "#,
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // TODO: trigger async document processing task

    Ok(Json(doc))
}

// === Chunk and Markdown endpoints ===

#[derive(Serialize, sqlx::FromRow)]
pub struct ChunkResponse {
    pub id: Uuid,
    pub document_id: Uuid,
    pub chunk_index: i32,
    pub heading_path: Option<String>,
    pub section_level: Option<i16>,
    pub content: String,
    pub content_tokens: Option<i32>,
    pub page_start: Option<i32>,
    pub page_end: Option<i32>,
    pub paragraph_index: Option<i32>,
    pub char_start: Option<i64>,
    pub char_end: Option<i64>,
    pub content_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ListChunksQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedChunks {
    pub items: Vec<ChunkResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

async fn list_chunks(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<ListChunksQuery>,
) -> Result<Json<PaginatedChunks>, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND workspace_id = $2)",
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_one(&state.pool)
    .await?
    .then_some(())
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM document_chunks WHERE document_id = $1",
    )
    .bind(doc_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, ChunkResponse>(
        r#"
        SELECT id, document_id, chunk_index, heading_path, section_level,
               content, content_tokens, page_start, page_end,
               paragraph_index, char_start, char_end, content_hash, created_at
        FROM document_chunks
        WHERE document_id = $1
        ORDER BY chunk_index ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(doc_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedChunks {
        items,
        total,
        page,
        per_page,
    }))
}

async fn get_markdown(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((ws_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    check_workspace_access(&state.pool, ws_id, auth.id).await?;

    let doc = sqlx::query_as::<_, (Option<String>, String)>(
        "SELECT markdown_file_path, title FROM documents WHERE id = $1 AND workspace_id = $2",
    )
    .bind(doc_id)
    .bind(ws_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    let (md_path, title) = doc;
    let md_path = md_path.ok_or_else(|| {
        AppError::NotFound("Markdown version not available yet (document may still be processing)".into())
    })?;

    let data = state
        .storage
        .download(&md_path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Storage download failed: {e}")))?;

    let filename = format!("{}.md", title.replace('/', "_"));
    Ok((
        [
            (header::CONTENT_TYPE, "text/markdown; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", filename),
            ),
        ],
        data,
    ))
}
