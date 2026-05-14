-- TrustRAG PostgreSQL 扩展初始化
-- pgvector: 向量搜索（pgvector/pgvector 镜像自带）
-- pg_bigm: 中文全文检索（需要单独安装，MVP 阶段先用 pg_trgm 替代）

CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 尝试加载 pg_bigm，如果不可用则忽略（生产环境需要自定义镜像）
DO $$
BEGIN
    CREATE EXTENSION IF NOT EXISTS pg_bigm;
    RAISE NOTICE 'pg_bigm extension loaded';
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'pg_bigm not available, using pg_trgm as fallback for full-text search';
END
$$;
