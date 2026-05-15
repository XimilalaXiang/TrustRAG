-- TrustRAG SQLite Schema (desktop mode)
-- Equivalent to PostgreSQL migrations 0001-0006

CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('admin', 'reviewer', 'user')),
    status          TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_login_at   TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_status ON users (status);

CREATE TABLE IF NOT EXISTS workspaces (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    name            TEXT NOT NULL,
    description     TEXT,
    owner_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    visibility      TEXT NOT NULL DEFAULT 'private' CHECK (visibility IN ('private', 'shared', 'public')),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_workspaces_owner ON workspaces (owner_id);

CREATE TABLE IF NOT EXISTS workspace_members (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            TEXT NOT NULL DEFAULT 'viewer' CHECK (role IN ('owner', 'admin', 'editor', 'viewer')),
    invited_by      TEXT REFERENCES users(id),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (workspace_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_workspace_members_workspace ON workspace_members(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspace_members_user ON workspace_members(user_id);

CREATE TABLE IF NOT EXISTS documents (
    id                  TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id        TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title               TEXT NOT NULL,
    original_filename   TEXT NOT NULL,
    file_type           TEXT NOT NULL CHECK (file_type IN ('pdf', 'docx', 'md', 'txt', 'html')),
    file_size_bytes     INTEGER,
    page_count          INTEGER,
    language            TEXT,
    tags                TEXT DEFAULT '[]',
    original_file_path  TEXT NOT NULL,
    markdown_file_path  TEXT,
    processing_status   TEXT NOT NULL DEFAULT 'pending' CHECK (processing_status IN ('pending', 'processing', 'chunking', 'embedding', 'ready', 'failed')),
    processing_error    TEXT,
    uploaded_by         TEXT NOT NULL REFERENCES users(id),
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_documents_workspace ON documents (workspace_id);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents (processing_status);
CREATE INDEX IF NOT EXISTS idx_documents_uploaded_by ON documents (uploaded_by);
CREATE INDEX IF NOT EXISTS idx_documents_workspace_created ON documents (workspace_id, created_at);

CREATE TABLE IF NOT EXISTS document_chunks (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    document_id     TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index     INTEGER NOT NULL,
    heading_path    TEXT,
    section_level   INTEGER,
    content         TEXT NOT NULL,
    content_tokens  INTEGER,
    page_start      INTEGER,
    page_end        INTEGER,
    paragraph_index INTEGER,
    char_start      INTEGER,
    char_end        INTEGER,
    embedding       BLOB,
    content_hash    TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (document_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_chunks_document ON document_chunks (document_id);
CREATE INDEX IF NOT EXISTS idx_chunks_doc_embedding ON document_chunks (document_id) WHERE embedding IS NOT NULL;

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    content,
    chunk_id UNINDEXED,
    content='document_chunks',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 in sync
CREATE TRIGGER IF NOT EXISTS chunks_fts_insert AFTER INSERT ON document_chunks BEGIN
    INSERT INTO chunks_fts(rowid, content, chunk_id) VALUES (new.rowid, new.content, new.id);
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_delete AFTER DELETE ON document_chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, content, chunk_id) VALUES ('delete', old.rowid, old.content, old.id);
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_update AFTER UPDATE OF content ON document_chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, content, chunk_id) VALUES ('delete', old.rowid, old.content, old.id);
    INSERT INTO chunks_fts(rowid, content, chunk_id) VALUES (new.rowid, new.content, new.id);
END;

CREATE TABLE IF NOT EXISTS model_configs (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id    TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    provider        TEXT NOT NULL CHECK (provider IN ('openai', 'anthropic', 'ollama', 'custom')),
    api_base_url    TEXT NOT NULL,
    api_key_enc     TEXT,
    model_name      TEXT NOT NULL,
    temperature     REAL DEFAULT 0.1,
    max_tokens      INTEGER DEFAULT 4096,
    is_default      INTEGER DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_model_configs_user ON model_configs (user_id);

CREATE TABLE IF NOT EXISTS embedding_configs (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id    TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    provider        TEXT NOT NULL CHECK (provider IN ('openai', 'ollama', 'local', 'custom')),
    api_base_url    TEXT,
    api_key_enc     TEXT,
    model_name      TEXT NOT NULL,
    dimensions      INTEGER NOT NULL DEFAULT 1536,
    is_default      INTEGER DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS conversations (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           TEXT,
    model_config_id TEXT REFERENCES model_configs(id) ON DELETE SET NULL,
    document_scope  TEXT DEFAULT '[]',
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_conversations_workspace ON conversations (workspace_id);
CREATE INDEX IF NOT EXISTS idx_conversations_user ON conversations (user_id);
CREATE INDEX IF NOT EXISTS idx_conversations_workspace_updated ON conversations (workspace_id, updated_at);

CREATE TABLE IF NOT EXISTS messages (
    id                  TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    conversation_id     TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role                TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content             TEXT NOT NULL,
    model_name          TEXT,
    prompt_tokens       INTEGER,
    completion_tokens   INTEGER,
    latency_ms          INTEGER,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages (conversation_id, created_at);
CREATE INDEX IF NOT EXISTS idx_messages_conv_created ON messages (conversation_id, created_at);

CREATE TABLE IF NOT EXISTS citations (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    message_id      TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    document_id     TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_id        TEXT NOT NULL REFERENCES document_chunks(id) ON DELETE CASCADE,
    citation_index  INTEGER NOT NULL,
    quoted_text     TEXT,
    page_number     INTEGER,
    heading_path    TEXT,
    paragraph_index INTEGER,
    char_start      INTEGER,
    char_end        INTEGER,
    relevance_score REAL,
    verified        INTEGER DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_citations_message ON citations (message_id);
CREATE INDEX IF NOT EXISTS idx_citations_document ON citations (document_id);
CREATE INDEX IF NOT EXISTS idx_citations_chunk ON citations (chunk_id);

CREATE TABLE IF NOT EXISTS review_records (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    citation_id     TEXT NOT NULL REFERENCES citations(id) ON DELETE CASCADE,
    reviewer_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status          TEXT NOT NULL CHECK (status IN ('approved', 'rejected', 'flagged', 'pending')),
    comment         TEXT,
    corrected_text  TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_reviews_citation ON review_records (citation_id);
CREATE INDEX IF NOT EXISTS idx_reviews_reviewer ON review_records (reviewer_id);
CREATE INDEX IF NOT EXISTS idx_reviews_citation_created ON review_records (citation_id, created_at);

CREATE TABLE IF NOT EXISTS entities (
    id              TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    entity_type     TEXT NOT NULL DEFAULT 'concept',
    document_id     TEXT REFERENCES documents(id) ON DELETE SET NULL,
    chunk_id        TEXT REFERENCES document_chunks(id) ON DELETE SET NULL,
    metadata        TEXT DEFAULT '{}',
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS entity_relations (
    id                  TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6)))),
    workspace_id        TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_entity_id    TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_entity_id    TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    relation_type       TEXT NOT NULL DEFAULT 'related_to',
    weight              REAL NOT NULL DEFAULT 1.0,
    metadata            TEXT DEFAULT '{}',
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_entities_workspace ON entities(workspace_id);
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(workspace_id, name);
CREATE INDEX IF NOT EXISTS idx_entity_relations_source ON entity_relations(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relations_target ON entity_relations(target_entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_relations_workspace ON entity_relations(workspace_id);
