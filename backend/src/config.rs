use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub listen_addr: String,
    pub database_url: String,
    #[cfg(feature = "postgres")]
    pub redis_url: String,
    #[cfg(feature = "postgres")]
    pub minio_endpoint: String,
    #[cfg(feature = "postgres")]
    pub minio_access_key: String,
    #[cfg(feature = "postgres")]
    pub minio_secret_key: String,
    #[cfg(feature = "postgres")]
    pub minio_bucket: String,
    #[cfg(feature = "postgres")]
    pub minio_region: String,
    #[cfg(feature = "desktop")]
    pub data_dir: String,
    pub jwt_secret: String,
    pub doc_processor_url: String,
    pub max_upload_size_mb: u64,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("TRUSTRAG").separator("__"))
            .set_default("listen_addr", "0.0.0.0:8080")?
            .set_default("jwt_secret", "change-me-in-production")?
            .set_default("doc_processor_url", "http://localhost:8081")?
            .set_default("max_upload_size_mb", 100i64)?;

        #[cfg(feature = "postgres")]
        let builder = builder
            .set_default("database_url", "postgres://trustrag:trustrag@localhost:5432/trustrag")?
            .set_default("redis_url", "redis://localhost:6379")?
            .set_default("minio_endpoint", "http://localhost:9000")?
            .set_default("minio_access_key", "minioadmin")?
            .set_default("minio_secret_key", "minioadmin")?
            .set_default("minio_bucket", "trustrag")?
            .set_default("minio_region", "us-east-1")?;

        #[cfg(feature = "desktop")]
        let builder = {
            let data_dir = Self::default_data_dir();
            builder
                .set_default("database_url", format!("sqlite://{}/trustrag.db?mode=rwc", data_dir))?
                .set_default("data_dir", data_dir)?
        };

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }

    #[cfg(feature = "desktop")]
    fn default_data_dir() -> String {
        directories::ProjectDirs::from("com", "trustrag", "TrustRAG")
            .map(|dirs| dirs.data_dir().to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                format!("{}/.trustrag", home)
            })
    }
}
