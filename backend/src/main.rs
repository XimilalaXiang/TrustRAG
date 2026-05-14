use axum::{extract::DefaultBodyLimit, Router, routing::get};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod auth;
mod api;
mod db;
mod services;
mod traits;

use api::AppState;
use auth::middleware::JwtSecret;
use services::storage::StorageService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "trustrag_backend=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::load()?;
    let pool = db::create_pool(&config.database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let storage = StorageService::new(&config)?;
    tracing::info!("Storage service initialized (bucket: {})", storage.bucket());

    let upload_limit = config.max_upload_size_mb * 1024 * 1024;

    let state = AppState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
        storage,
        max_upload_size: config.max_upload_size_mb,
        embedding_provider: None, // configured via model settings at runtime
        doc_processor_url: config.doc_processor_url.clone(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(api::users::router())
        .merge(api::workspaces::router())
        .merge(api::documents::router())
        .merge(api::search::router())
        .with_state(state)
        .layer(axum::Extension(JwtSecret(config.jwt_secret.clone())))
        .layer(DefaultBodyLimit::max(upload_limit as usize))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("TrustRAG backend listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
