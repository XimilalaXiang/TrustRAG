use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub listen_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,
    pub minio_region: String,
    pub jwt_secret: String,
    pub doc_processor_url: String,
    pub max_upload_size_mb: u64,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("TRUSTRAG").separator("__"))
            .set_default("listen_addr", "0.0.0.0:8080")?
            .set_default("database_url", "postgres://trustrag:trustrag@localhost:5432/trustrag")?
            .set_default("redis_url", "redis://localhost:6379")?
            .set_default("minio_endpoint", "http://localhost:9000")?
            .set_default("minio_access_key", "minioadmin")?
            .set_default("minio_secret_key", "minioadmin")?
            .set_default("minio_bucket", "trustrag")?
            .set_default("minio_region", "us-east-1")?
            .set_default("jwt_secret", "change-me-in-production")?
            .set_default("doc_processor_url", "http://localhost:8081")?
            .set_default("max_upload_size_mb", 100i64)?
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
