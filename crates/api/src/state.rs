use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::observability;
use futures_util::StreamExt;
use gotong_domain::chat::ChatMessage;
use gotong_domain::idempotency::{IdempotencyConfig, IdempotencyService};
use gotong_domain::ports::idempotency::IdempotencyStore;
use gotong_domain::ports::{
    adaptive_path::AdaptivePathRepository,
    chat::ChatRepository,
    contributions::ContributionRepository,
    discovery::{FeedRepository, NotificationRepository},
    evidence::EvidenceRepository,
    jobs::JobQueue,
    ontology::OntologyRepository,
    siaga::SiagaRepository,
    vault::VaultRepository,
    vouches::VouchRepository,
    webhook::WebhookOutboxRepository,
};
use gotong_domain::util::uuid_v7_without_dashes;
use gotong_infra::config::AppConfig;
use gotong_infra::db::DbConfig;
use gotong_infra::idempotency::RedisIdempotencyStore;
use gotong_infra::jobs::RedisJobQueue;
use gotong_infra::repositories::{
    InMemoryAdaptivePathRepository, InMemoryChatRepository, InMemoryContributionRepository,
    InMemoryDiscoveryFeedRepository, InMemoryDiscoveryNotificationRepository,
    InMemoryEvidenceRepository, InMemoryModerationRepository, InMemoryOntologyRepository,
    InMemorySiagaRepository, InMemoryVaultRepository, InMemoryVouchRepository,
    InMemoryWebhookOutboxRepository, SurrealAdaptivePathRepository, SurrealChatRepository,
    SurrealContributionRepository, SurrealDiscoveryFeedRepository,
    SurrealDiscoveryNotificationRepository, SurrealEvidenceRepository, SurrealModerationRepository,
    SurrealOntologyRepository, SurrealSiagaRepository, SurrealVaultRepository,
    SurrealVouchRepository, SurrealWebhookOutboxRepository,
};
use redis::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use tracing::warn;

type RepositoryBundle = (
    Arc<dyn AdaptivePathRepository>,
    Arc<dyn ContributionRepository>,
    Arc<dyn EvidenceRepository>,
    Arc<dyn VouchRepository>,
    Arc<dyn VaultRepository>,
    Arc<dyn ChatRepository>,
    Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
    Arc<dyn OntologyRepository>,
    Arc<dyn SiagaRepository>,
    Arc<dyn FeedRepository>,
    Arc<dyn NotificationRepository>,
    Arc<dyn WebhookOutboxRepository>,
);
type SharedJobQueue = Option<Arc<dyn JobQueue>>;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub idempotency: IdempotencyService,
    pub adaptive_path_repo: Arc<dyn AdaptivePathRepository>,
    pub contribution_repo: Arc<dyn ContributionRepository>,
    pub evidence_repo: Arc<dyn EvidenceRepository>,
    pub vouch_repo: Arc<dyn VouchRepository>,
    pub vault_repo: Arc<dyn VaultRepository>,
    pub chat_repo: Arc<dyn ChatRepository>,
    pub moderation_repo: Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
    pub ontology_repo: Arc<dyn OntologyRepository>,
    #[allow(dead_code)]
    pub siaga_repo: Arc<dyn SiagaRepository>,
    pub feed_repo: Arc<dyn FeedRepository>,
    pub notification_repo: Arc<dyn NotificationRepository>,
    pub webhook_outbox_repo: Arc<dyn WebhookOutboxRepository>,
    pub chat_realtime: ChatRealtimeBus,
    pub job_queue: SharedJobQueue,
}

#[derive(Clone)]
pub struct ChatRealtimeBus {
    senders: Arc<RwLock<HashMap<String, broadcast::Sender<ChatMessage>>>>,
    active_bridges: Arc<RwLock<HashSet<String>>>,
    buffer_size: usize,
    transport: ChatRealtimeTransport,
    instance_id: String,
}

