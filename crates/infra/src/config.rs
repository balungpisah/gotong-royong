use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_env: String,
    pub port: u16,
    pub log_level: String,
    pub surreal_endpoint: String,
    pub data_backend: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub worker_queue_prefix: String,
    pub worker_poll_interval_ms: u64,
    pub worker_promote_batch: usize,
    pub worker_backoff_base_ms: u64,
    pub worker_backoff_max_ms: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder()
            .set_default("app_env", "development")?
            .set_default("port", 3000)?
            .set_default("log_level", "info")?
            .set_default("surreal_endpoint", "ws://127.0.0.1:8000")?
            .set_default("data_backend", "memory")?
            .set_default("surreal_ns", "gotong")?
            .set_default("surreal_db", "chat")?
            .set_default("surreal_user", "root")?
            .set_default("surreal_pass", "root")?
            .set_default("redis_url", "redis://127.0.0.1:6379")?
            .set_default("jwt_secret", "dev-secret")?
            .set_default("s3_endpoint", "http://127.0.0.1:9000")?
            .set_default("s3_bucket", "gotong-royong-evidence-dev")?
            .set_default("s3_region", "us-east-1")?
            .set_default("s3_access_key", "minioadmin")?
            .set_default("s3_secret_key", "minioadmin")?
            .set_default("worker_queue_prefix", "gotong:jobs")?
            .set_default("worker_poll_interval_ms", 1000)?
            .set_default("worker_promote_batch", 50)?
            .set_default("worker_backoff_base_ms", 1000)?
            .set_default("worker_backoff_max_ms", 60000)?
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        cfg.try_deserialize()
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }
}
