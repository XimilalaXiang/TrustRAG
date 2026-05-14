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

fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "trustrag_backend=debug,tower_http=info,sqlx=warn".into());

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json().with_target(true).with_thread_ids(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .init();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "TrustRAG backend starting"
    );

    let config = config::AppConfig::load()?;
    tracing::info!(listen_addr = %config.listen_addr, "Configuration loaded");

    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool established");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations completed");

    let storage = StorageService::new(&config)?;
    tracing::info!(bucket = %storage.bucket(), "Storage service initialized");

    let upload_limit = config.max_upload_size_mb * 1024 * 1024;

    let state = AppState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
        storage,
        max_upload_size: config.max_upload_size_mb,
        embedding_provider: None,
        doc_processor_url: config.doc_processor_url.clone(),
    };

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            let request_id = uuid::Uuid::new_v4().to_string();
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %request_id,
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                tracing::info!(
                    http.status = response.status().as_u16(),
                    latency_ms = latency.as_millis() as u64,
                    "Response sent"
                );
            },
        )
        .on_failure(
            |error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, _span: &tracing::Span| {
                tracing::error!(
                    error = %error,
                    latency_ms = latency.as_millis() as u64,
                    "Request failed"
                );
            },
        );

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(api::users::router())
        .merge(api::workspaces::router())
        .merge(api::documents::router())
        .merge(api::search::router())
        .merge(api::models::router())
        .merge(api::chat::router())
        .merge(api::citations::router())
        .with_state(state)
        .layer(axum::Extension(JwtSecret(config.jwt_secret.clone())))
        .layer(DefaultBodyLimit::max(upload_limit as usize))
        .layer(CorsLayer::permissive())
        .layer(trace_layer);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(
        listen_addr = %config.listen_addr,
        "TrustRAG backend ready and listening"
    );
    axum::serve(listener, app).await?;

    Ok(())
}
