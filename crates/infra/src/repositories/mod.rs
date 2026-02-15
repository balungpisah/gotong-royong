use std::collections::HashMap;
use std::sync::Arc;

use gotong_domain::DomainResult;
use gotong_domain::contributions::Contribution;
use gotong_domain::error::DomainError;
use gotong_domain::evidence::Evidence;
use gotong_domain::ports::contributions::ContributionRepository;
use gotong_domain::ports::evidence::EvidenceRepository;
use gotong_domain::ports::transitions::TrackTransitionRepository;
use gotong_domain::ports::vouches::VouchRepository;
use gotong_domain::transitions::TrackStateTransition;
use gotong_domain::vouches::Vouch;

use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryContributionRepository {
    store: Arc<RwLock<HashMap<String, Contribution>>>,
}

impl InMemoryContributionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ContributionRepository for InMemoryContributionRepository {
    fn create(
        &self,
        contribution: &Contribution,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Contribution>> {
        let contribution = contribution.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&contribution.contribution_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(contribution.contribution_id.clone(), contribution.clone());
            Ok(contribution)
        })
    }

    fn get(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Contribution>>> {
        let contribution_id = contribution_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            Ok(items.get(&contribution_id).cloned())
        })
    }

    fn list_by_author(
        &self,
        author_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            let mut contributions: Vec<_> = items
                .values()
                .filter(|item| item.author_id == author_id)
                .cloned()
                .collect();
            contributions.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.contribution_id.cmp(&a.contribution_id))
            });
            Ok(contributions)
        })
    }

    fn list_recent(
        &self,
        author_id: &str,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = self_recent_filtered(&store, &author_id, limit).await;
            items.truncate(limit);
            Ok(items)
        })
    }
}

async fn self_recent_filtered(
    store: &Arc<RwLock<HashMap<String, Contribution>>>,
    author_id: &str,
    limit: usize,
) -> Vec<Contribution> {
    let mut contributions: Vec<_> = store
        .read()
        .await
        .values()
        .filter(|item| item.author_id == author_id)
        .cloned()
        .collect();
    contributions.sort_by(|a, b| {
        b.created_at_ms
            .cmp(&a.created_at_ms)
            .then_with(|| b.contribution_id.cmp(&a.contribution_id))
    });
    contributions.into_iter().take(limit).collect()
}

#[derive(Default)]
pub struct InMemoryEvidenceRepository {
    store: Arc<RwLock<HashMap<String, Evidence>>>,
}

impl InMemoryEvidenceRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EvidenceRepository for InMemoryEvidenceRepository {
    fn create(
        &self,
        evidence: &Evidence,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Evidence>> {
        let evidence = evidence.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&evidence.evidence_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(evidence.evidence_id.clone(), evidence.clone());
            Ok(evidence)
        })
    }

    fn get(
        &self,
        evidence_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Evidence>>> {
        let evidence_id = evidence_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            Ok(items.get(&evidence_id).cloned())
        })
    }

    fn list_by_contribution(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Evidence>>> {
        let contribution_id = contribution_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            let mut evidence: Vec<_> = items
                .values()
                .filter(|item| item.contribution_id == contribution_id)
                .cloned()
                .collect();
            evidence.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.evidence_id.cmp(&a.evidence_id))
            });
            Ok(evidence)
        })
    }
}

#[derive(Default)]
pub struct InMemoryVouchRepository {
    store: Arc<RwLock<HashMap<String, Vouch>>>,
}

impl InMemoryVouchRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VouchRepository for InMemoryVouchRepository {
    fn create(&self, vouch: &Vouch) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vouch>> {
        let vouch = vouch.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&vouch.vouch_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(vouch.vouch_id.clone(), vouch.clone());
            Ok(vouch)
        })
    }

    fn list_by_vouchee(
        &self,
        vouchee_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let vouchee_id = vouchee_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut vouches = store
                .read()
                .await
                .values()
                .filter(|item| item.vouchee_id == vouchee_id)
                .cloned()
                .collect::<Vec<_>>();
            vouches.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.vouch_id.cmp(&a.vouch_id))
            });
            Ok(vouches)
        })
    }

    fn list_by_voucher(
        &self,
        voucher_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let voucher_id = voucher_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut vouches = store
                .read()
                .await
                .values()
                .filter(|item| item.voucher_id == voucher_id)
                .cloned()
                .collect::<Vec<_>>();
            vouches.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.vouch_id.cmp(&a.vouch_id))
            });
            Ok(vouches)
        })
    }
}

#[derive(Default)]
pub struct InMemoryTrackTransitionRepository {
    transitions: Arc<RwLock<HashMap<String, TrackStateTransition>>>,
    by_request: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryTrackTransitionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl InMemoryTrackTransitionRepository {
    fn request_key(entity_id: &str, request_id: &str) -> String {
        format!("{entity_id}:{request_id}")
    }
}

impl TrackTransitionRepository for InMemoryTrackTransitionRepository {
    fn create(
        &self,
        transition: &TrackStateTransition,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<TrackStateTransition>> {
        let transition = transition.clone();
        let transitions = self.transitions.clone();
        let by_request = self.by_request.clone();
        Box::pin(async move {
            let mut transition_map = transitions.write().await;
            if transition_map.contains_key(&transition.transition_id) {
                return Err(DomainError::Conflict);
            }

            let request_key = Self::request_key(&transition.entity_id, &transition.request_id);
            let mut request_map = by_request.write().await;
            if request_map.contains_key(&request_key) {
                return Err(DomainError::Conflict);
            }

            request_map.insert(request_key, transition.transition_id.clone());
            transition_map.insert(transition.transition_id.clone(), transition.clone());
            Ok(transition)
        })
    }

    fn get_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let request_key = Self::request_key(entity_id, request_id);
        let transitions = self.transitions.clone();
        let by_request = self.by_request.clone();
        Box::pin(async move {
            let request_map = by_request.read().await;
            let Some(transition_id) = request_map.get(&request_key) else {
                return Ok(None);
            };
            let transitions = transitions.read().await;
            Ok(transitions.get(transition_id).cloned())
        })
    }

    fn get_by_transition_id(
        &self,
        transition_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let transition_id = transition_id.to_string();
        let transitions = self.transitions.clone();
        Box::pin(async move {
            let transitions = transitions.read().await;
            Ok(transitions.get(&transition_id).cloned())
        })
    }

    fn list_by_entity(
        &self,
        entity_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<TrackStateTransition>>> {
        let entity_id = entity_id.to_string();
        let transitions = self.transitions.clone();
        Box::pin(async move {
            let transitions = transitions.read().await;
            let mut list: Vec<_> = transitions
                .values()
                .filter(|transition| transition.entity_id == entity_id)
                .cloned()
                .collect();
            list.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.transition_id.cmp(&right.transition_id))
            });
            Ok(list)
        })
    }
}
