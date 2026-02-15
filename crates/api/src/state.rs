use std::sync::Arc;

use gotong_domain::idempotency::{IdempotencyConfig, IdempotencyService};
use gotong_domain::ports::idempotency::IdempotencyStore;
use gotong_domain::ports::{
    contributions::ContributionRepository, evidence::EvidenceRepository, vouches::VouchRepository,
};
use gotong_infra::config::AppConfig;
use gotong_infra::idempotency::RedisIdempotencyStore;
use gotong_infra::repositories::{
    InMemoryContributionRepository, InMemoryEvidenceRepository, InMemoryVouchRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub idempotency: IdempotencyService,
    pub contribution_repo: Arc<dyn ContributionRepository>,
    pub evidence_repo: Arc<dyn EvidenceRepository>,
    pub vouch_repo: Arc<dyn VouchRepository>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let store = RedisIdempotencyStore::connect(&config.redis_url).await?;
        let contribution_repo = Arc::new(InMemoryContributionRepository::new());
        let evidence_repo = Arc::new(InMemoryEvidenceRepository::new());
        let vouch_repo = Arc::new(InMemoryVouchRepository::new());
        let idempotency = IdempotencyService::new(Arc::new(store), IdempotencyConfig::default());
        Ok(Self {
            config,
            idempotency,
            contribution_repo,
            evidence_repo,
            vouch_repo,
        })
    }

    #[allow(dead_code)]
    pub fn with_idempotency_store(config: AppConfig, store: Arc<dyn IdempotencyStore>) -> Self {
        let idempotency = IdempotencyService::new(store, IdempotencyConfig::default());
        let contribution_repo = Arc::new(InMemoryContributionRepository::new());
        let evidence_repo = Arc::new(InMemoryEvidenceRepository::new());
        let vouch_repo = Arc::new(InMemoryVouchRepository::new());
        Self {
            config,
            idempotency,
            contribution_repo,
            evidence_repo,
            vouch_repo,
        }
    }

    #[allow(dead_code)]
    pub fn with_repositories(
        config: AppConfig,
        store: Arc<dyn IdempotencyStore>,
        contribution_repo: Arc<dyn ContributionRepository>,
        evidence_repo: Arc<dyn EvidenceRepository>,
        vouch_repo: Arc<dyn VouchRepository>,
    ) -> Self {
        let idempotency = IdempotencyService::new(store, IdempotencyConfig::default());
        Self {
            config,
            idempotency,
            contribution_repo,
            evidence_repo,
            vouch_repo,
        }
    }
}
