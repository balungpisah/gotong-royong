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
    pub chat_realtime_transport: String,
    pub chat_realtime_channel_prefix: String,
    pub worker_queue_prefix: String,
    pub worker_poll_interval_ms: u64,
    pub worker_promote_batch: usize,
    pub worker_backoff_base_ms: u64,
    pub worker_backoff_max_ms: u64,
    pub worker_ttl_cleanup_interval_ms: u64,
    pub worker_concept_verification_interval_ms: u64,
    pub webhook_enabled: bool,
    pub webhook_markov_url: String,
    pub webhook_secret: String,
    pub webhook_max_attempts: u32,
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
            .set_default("chat_realtime_transport", "local")?
            .set_default("chat_realtime_channel_prefix", "gotong:chat:realtime")?
            .set_default("worker_queue_prefix", "gotong:jobs")?
            .set_default("worker_poll_interval_ms", 1000)?
            .set_default("worker_promote_batch", 50)?
            .set_default("worker_backoff_base_ms", 1000)?
            .set_default("worker_backoff_max_ms", 60000)?
            .set_default("worker_ttl_cleanup_interval_ms", 3_600_000)?
            .set_default("worker_concept_verification_interval_ms", 86_400_000)?
            .set_default("webhook_enabled", false)?
            .set_default(
                "webhook_markov_url",
                "https://api.markov.local/v1/platforms/gotong_royong/webhook",
            )?
            .set_default("webhook_secret", "dev_webhook_secret_32_chars_minimum")?
            .set_default("webhook_max_attempts", 5u32)?
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        let config = cfg.try_deserialize::<AppConfig>()?;
        if config.webhook_enabled && config.webhook_markov_url.trim().is_empty() {
            return Err(config::ConfigError::Message(
                "webhook is enabled but webhook_markov_url is empty".to_string(),
            ));
        }
        if config.webhook_enabled && config.webhook_secret.trim().len() < 4 {
            return Err(config::ConfigError::Message(
                "webhook is enabled but webhook_secret is too short".to_string(),
            ));
        }
        Ok(config)
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }
}