#[derive(Clone)]
enum ChatRealtimeTransport {
    Local,
    Redis {
        channel_prefix: String,
        client: Option<Client>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
struct ChatRealtimeEnvelope {
    thread_id: String,
    sender_id: String,
    message: ChatMessage,
}

impl ChatRealtimeBus {
    pub fn new(config: &gotong_infra::config::AppConfig) -> Self {
        let transport = config.chat_realtime_transport.trim().to_ascii_lowercase();
        let transport = transport.as_str();
        let transport = match transport {
            "local" => ChatRealtimeTransport::Local,
            "redis" => {
                let redis_url = config.redis_url.clone();
                let client = Client::open(redis_url.clone()).ok();
                if client.is_none() {
                    observability::register_chat_realtime_bridge_event(
                        "transport_init_fallback",
                        "redis",
                        "invalid_url",
                    );
                    warn!(
                        redis_url = %redis_url,
                        "invalid redis url for redis realtime transport; using local transport fallback"
                    );
                    ChatRealtimeTransport::Local
                } else {
                    ChatRealtimeTransport::Redis {
                        channel_prefix: config.chat_realtime_channel_prefix.clone(),
                        client,
                    }
                }
            }
            other => {
                warn!(
                    transport = %other,
                    "unsupported CHAT_REALTIME_TRANSPORT value; falling back to local transport"
                );
                observability::register_chat_realtime_bridge_event(
                    "transport_init_fallback",
                    "unsupported",
                    other,
                );
                ChatRealtimeTransport::Local
            }
        };

        Self {
            senders: Arc::new(RwLock::new(HashMap::new())),
            buffer_size: 64,
            active_bridges: Arc::new(RwLock::new(HashSet::new())),
            transport,
            instance_id: uuid_v7_without_dashes(),
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

    fn channel_name(&self, thread_id: &str) -> Option<String> {
        match &self.transport {
            ChatRealtimeTransport::Redis { channel_prefix, .. } => {
                Some(format!("{channel_prefix}:{thread_id}"))
            }
            ChatRealtimeTransport::Local => None,
        }
    }

    fn channel_redis_client(&self) -> Option<Client> {
        match &self.transport {
            ChatRealtimeTransport::Redis { client, .. } => client.clone(),
            ChatRealtimeTransport::Local => None,
        }
    }

    async fn publish_to_redis(&self, envelope: &ChatRealtimeEnvelope) {
        let Some(redis_conn) = self.channel_redis_client() else {
            return;
        };
        let Some(channel) = self.channel_name(&envelope.thread_id) else {
            return;
        };

        let serialized = match serde_json::to_string(envelope) {
            Ok(value) => value,
            Err(err) => {
                warn!(error = %err, "chat realtime envelope serialization failed");
                return;
            }
        };

        let mut redis_conn = match redis_conn.get_multiplexed_async_connection().await {
            Ok(connection) => connection,
            Err(err) => {
                observability::register_chat_realtime_bridge_event(
                    "publish_connection_failed",
                    "redis",
                    "connection",
                );
                warn!(error = %err, "chat realtime redis connection failed");
                return;
            }
        };

        if let Err(err) = redis::cmd("PUBLISH")
            .arg(channel)
            .arg(serialized)
            .query_async::<_, i64>(&mut redis_conn)
            .await
        {
            observability::register_chat_realtime_bridge_event(
                "publish_command_failed",
                "redis",
                "publish",
            );
            warn!(error = %err, "chat realtime redis publish failed");
        }
    }

    async fn spawn_redis_bridge(&self, thread_id: String) {
        let (channel, client) = match &self.transport {
            ChatRealtimeTransport::Redis {
                client,
                channel_prefix,
                ..
            } if client.is_some() => {
                let Some(url_client) = client.clone() else {
                    return;
                };
                (format!("{channel_prefix}:{thread_id}"), url_client)
            }
            _ => return,
        };

        let sender_map = self.senders.clone();
        let local_instance = self.instance_id.clone();
        tokio::spawn(async move {
            use tokio::time::{Duration, sleep};

            let mut backoff_ms = 250_u64;
            let max_backoff_ms = 5_000_u64;

            loop {
                let mut pubsub = match client.clone().get_async_pubsub().await {
                    Ok(pubsub) => pubsub,
                    Err(err) => {
                        observability::register_chat_realtime_bridge_event(
                            "subscription_connect_failed",
                            "redis",
                            "connect",
                        );
                        warn!(error = %err, "chat realtime redis subscription failed");
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms = (backoff_ms * 2).min(max_backoff_ms);
                        continue;
                    }
                };
                backoff_ms = 250_u64;
                if let Err(err) = pubsub.subscribe(channel.clone()).await {
                    observability::register_chat_realtime_bridge_event(
                        "subscription_subscribe_failed",
                        "redis",
                        "subscribe",
                    );
                    warn!(error = %err, "chat realtime redis channel subscribe failed");
                    sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(max_backoff_ms);
                    continue;
                }

                let mut stream = pubsub.on_message();
                loop {
                    match stream.next().await {
                        Some(message) => {
                            let payload: String = match message.get_payload() {
                                Ok(payload) => payload,
                                Err(err) => {
                                    observability::register_chat_realtime_bridge_event(
                                        "message_payload_decode_failed",
                                        "redis",
                                        "decode",
                                    );
                                    warn!(
                                        error = %err,
                                        "chat realtime redis payload decode failed"
                                    );
                                    continue;
                                }
                            };

                            let envelope: ChatRealtimeEnvelope =
                                match serde_json::from_str(&payload) {
                                    Ok(envelope) => envelope,
                                    Err(err) => {
                                        observability::register_chat_realtime_bridge_event(
                                            "message_payload_parse_failed",
                                            "redis",
                                            "parse",
                                        );
                                        warn!(error = %err, "chat realtime envelope parse failed");
                                        continue;
                                    }
                                };

                            if envelope.sender_id == local_instance {
                                continue;
                            }

                            let sender = {
                                let senders = sender_map.read().await;
                                senders.get(&envelope.thread_id).cloned()
                            };
                            let Some(sender) = sender else {
                                continue;
                            };
                            if sender.send(envelope.message).is_err() {
                                warn!(
                                    "chat realtime broadcast failed for thread {}",
                                    envelope.thread_id
                                );
                            }
                        }
                        None => {
                            observability::register_chat_realtime_bridge_event(
                                "stream_ended",
                                "redis",
                                "reconnect",
                            );
                            warn!("chat realtime redis stream ended");
                            break;
                        }
                    }
                }
                observability::register_chat_realtime_bridge_event(
                    "stream_reconnect_backoff",
                    "redis",
                    "reconnect",
                );
                warn!("chat realtime redis stream ended; reconnecting");
                sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(max_backoff_ms);
            }
        });
    }

    async fn ensure_redis_bridge(&self, thread_id: &str) {
        let mut active = self.active_bridges.write().await;
        if !active.insert(thread_id.to_string()) {
            return;
        }
        drop(active);
        self.spawn_redis_bridge(thread_id.to_string()).await;
    }

    pub async fn publish(&self, thread_id: &str, message: ChatMessage) {
        let message_for_redis = message.clone();
        let sender = self.sender_for(thread_id).await;
        if sender.send(message).is_err() {
            let mut senders = self.senders.write().await;
            senders.remove(thread_id);
            let mut active_bridges = self.active_bridges.write().await;
            active_bridges.remove(thread_id);
        }

        let envelope = ChatRealtimeEnvelope {
            thread_id: thread_id.to_string(),
            sender_id: self.instance_id.clone(),
            message: message_for_redis,
        };

        match &self.transport {
            ChatRealtimeTransport::Redis { .. } => {
                let bus = self.clone();
                let envelope = envelope.clone();
                tokio::spawn(async move {
                    bus.publish_to_redis(&envelope).await;
                });
            }
            ChatRealtimeTransport::Local => {}
        }
    }

    pub async fn subscribe(&self, thread_id: &str) -> broadcast::Receiver<ChatMessage> {
        if matches!(self.transport, ChatRealtimeTransport::Redis { .. }) {
            self.ensure_redis_bridge(thread_id).await;
        }
        self.sender_for(thread_id).await.subscribe()
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> anyhow::Result<Self> {
        let store = RedisIdempotencyStore::connect(&config.redis_url).await?;
        let (
            adaptive_path_repo,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            ontology_repo,
            siaga_repo,
            feed_repo,
            notification_repo,
            webhook_outbox_repo,
        ) = repositories_for_config(&config).await?;
        let job_queue = job_queue_for_config(&config).await?;
        let idempotency = IdempotencyService::new(Arc::new(store), IdempotencyConfig::default());
        let chat_realtime = ChatRealtimeBus::new(&config);
        Ok(Self {
            config,
            idempotency,
            adaptive_path_repo,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            ontology_repo,
            siaga_repo,
            feed_repo,
            notification_repo,
            webhook_outbox_repo,
            chat_realtime,
            job_queue,
        })
    }

    #[allow(dead_code)]
    pub fn with_idempotency_store(config: AppConfig, store: Arc<dyn IdempotencyStore>) -> Self {
        let (
            adaptive_path_repo,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            ontology_repo,
            siaga_repo,
            feed_repo,
            notification_repo,
            webhook_outbox_repo,
        ) = memory_repositories();
        let chat_realtime = ChatRealtimeBus::new(&config);
        Self {
            config,
            idempotency: IdempotencyService::new(store, IdempotencyConfig::default()),
            adaptive_path_repo,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            ontology_repo,
            siaga_repo,
            feed_repo,
            notification_repo,
            webhook_outbox_repo,
            chat_realtime,
            job_queue: None,
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn with_repositories(
        config: AppConfig,
        store: Arc<dyn IdempotencyStore>,
        adaptive_path_repo: Arc<dyn AdaptivePathRepository>,
        contribution_repo: Arc<dyn ContributionRepository>,
        evidence_repo: Arc<dyn EvidenceRepository>,
        vouch_repo: Arc<dyn VouchRepository>,
        vault_repo: Arc<dyn VaultRepository>,
        chat_repo: Arc<dyn ChatRepository>,
        moderation_repo: Arc<dyn gotong_domain::ports::moderation::ModerationRepository>,
        ontology_repo: Arc<dyn OntologyRepository>,
        siaga_repo: Arc<dyn SiagaRepository>,
        feed_repo: Arc<dyn FeedRepository>,
        notification_repo: Arc<dyn NotificationRepository>,
        webhook_outbox_repo: Arc<dyn WebhookOutboxRepository>,
    ) -> Self {
        let idempotency = IdempotencyService::new(store, IdempotencyConfig::default());
        let chat_realtime = ChatRealtimeBus::new(&config);
        Self {
            config,
            idempotency,
            adaptive_path_repo,
            contribution_repo,
            evidence_repo,
            vouch_repo,
            vault_repo,
            chat_repo,
            moderation_repo,
            ontology_repo,
            siaga_repo,
            feed_repo,
            notification_repo,
            webhook_outbox_repo,
            chat_realtime,
            job_queue: None,
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
            let adaptive_path_repo = SurrealAdaptivePathRepository::new(&db_config).await?;
            let vault_repo = SurrealVaultRepository::new(&db_config).await?;
            let chat_repo = SurrealChatRepository::new(&db_config).await?;
            let contribution_repo = SurrealContributionRepository::new(&db_config).await?;
            let evidence_repo = SurrealEvidenceRepository::new(&db_config).await?;
            let vouch_repo = SurrealVouchRepository::new(&db_config).await?;
            let moderation_repo = SurrealModerationRepository::new(&db_config).await?;
            let ontology_repo = SurrealOntologyRepository::new(&db_config).await?;
            let siaga_repo = SurrealSiagaRepository::new(&db_config).await?;
            let feed_repo = SurrealDiscoveryFeedRepository::new(&db_config).await?;
            let notification_repo = SurrealDiscoveryNotificationRepository::new(&db_config).await?;
            let webhook_outbox_repo = SurrealWebhookOutboxRepository::new(&db_config).await?;
            Ok((
                Arc::new(adaptive_path_repo),
                Arc::new(contribution_repo),
                Arc::new(evidence_repo),
                Arc::new(vouch_repo),
                Arc::new(vault_repo),
                Arc::new(chat_repo),
                Arc::new(moderation_repo),
                Arc::new(ontology_repo),
                Arc::new(siaga_repo),
                Arc::new(feed_repo),
                Arc::new(notification_repo),
                Arc::new(webhook_outbox_repo),
            ))
        }
        _ => anyhow::bail!("unsupported DATA_BACKEND '{}'", config.data_backend),
    }
}

fn memory_repositories() -> RepositoryBundle {
    (
        Arc::new(InMemoryAdaptivePathRepository::new()),
        Arc::new(InMemoryContributionRepository::new()),
        Arc::new(InMemoryEvidenceRepository::new()),
        Arc::new(InMemoryVouchRepository::new()),
        Arc::new(InMemoryVaultRepository::new()),
        Arc::new(InMemoryChatRepository::new()),
        Arc::new(InMemoryModerationRepository::new()),
        Arc::new(InMemoryOntologyRepository::new()),
        Arc::new(InMemorySiagaRepository::new()),
        Arc::new(InMemoryDiscoveryFeedRepository::new()),
        Arc::new(InMemoryDiscoveryNotificationRepository::new()),
        Arc::new(InMemoryWebhookOutboxRepository::new()),
    )
}

async fn job_queue_for_config(config: &AppConfig) -> anyhow::Result<SharedJobQueue> {
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
    use tokio::time::Duration;

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
            chat_realtime_transport: "local".to_string(),
            chat_realtime_channel_prefix: "gotong:chat:realtime:test".to_string(),
            worker_queue_prefix: "gotong:jobs".to_string(),
            worker_poll_interval_ms: 1000,
            worker_promote_batch: 10,
            worker_backoff_base_ms: 1000,
            worker_backoff_max_ms: 60000,
            worker_ttl_cleanup_interval_ms: 3_600_000,
            worker_concept_verification_interval_ms: 86_400_000,
            worker_concept_verification_qids: "Q2095".to_string(),
            webhook_enabled: false,
            webhook_markov_url: "http://127.0.0.1:5000/webhook".to_string(),
            webhook_secret: "test-webhook-secret-32-chars-minimum".to_string(),
            webhook_max_attempts: 5,
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

    #[tokio::test]
    async fn chat_realtime_bus_local_mode_delivers_to_its_subscriber() {
        let config = AppConfig {
            chat_realtime_transport: "local".to_string(),
            ..app_config("test", "memory")
        };
        let bus = ChatRealtimeBus::new(&config);
        let thread_id = "thread-local-test";

        let mut receiver = bus.subscribe(thread_id).await;
        let message = ChatMessage {
            thread_id: thread_id.to_string(),
            message_id: "msg-local-1".to_string(),
            author_id: "user-1".to_string(),
            body: "hello".to_string(),
            attachments: Vec::new(),
            created_at_ms: 1,
            edited_at_ms: None,
            deleted_at_ms: None,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
        };
        bus.publish(thread_id, message.clone()).await;
        let received = tokio::time::timeout(Duration::from_secs(2), receiver.recv())
            .await
            .expect("message timed out")
            .expect("stream closed");
        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn chat_realtime_bus_redis_fanout_across_instances() {
        let base_config = app_config("test", "memory");
        if !redis_is_available(&base_config.redis_url).await {
            return;
        }

        let prefix = format!("gotong:chat:test:{}", uuid_v7_without_dashes());
        let config = AppConfig {
            chat_realtime_transport: "redis".to_string(),
            chat_realtime_channel_prefix: prefix.clone(),
            ..base_config
        };

        let bus_a = ChatRealtimeBus::new(&config);
        let bus_b = ChatRealtimeBus::new(&config);
        let thread_id = "thread-redis-fanout";
        let mut receiver_a = bus_a.subscribe(thread_id).await;
        let mut receiver_b = bus_b.subscribe(thread_id).await;

        let message = ChatMessage {
            thread_id: thread_id.to_string(),
            message_id: "msg-redis-1".to_string(),
            author_id: "user-1".to_string(),
            body: "hello".to_string(),
            attachments: Vec::new(),
            created_at_ms: 1,
            edited_at_ms: None,
            deleted_at_ms: None,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
        };

        bus_a.publish(thread_id, message.clone()).await;

        let first = tokio::time::timeout(Duration::from_secs(3), receiver_a.recv())
            .await
            .expect("message timed out")
            .expect("stream closed");
        let second = tokio::time::timeout(Duration::from_secs(3), receiver_b.recv())
            .await
            .expect("message timed out")
            .expect("stream closed");
        assert_eq!(first, message);
        assert_eq!(second, message);
    }

    async fn redis_is_available(redis_url: &str) -> bool {
        let client = match redis::Client::open(redis_url) {
            Ok(client) => client,
            Err(_) => return false,
        };
        let mut conn = match client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map(|response| response == "PONG")
            .unwrap_or(false)
    }
}
