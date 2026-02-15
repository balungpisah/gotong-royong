use std::collections::HashMap;
use std::sync::Arc;

use gotong_domain::chat::ChatMessage;
use gotong_domain::idempotency::{IdempotencyConfig, IdempotencyService};
use gotong_domain::ports::idempotency::IdempotencyStore;
use gotong_domain::ports::{
    chat::ChatRepository, contributions::ContributionRepository, evidence::EvidenceRepository,
    jobs::JobQueue, siaga::SiagaRepository, transitions::TrackTransitionRepository,
    vault::VaultRepository, vouches::VouchRepository,
};
use gotong_infra::config::AppConfig;
use gotong_infra::db::DbConfig;
use gotong_infra::idempotency::RedisIdempotencyStore;
use gotong_infra::jobs::RedisJobQueue;
use gotong_infra::repositories::{
    InMemoryChatRepository, InMemoryContributionRepository, InMemoryEvidenceRepository,
    InMemoryModerationRepository, InMemorySiagaRepository, InMemoryTrackTransitionRepository,
    InMemoryVaultRepository, InMemoryVouchRepository, SurrealChatRepository,
    SurrealModerationRepository, SurrealSiagaRepository, SurrealTrackTransitionRepository,
    SurrealVaultRepository,
};
use tokio::sync::{RwLock, broadcast};

type RepositoryBundle = (
    Arc<dyn ContributionRepository>,
    Arc<dyn EvidenceRepository>,
    Arc<dyn VouchRepository>,
    Arc<dyn TrackTransitionRepository>,
    Arc<dyn VaultRepository>,
    Arc<dyn ChatRepository>,
    Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
    Arc<dyn SiagaRepository>,
);
type TransitionJobQueue = Option<Arc<dyn JobQueue>>;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub idempotency: IdempotencyService,
    pub contribution_repo: Arc<dyn ContributionRepository>,
    pub evidence_repo: Arc<dyn EvidenceRepository>,
    pub vouch_repo: Arc<dyn VouchRepository>,
    pub transition_repo: Arc<dyn TrackTransitionRepository>,
    pub vault_repo: Arc<dyn VaultRepository>,
    pub chat_repo: Arc<dyn ChatRepository>,
    pub moderation_repo: Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
    #[allow(dead_code)]
    pub siaga_repo: Arc<dyn SiagaRepository>,
    pub chat_realtime: ChatRealtimeBus,
    pub transition_job_queue: TransitionJobQueue,
}

#[derive(Clone)]
pub struct ChatRealtimeBus {
    senders: Arc<RwLock<HashMap<String, broadcast::Sender<ChatMessage>>>>,
    buffer_size: usize,
}

