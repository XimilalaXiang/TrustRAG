use sqlx::PgPool;

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
}
