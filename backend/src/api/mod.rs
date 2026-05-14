use sqlx::PgPool;

use crate::services::storage::StorageService;

pub mod users;
pub mod workspaces;
pub mod documents;
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
}
