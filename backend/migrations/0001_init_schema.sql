-- TrustRAG 初始数据库 Schema
-- 依赖扩展：pgvector, pg_bigm（在 infra/postgres/init.sql 中启用）

-- ============================================================
-- 1. 用户与认证
-- ============================================================

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           VARCHAR(255) NOT NULL UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    display_name    VARCHAR(100) NOT NULL,
    role            VARCHAR(20) NOT NULL DEFAULT 'user'
                    CHECK (role IN ('admin', 'reviewer', 'user')),
    status          VARCHAR(20) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'suspended', 'deleted')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at   TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status ON users (status);

-- ============================================================
-- 2. 工作区
-- ============================================================

CREATE TABLE workspaces (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(200) NOT NULL,
    description     TEXT,
    owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    visibility      VARCHAR(20) NOT NULL DEFAULT 'private'
                    CHECK (visibility IN ('private', 'shared', 'public')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_workspaces_owner ON workspaces (owner_id);

-- 工作区成员（多对多）
CREATE TABLE workspace_members (
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            VARCHAR(20) NOT NULL DEFAULT 'viewer'
                    CHECK (role IN ('admin', 'editor', 'viewer')),
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, user_id)
);

-- ============================================================
-- 3. 文档管理
-- ============================================================

CREATE TABLE documents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id        UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title               VARCHAR(500) NOT NULL,
    original_filename   VARCHAR(500) NOT NULL,
    file_type           VARCHAR(20) NOT NULL
                        CHECK (file_type IN ('pdf', 'docx', 'md', 'txt', 'html')),
    file_size_bytes     BIGINT,
    page_count          INT,
    language            VARCHAR(10),
    tags                JSONB DEFAULT '[]'::jsonb,
    original_file_path  VARCHAR(1000) NOT NULL,
    markdown_file_path  VARCHAR(1000),
    processing_status   VARCHAR(20) NOT NULL DEFAULT 'pending'
                        CHECK (processing_status IN (
                            'pending', 'processing', 'chunking',
                            'embedding', 'ready', 'failed'
                        )),
    processing_error    TEXT,
    uploaded_by         UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_documents_workspace ON documents (workspace_id);
CREATE INDEX idx_documents_status ON documents (processing_status);
CREATE INDEX idx_documents_uploaded_by ON documents (uploaded_by);
CREATE INDEX idx_documents_tags ON documents USING gin (tags);

-- ============================================================
-- 4. 文档分块 + 向量索引
-- ============================================================

CREATE TABLE document_chunks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index     INT NOT NULL,
    heading_path    VARCHAR(1000),
    section_level   SMALLINT,
    content         TEXT NOT NULL,
    content_tokens  INT,
    page_start      INT,
    page_end        INT,
    paragraph_index INT,
    char_start      BIGINT,
    char_end        BIGINT,
    embedding       vector(1536),
    content_hash    VARCHAR(64),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (document_id, chunk_index)
);

CREATE INDEX idx_chunks_document ON document_chunks (document_id);
CREATE INDEX idx_chunks_embedding ON document_chunks
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- pg_bigm 中文全文检索索引
CREATE INDEX idx_chunks_content_bigm ON document_chunks
    USING gin (content gin_bigm_ops);

-- ============================================================
-- 5. 模型配置
-- ============================================================

CREATE TABLE model_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            VARCHAR(200) NOT NULL,
    provider        VARCHAR(50) NOT NULL
                    CHECK (provider IN ('openai', 'anthropic', 'ollama', 'custom')),
    api_base_url    VARCHAR(500) NOT NULL,
    api_key_enc     VARCHAR(500),
    model_name      VARCHAR(200) NOT NULL,
    temperature     REAL DEFAULT 0.1,
    max_tokens      INT DEFAULT 4096,
    is_default      BOOLEAN DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_model_configs_user ON model_configs (user_id);

-- embedding 模型配置
CREATE TABLE embedding_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            VARCHAR(200) NOT NULL,
    provider        VARCHAR(50) NOT NULL
                    CHECK (provider IN ('openai', 'local', 'custom')),
    api_base_url    VARCHAR(500),
    api_key_enc     VARCHAR(500),
    model_name      VARCHAR(200) NOT NULL,
    dimensions      INT NOT NULL DEFAULT 1536,
    is_default      BOOLEAN DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- 6. 对话与消息
-- ============================================================

CREATE TABLE conversations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           VARCHAR(500),
    model_config_id UUID REFERENCES model_configs(id) ON DELETE SET NULL,
    document_scope  JSONB DEFAULT '[]'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_conversations_workspace ON conversations (workspace_id);
CREATE INDEX idx_conversations_user ON conversations (user_id);

CREATE TABLE messages (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id     UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role                VARCHAR(20) NOT NULL
                        CHECK (role IN ('user', 'assistant', 'system')),
    content             TEXT NOT NULL,
    model_name          VARCHAR(200),
    prompt_tokens       INT,
    completion_tokens   INT,
    latency_ms          INT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_messages_conversation ON messages (conversation_id, created_at);

-- ============================================================
-- 7. 引用与溯源（核心差异化）
-- ============================================================

CREATE TABLE citations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id      UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    document_id     UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_id        UUID NOT NULL REFERENCES document_chunks(id) ON DELETE CASCADE,
    citation_index  SMALLINT NOT NULL,
    quoted_text     TEXT,
    page_number     INT,
    heading_path    VARCHAR(1000),
    paragraph_index INT,
    char_start      BIGINT,
    char_end        BIGINT,
    relevance_score REAL,
    verified        BOOLEAN DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_citations_message ON citations (message_id);
CREATE INDEX idx_citations_document ON citations (document_id);
CREATE INDEX idx_citations_chunk ON citations (chunk_id);

-- ============================================================
-- 8. 审核记录（Phase 2，但表先建好）
-- ============================================================

CREATE TABLE review_records (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    citation_id     UUID NOT NULL REFERENCES citations(id) ON DELETE CASCADE,
    reviewer_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status          VARCHAR(20) NOT NULL
                    CHECK (status IN ('approved', 'rejected', 'flagged', 'pending')),
    comment         TEXT,
    corrected_text  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reviews_citation ON review_records (citation_id);
CREATE INDEX idx_reviews_reviewer ON review_records (reviewer_id);

-- ============================================================
-- 9. 触发器：自动更新 updated_at
-- ============================================================

CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_updated_at_users
    BEFORE UPDATE ON users FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_updated_at_workspaces
    BEFORE UPDATE ON workspaces FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_updated_at_documents
    BEFORE UPDATE ON documents FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_updated_at_conversations
    BEFORE UPDATE ON conversations FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_updated_at_model_configs
    BEFORE UPDATE ON model_configs FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();

CREATE TRIGGER set_updated_at_review_records
    BEFORE UPDATE ON review_records FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
