use opendal::Operator;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct StorageService {
    operator: Operator,
    #[allow(dead_code)]
    root: String,
}

impl StorageService {
    #[cfg(feature = "postgres")]
    pub fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let builder = opendal::services::S3::default()
            .endpoint(&config.minio_endpoint)
            .access_key_id(&config.minio_access_key)
            .secret_access_key(&config.minio_secret_key)
            .bucket(&config.minio_bucket)
            .region(&config.minio_region);

        let operator = Operator::new(builder)?.finish();

        Ok(Self {
            operator,
            root: config.minio_bucket.clone(),
        })
    }

    #[cfg(sqlite_mode)]
    pub fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let storage_dir = format!("{}/files", config.data_dir);
        std::fs::create_dir_all(&storage_dir)?;

        let builder = opendal::services::Fs::default()
            .root(&storage_dir);

        let operator = Operator::new(builder)?.finish();

        Ok(Self {
            operator,
            root: storage_dir,
        })
    }

    pub fn document_path(workspace_id: &uuid::Uuid, doc_id: &uuid::Uuid, filename: &str) -> String {
        format!("workspaces/{}/documents/{}/{}", workspace_id, doc_id, filename)
    }

    pub fn markdown_path(workspace_id: &uuid::Uuid, doc_id: &uuid::Uuid) -> String {
        format!("workspaces/{}/documents/{}/markdown.md", workspace_id, doc_id)
    }

    pub async fn upload(&self, path: &str, data: bytes::Bytes) -> anyhow::Result<()> {
        self.operator.write(path, data).await?;
        Ok(())
    }

    pub async fn download(&self, path: &str) -> anyhow::Result<bytes::Bytes> {
        let data = self.operator.read(path).await?;
        Ok(data.to_bytes())
    }

    pub async fn delete(&self, path: &str) -> anyhow::Result<()> {
        self.operator.delete(path).await?;
        Ok(())
    }

    pub async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        match self.operator.stat(path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn content_length(&self, path: &str) -> anyhow::Result<u64> {
        let meta = self.operator.stat(path).await?;
        Ok(meta.content_length())
    }

    pub async fn delete_dir(&self, prefix: &str) -> anyhow::Result<()> {
        let entries: Vec<_> = self.operator.list(prefix).await?;
        for entry in entries {
            self.operator.delete(entry.path()).await?;
        }
        Ok(())
    }

    pub fn bucket(&self) -> &str {
        &self.root
    }

    pub fn operator(&self) -> &Operator {
        &self.operator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_path() {
        let ws_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let doc_id = uuid::Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        let path = StorageService::document_path(&ws_id, &doc_id, "test.pdf");
        assert_eq!(
            path,
            "workspaces/550e8400-e29b-41d4-a716-446655440000/documents/6ba7b810-9dad-11d1-80b4-00c04fd430c8/test.pdf"
        );
    }

    #[test]
    fn test_markdown_path() {
        let ws_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let doc_id = uuid::Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        let path = StorageService::markdown_path(&ws_id, &doc_id);
        assert_eq!(
            path,
            "workspaces/550e8400-e29b-41d4-a716-446655440000/documents/6ba7b810-9dad-11d1-80b4-00c04fd430c8/markdown.md"
        );
    }
}
