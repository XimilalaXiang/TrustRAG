use std::sync::Arc;

use sqlx::PgPool;

use crate::services::storage::StorageService;
use crate::traits::embedding_provider::EmbeddingProvider;

pub mod users;
pub mod workspaces;
pub mod documents;
pub mod search;
pub mod models;
pub mod chat;
pub mod citations;
pub mod reviews;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub storage: StorageService,
    pub max_upload_size: u64,
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    pub doc_processor_url: String,
}
