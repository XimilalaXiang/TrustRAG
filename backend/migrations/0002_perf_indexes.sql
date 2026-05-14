-- Performance indexes for common query patterns

-- Speed up chunk lookups by document with embedding presence
CREATE INDEX IF NOT EXISTS idx_chunks_doc_embedding
    ON document_chunks (document_id) WHERE embedding IS NOT NULL;

-- Speed up workspace document listing (commonly filtered + ordered)
CREATE INDEX IF NOT EXISTS idx_documents_workspace_created
    ON documents (workspace_id, created_at DESC);

-- Speed up conversation message listing
CREATE INDEX IF NOT EXISTS idx_messages_conv_created
    ON messages (conversation_id, created_at ASC);

-- Speed up conversation listing per workspace
CREATE INDEX IF NOT EXISTS idx_conversations_workspace_updated
    ON conversations (workspace_id, updated_at DESC);

-- Composite index for review lookups by citation + time
CREATE INDEX IF NOT EXISTS idx_reviews_citation_created
    ON review_records (citation_id, created_at DESC);
