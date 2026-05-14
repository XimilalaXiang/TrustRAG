use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub messages: Vec<LlmMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub model: String,
}

/// Streamed chunk from LLM
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Delta(String),
    Done(LlmResponse),
    Error(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, request: &LlmRequest) -> anyhow::Result<LlmResponse>;

    async fn stream(
        &self,
        request: &LlmRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> anyhow::Result<()>;

    fn model_name(&self) -> &str;
}
