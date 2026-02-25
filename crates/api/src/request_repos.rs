use std::sync::Arc;

use gotong_domain::ports::adaptive_path::AdaptivePathRepository;
use gotong_domain::ports::chat::ChatRepository;
use gotong_domain::ports::contributions::ContributionRepository;
use gotong_domain::ports::discovery::{FeedRepository, NotificationRepository};
use gotong_domain::ports::evidence::EvidenceRepository;
use gotong_domain::ports::moderation::ModerationRepository;
use gotong_domain::ports::ontology::OntologyRepository;
use gotong_domain::ports::siaga::SiagaRepository;
use gotong_domain::ports::vault::VaultRepository;
use gotong_domain::ports::vouches::VouchRepository;
use gotong_domain::ports::webhook::WebhookOutboxRepository;
use gotong_infra::repositories::{
    SurrealAdaptivePathRepository, SurrealChatRepository, SurrealContributionRepository,
    SurrealDiscoveryFeedRepository, SurrealDiscoveryNotificationRepository,
    SurrealEvidenceRepository, SurrealModerationRepository, SurrealOntologyRepository,
    SurrealSiagaRepository, SurrealVaultRepository, SurrealVouchRepository,
    SurrealWebhookOutboxRepository,
};

use crate::middleware::AuthContext;
use crate::state::AppState;

pub fn adaptive_path_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn AdaptivePathRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealAdaptivePathRepository::with_client(session.client())),
        None => state.adaptive_path_repo.clone(),
    }
}

pub fn contribution_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn ContributionRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealContributionRepository::with_client(session.client())),
        None => state.contribution_repo.clone(),
    }
}

pub fn evidence_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn EvidenceRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealEvidenceRepository::with_client(session.client())),
        None => state.evidence_repo.clone(),
    }
}

pub fn vouch_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn VouchRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealVouchRepository::with_client(session.client())),
        None => state.vouch_repo.clone(),
    }
}

pub fn vault_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn VaultRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealVaultRepository::with_client(session.client())),
        None => state.vault_repo.clone(),
    }
}

pub fn chat_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn ChatRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealChatRepository::with_client(session.client())),
        None => state.chat_repo.clone(),
    }
}

pub fn moderation_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn ModerationRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealModerationRepository::with_client(session.client())),
        None => state.moderation_repo.clone(),
    }
}

pub fn ontology_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn OntologyRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealOntologyRepository::with_client(session.client())),
        None => state.ontology_repo.clone(),
    }
}

#[allow(dead_code)]
pub fn siaga_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn SiagaRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealSiagaRepository::with_client(session.client())),
        None => state.siaga_repo.clone(),
    }
}

pub fn feed_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn FeedRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealDiscoveryFeedRepository::with_client(
            session.client(),
        )),
        None => state.feed_repo.clone(),
    }
}

pub fn notification_repo(state: &AppState, auth: &AuthContext) -> Arc<dyn NotificationRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealDiscoveryNotificationRepository::with_client(
            session.client(),
        )),
        None => state.notification_repo.clone(),
    }
}

#[allow(dead_code)]
pub fn webhook_outbox_repo(
    state: &AppState,
    auth: &AuthContext,
) -> Arc<dyn WebhookOutboxRepository> {
    match &auth.surreal_db_session {
        Some(session) => Arc::new(SurrealWebhookOutboxRepository::with_client(
            session.client(),
        )),
        None => state.webhook_outbox_repo.clone(),
    }
}
