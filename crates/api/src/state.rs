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

type RepositoryBundle = (
    Arc<dyn ContributionRepository>,
    Arc<dyn EvidenceRepository>,
    Arc<dyn VouchRepository>,
);

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
        let (contribution_repo, evidence_repo, vouch_repo) = repositories_for_config(&config)?;
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
        let (contribution_repo, evidence_repo, vouch_repo) = memory_repositories();
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

fn repositories_for_config(config: &AppConfig) -> anyhow::Result<RepositoryBundle> {
    let backend = config.data_backend.trim().to_ascii_lowercase();
    match backend.as_str() {
        "memory" | "mem" | "in-memory" | "in_memory" => {
            if config.is_production() {
                anyhow::bail!(
                    "in-memory repositories are not allowed in production; configure a persistent backend"
                );
            }
            Ok(memory_repositories())
        }
        "surreal" | "surrealdb" | "tikv" => {
            anyhow::bail!(
                "surreal-backed repositories are planned but not wired in this PR. set DATA_BACKEND=memory for local/test"
            );
        }
        _ => anyhow::bail!("unsupported DATA_BACKEND '{}'", config.data_backend),
    }
}

fn memory_repositories() -> RepositoryBundle {
    (
        Arc::new(InMemoryContributionRepository::new()),
        Arc::new(InMemoryEvidenceRepository::new()),
        Arc::new(InMemoryVouchRepository::new()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_config(app_env: &str, data_backend: &str) -> AppConfig {
        AppConfig {
            app_env: app_env.to_string(),
            port: 3000,
            log_level: "info".to_string(),
            surreal_endpoint: "ws://127.0.0.1:8000".to_string(),
            data_backend: data_backend.to_string(),
            surreal_ns: "gotong".to_string(),
            surreal_db: "chat".to_string(),
            surreal_user: "root".to_string(),
            surreal_pass: "root".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            jwt_secret: "test-secret".to_string(),
            s3_endpoint: "http://127.0.0.1:9000".to_string(),
            s3_bucket: "gotong-royong-evidence-test".to_string(),
            s3_region: "us-east-1".to_string(),
            s3_access_key: "test-access-key".to_string(),
            s3_secret_key: "test-secret-key".to_string(),
            worker_queue_prefix: "gotong:jobs".to_string(),
            worker_poll_interval_ms: 1000,
            worker_promote_batch: 10,
            worker_backoff_base_ms: 1000,
            worker_backoff_max_ms: 60000,
        }
    }

    #[test]
    fn memory_backend_rejected_in_production() {
        let config = app_config("production", "memory");
        assert!(repositories_for_config(&config).is_err());
    }

    #[test]
    fn unknown_backend_is_rejected() {
        let config = app_config("development", "nonsense");
        assert!(repositories_for_config(&config).is_err());
    }

    #[test]
    fn memory_backend_allows_local_and_test() {
        let dev_config = app_config("development", "memory");
        let test_config = app_config("test", "memory");
        assert!(repositories_for_config(&dev_config).is_ok());
        assert!(repositories_for_config(&test_config).is_ok());
    }
}
