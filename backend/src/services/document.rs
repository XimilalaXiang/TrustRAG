use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::chunking::{chunk_markdown, ChunkConfig};
use crate::services::embedding::store_chunk_embeddings;
use crate::services::storage::StorageService;
use crate::traits::embedding_provider::EmbeddingProvider;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocProcessorResponse {
    pub markdown: String,
    pub pages: serde_json::Value,
    pub headings: serde_json::Value,
    pub metadata: DocProcessorMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocProcessorMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<i32>,
    pub language: Option<String>,
}

/// Update document processing status.
async fn update_status(
    pool: &PgPool,
    doc_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE documents SET processing_status = $1, processing_error = $2 WHERE id = $3",
    )
    .bind(status)
    .bind(error)
    .bind(doc_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Full document processing pipeline (async task).
///
/// Steps:
/// 1. Download original file from MinIO
/// 2. Call Python doc-processor to parse
/// 3. Store Markdown in MinIO
/// 4. Chunk the Markdown
/// 5. Insert chunks into DB
/// 6. Generate embeddings
/// 7. Store embeddings in pgvector
/// 8. Update document status to 'ready'
pub async fn process_document(
    pool: PgPool,
    storage: StorageService,
    doc_processor_url: String,
    embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    doc_id: Uuid,
    workspace_id: Uuid,
) {
    if let Err(e) = process_document_inner(
        &pool,
        &storage,
        &doc_processor_url,
        embedding_provider.as_deref(),
        doc_id,
        workspace_id,
    )
    .await
    {
        tracing::error!("Document processing failed for {}: {}", doc_id, e);
        let _ = update_status(&pool, doc_id, "failed", Some(&e.to_string())).await;
    }
}

async fn process_document_inner(
    pool: &PgPool,
    storage: &StorageService,
    doc_processor_url: &str,
    embedding_provider: Option<&dyn EmbeddingProvider>,
    doc_id: Uuid,
    workspace_id: Uuid,
) -> anyhow::Result<()> {
    update_status(pool, doc_id, "processing", None).await?;

    let doc = sqlx::query_as::<_, (String, String, String)>(
        "SELECT original_file_path, original_filename, file_type FROM documents WHERE id = $1",
    )
    .bind(doc_id)
    .fetch_one(pool)
    .await?;

    let (file_path, filename, file_type) = doc;

    // Step 1: Download file from MinIO
    let file_bytes = storage.download(&file_path).await?;

    // Step 2: Call Python doc-processor
    let parse_url = match file_type.as_str() {
        "pdf" => format!("{}/api/parse/pdf", doc_processor_url),
        "docx" => format!("{}/api/parse/docx", doc_processor_url),
        "txt" | "md" | "html" => format!("{}/api/parse/txt", doc_processor_url),
        _ => anyhow::bail!("Unsupported file type: {}", file_type),
    };

    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(file_bytes.to_vec())
        .file_name(filename.clone())
        .mime_str("application/octet-stream")?;
    let form = reqwest::multipart::Form::new().part("file", part);

    let response = client
        .post(&parse_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("[doc-processor] request failed: {}", e))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("[doc-processor] returned error status: {}", e))?;

    let response_bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("[doc-processor] failed to read response body: {}", e))?;

    tracing::info!(
        "doc-processor response for {}: {} bytes",
        doc_id,
        response_bytes.len()
    );

    let parse_result: DocProcessorResponse = serde_json::from_slice(&response_bytes)
        .map_err(|e| {
            let preview = String::from_utf8_lossy(
                &response_bytes[..response_bytes.len().min(500)],
            );
            anyhow::anyhow!(
                "[doc-processor] failed to deserialize response: {}. Body preview: {}",
                e,
                preview
            )
        })?;

    // Step 3: Update document metadata
    sqlx::query(
        r#"
        UPDATE documents
        SET page_count = $1, language = $2,
            title = COALESCE($3, title)
        WHERE id = $4
        "#,
    )
    .bind(parse_result.metadata.page_count)
    .bind(&parse_result.metadata.language)
    .bind(&parse_result.metadata.title)
    .bind(doc_id)
    .execute(pool)
    .await?;

    // Step 4: Store Markdown in MinIO
    let md_path = StorageService::markdown_path(&workspace_id, &doc_id);
    storage
        .upload(&md_path, bytes::Bytes::from(parse_result.markdown.clone()))
        .await?;

    sqlx::query("UPDATE documents SET markdown_file_path = $1 WHERE id = $2")
        .bind(&md_path)
        .bind(doc_id)
        .execute(pool)
        .await?;

    // Step 5: Chunk the Markdown
    update_status(pool, doc_id, "chunking", None).await?;

    let chunk_config = ChunkConfig::default();
    let chunks = chunk_markdown(&parse_result.markdown, &chunk_config);

    // Step 6: Insert chunks into DB
    sqlx::query("DELETE FROM document_chunks WHERE document_id = $1")
        .bind(doc_id)
        .execute(pool)
        .await?;

    let mut chunk_ids = Vec::with_capacity(chunks.len());
    let mut chunk_texts = Vec::with_capacity(chunks.len());

    for chunk in &chunks {
        let chunk_id = Uuid::new_v4();
        chunk_ids.push(chunk_id);
        chunk_texts.push(chunk.content.clone());

        sqlx::query(
            r#"
            INSERT INTO document_chunks (
                id, document_id, chunk_index, heading_path, content,
                char_start, char_end, content_hash
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(chunk_id)
        .bind(doc_id)
        .bind(chunk.index as i32)
        .bind(&chunk.heading_path)
        .bind(&chunk.content)
        .bind(chunk.char_start as i64)
        .bind(chunk.char_end as i64)
        .bind(&chunk.content_hash)
        .execute(pool)
        .await?;
    }

    // Step 7: Generate and store embeddings
    if let Some(provider) = embedding_provider {
        update_status(pool, doc_id, "embedding", None).await?;
        tracing::info!(
            "Generating embeddings for {} ({} chunks, model: {})",
            doc_id,
            chunk_texts.len(),
            provider.model_name()
        );

        let embeddings = provider
            .embed_texts(&chunk_texts)
            .await
            .map_err(|e| anyhow::anyhow!("[embedding] failed to generate embeddings: {}", e))?;

        store_chunk_embeddings(pool, &chunk_ids, &embeddings)
            .await
            .map_err(|e| anyhow::anyhow!("[embedding] failed to store embeddings: {}", e))?;
    }

    // Step 8: Mark as ready
    update_status(pool, doc_id, "ready", None).await?;
    tracing::info!("Document {} processed: {} chunks", doc_id, chunks.len());

    Ok(())
}