impl ChatRealtimeBus {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(RwLock::new(HashMap::new())),
            buffer_size: 64,
        }
    }

    async fn sender_for(&self, thread_id: &str) -> broadcast::Sender<ChatMessage> {
        let mut senders = self.senders.write().await;
        if let Some(sender) = senders.get(thread_id) {
            return sender.clone();
        }
        let sender = broadcast::channel(self.buffer_size).0;
        senders.insert(thread_id.to_string(), sender.clone());
        sender
    }

    pub async fn publish(&self, thread_id: &str, message: ChatMessage) {
        let sender = self.sender_for(thread_id).await;
        if sender.send(message).is_err() {
            let mut senders = self.senders.write().await;
            senders.remove(thread_id);
        }
    }

    pub async fn subscribe(&self, thread_id: &str) -> broadcast::Receiver<ChatMessage> {
        self.sender_for(thread_id).await.subscribe()
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let store = RedisIdempotencyStore::connect(&config.redis_url).await?;
        let (
            contribution_repo,
            evidence_repo,
            vouch_repo,
            transition_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            siaga_repo,
        ) = repositories_for_config(&config).await?;
        let transition_job_queue = transition_job_queue_for_config(&config).await?;
        let idempotency = IdempotencyService::new(Arc::new(store), IdempotencyConfig::default());
        Ok(Self {
            config,
            idempotency,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            transition_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            siaga_repo,
            chat_realtime: ChatRealtimeBus::new(),
            transition_job_queue,
        })
    }

    #[allow(dead_code)]
    pub fn with_idempotency_store(config: AppConfig, store: Arc<dyn IdempotencyStore>) -> Self {
        let (
            contribution_repo,
            evidence_repo,
            vouch_repo,
            transition_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            siaga_repo,
        ) = memory_repositories();
        Self {
            config,
            idempotency: IdempotencyService::new(store, IdempotencyConfig::default()),
            contribution_repo,
            evidence_repo,
            vouch_repo,
            transition_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            siaga_repo,
            chat_realtime: ChatRealtimeBus::new(),
            transition_job_queue: None,
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn with_repositories(
        config: AppConfig,
        store: Arc<dyn IdempotencyStore>,
        contribution_repo: Arc<dyn ContributionRepository>,
        evidence_repo: Arc<dyn EvidenceRepository>,
        vouch_repo: Arc<dyn VouchRepository>,
        transition_repo: Arc<dyn TrackTransitionRepository>,
        vault_repo: Arc<dyn VaultRepository>,
        chat_repo: Arc<dyn ChatRepository>,
        moderation_repo: Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
        siaga_repo: Arc<dyn SiagaRepository>,
    ) -> Self {
        let idempotency = IdempotencyService::new(store, IdempotencyConfig::default());
        Self {
            config,
            idempotency,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            transition_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            siaga_repo,
            chat_realtime: ChatRealtimeBus::new(),
            transition_job_queue: None,
        }
    }
}

async fn repositories_for_config(config: &AppConfig) -> anyhow::Result<RepositoryBundle> {
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
            let db_config = DbConfig::from_app_config(config);
            let transition_repo = SurrealTrackTransitionRepository::new(&db_config).await?;
            let vault_repo = SurrealVaultRepository::new(&db_config).await?;
            let chat_repo = SurrealChatRepository::new(&db_config).await?;
            let moderation_repo = SurrealModerationRepository::new(&db_config).await?;
            let siaga_repo = SurrealSiagaRepository::new(&db_config).await?;
            Ok((
                Arc::new(InMemoryContributionRepository::new()),
                Arc::new(InMemoryEvidenceRepository::new()),
                Arc::new(InMemoryVouchRepository::new()),
                Arc::new(transition_repo),
                Arc::new(vault_repo),
                Arc::new(chat_repo),
                Arc::new(moderation_repo),
                Arc::new(siaga_repo),
            ))
        }
        _ => anyhow::bail!("unsupported DATA_BACKEND '{}'", config.data_backend),
    }
}

fn memory_repositories() -> RepositoryBundle {
    (
        Arc::new(InMemoryContributionRepository::new()),
        Arc::new(InMemoryEvidenceRepository::new()),
        Arc::new(InMemoryVouchRepository::new()),
        Arc::new(InMemoryTrackTransitionRepository::new()),
        Arc::new(InMemoryVaultRepository::new()),
        Arc::new(InMemoryChatRepository::new()),
        Arc::new(InMemoryModerationRepository::new()),
        Arc::new(InMemorySiagaRepository::new()),
    )
}

async fn transition_job_queue_for_config(config: &AppConfig) -> anyhow::Result<TransitionJobQueue> {
    if config.app_env.eq_ignore_ascii_case("test") {
        return Ok(None);
    }

    let backend = config.data_backend.trim().to_ascii_lowercase();
    if matches!(
        backend.as_str(),
        "surreal" | "surrealdb" | "tikv" | "memory" | "mem" | "in-memory" | "in_memory"
    ) {
        let queue = RedisJobQueue::connect_with_prefix(
            &config.redis_url,
            config.worker_queue_prefix.clone(),
        )
        .await?;
        return Ok(Some(Arc::new(queue)));
    }

    anyhow::bail!("unsupported DATA_BACKEND '{}'", config.data_backend)
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

    #[tokio::test]
    async fn memory_backend_rejected_in_production() {
        let config = app_config("production", "memory");
        assert!(repositories_for_config(&config).await.is_err());
    }

    #[tokio::test]
    async fn unknown_backend_is_rejected() {
        let config = app_config("development", "nonsense");
        assert!(repositories_for_config(&config).await.is_err());
    }

    #[tokio::test]
    async fn memory_backend_allows_local_and_test() {
        let dev_config = app_config("development", "memory");
        let test_config = app_config("test", "memory");
        assert!(repositories_for_config(&dev_config).await.is_ok());
        assert!(repositories_for_config(&test_config).await.is_ok());
    }
}
