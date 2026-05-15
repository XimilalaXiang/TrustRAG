-- Add 'ollama' to the embedding_configs provider CHECK constraint
ALTER TABLE embedding_configs DROP CONSTRAINT IF EXISTS embedding_configs_provider_check;
ALTER TABLE embedding_configs ADD CONSTRAINT embedding_configs_provider_check
    CHECK (provider IN ('openai', 'ollama', 'local', 'custom'));
