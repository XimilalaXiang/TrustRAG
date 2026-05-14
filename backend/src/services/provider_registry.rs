use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::traits::embedding_provider::EmbeddingProvider;
use crate::traits::llm_provider::LlmProvider;

#[derive(Clone, Debug)]
pub struct ProviderInfo {
    pub id: String,
    pub provider_type: ProviderType,
    pub display_name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProviderType {
    Llm,
    Embedding,
}

pub struct ProviderRegistry {
    llm_providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
    embedding_providers: RwLock<HashMap<String, Arc<dyn EmbeddingProvider>>>,
    provider_info: RwLock<Vec<ProviderInfo>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            llm_providers: RwLock::new(HashMap::new()),
            embedding_providers: RwLock::new(HashMap::new()),
            provider_info: RwLock::new(Vec::new()),
        }
    }

    pub async fn register_llm(
        &self,
        id: &str,
        display_name: &str,
        description: &str,
        provider: Arc<dyn LlmProvider>,
    ) {
        self.llm_providers
            .write()
            .await
            .insert(id.to_string(), provider);
        let mut info = self.provider_info.write().await;
        info.retain(|p| !(p.id == id && p.provider_type == ProviderType::Llm));
        info.push(ProviderInfo {
            id: id.to_string(),
            provider_type: ProviderType::Llm,
            display_name: display_name.to_string(),
            description: description.to_string(),
        });
        tracing::info!(provider_id = id, kind = "llm", "Provider registered");
    }

    pub async fn register_embedding(
        &self,
        id: &str,
        display_name: &str,
        description: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) {
        self.embedding_providers
            .write()
            .await
            .insert(id.to_string(), provider);
        let mut info = self.provider_info.write().await;
        info.retain(|p| !(p.id == id && p.provider_type == ProviderType::Embedding));
        info.push(ProviderInfo {
            id: id.to_string(),
            provider_type: ProviderType::Embedding,
            display_name: display_name.to_string(),
            description: description.to_string(),
        });
        tracing::info!(provider_id = id, kind = "embedding", "Provider registered");
    }

    pub async fn get_llm(&self, id: &str) -> Option<Arc<dyn LlmProvider>> {
        self.llm_providers.read().await.get(id).cloned()
    }

    pub async fn get_embedding(&self, id: &str) -> Option<Arc<dyn EmbeddingProvider>> {
        self.embedding_providers.read().await.get(id).cloned()
    }

    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        self.provider_info.read().await.clone()
    }

    pub async fn unregister(&self, id: &str) {
        self.llm_providers.write().await.remove(id);
        self.embedding_providers.write().await.remove(id);
        self.provider_info.write().await.retain(|p| p.id != id);
        tracing::info!(provider_id = id, "Provider unregistered");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_new_empty() {
        let reg = ProviderRegistry::new();
        assert!(reg.list_providers().await.is_empty());
        assert!(reg.get_llm("nonexistent").await.is_none());
        assert!(reg.get_embedding("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let reg = ProviderRegistry::new();
        reg.provider_info.write().await.push(ProviderInfo {
            id: "test".to_string(),
            provider_type: ProviderType::Llm,
            display_name: "Test".to_string(),
            description: "Test provider".to_string(),
        });
        assert_eq!(reg.list_providers().await.len(), 1);
        reg.unregister("test").await;
        assert!(reg.list_providers().await.is_empty());
    }
}
