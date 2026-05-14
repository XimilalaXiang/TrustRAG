use axum::{Router, routing::get};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod auth;
mod api;
mod db;

use api::AppState;
use auth::middleware::JwtSecret;

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

    let state = AppState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(api::users::router())
        .merge(api::workspaces::router())
        .with_state(state)
        .layer(axum::Extension(JwtSecret(config.jwt_secret.clone())))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("TrustRAG backend listening on {}", config.listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
