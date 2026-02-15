use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_env: String,
    pub port: u16,
    pub log_level: String,
    pub surreal_endpoint: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder()
            .set_default("app_env", "development")?
            .set_default("port", 3000)?
            .set_default("log_level", "info")?
            .set_default("surreal_endpoint", "ws://127.0.0.1:8000")?
            .set_default("surreal_ns", "gotong")?
            .set_default("surreal_db", "chat")?
            .set_default("surreal_user", "root")?
            .set_default("surreal_pass", "root")?
            .set_default("redis_url", "redis://127.0.0.1:6379")?
            .set_default("jwt_secret", "dev-secret")?
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        cfg.try_deserialize()
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }
}
