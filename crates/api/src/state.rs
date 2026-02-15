use std::sync::Arc;

use gotong_domain::idempotency::{IdempotencyConfig, IdempotencyService};
use gotong_domain::ports::idempotency::IdempotencyStore;
use gotong_infra::config::AppConfig;
use gotong_infra::idempotency::RedisIdempotencyStore;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub idempotency: IdempotencyService,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let store = RedisIdempotencyStore::connect(&config.redis_url).await?;
        let idempotency = IdempotencyService::new(Arc::new(store), IdempotencyConfig::default());
        Ok(Self {
            config,
            idempotency,
        })
    }

    #[allow(dead_code)]
    pub fn with_idempotency_store(config: AppConfig, store: Arc<dyn IdempotencyStore>) -> Self {
        let idempotency = IdempotencyService::new(store, IdempotencyConfig::default());
        Self {
            config,
            idempotency,
        }
    }
}
