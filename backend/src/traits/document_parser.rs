use async_trait::async_trait;

#[async_trait]
pub trait DocumentParser: Send + Sync {
    async fn parse(&self, file_bytes: &[u8], file_type: &str) -> anyhow::Result<String>;
}
