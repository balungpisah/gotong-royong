use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::mode::Mode;
use crate::ports::contributions::ContributionRepository;

const MAX_TITLE_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 2_000;
const MAX_SKILL_IDS: usize = 10;
const MAX_METADATA_KEYS: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContributionType {
    TaskCompletion,
    CodeReview,
    Documentation,
    Mentoring,
    EventOrganization,
    CommunityService,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contribution {
    pub contribution_id: String,
    pub author_id: String,
    pub author_username: String,
    pub mode: Mode,
    pub contribution_type: ContributionType,
    pub title: String,
    pub description: Option<String>,
    pub evidence_url: Option<String>,
    pub skill_ids: Vec<String>,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub request_id: String,
    pub correlation_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug)]
pub struct ContributionCreate {
    pub mode: Mode,
    pub contribution_type: ContributionType,
    pub title: String,
    pub description: Option<String>,
    pub evidence_url: Option<String>,
    pub skill_ids: Vec<String>,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Clone)]
pub struct ContributionService {
    repository: Arc<dyn ContributionRepository>,
}

impl ContributionService {
    pub fn new(repository: Arc<dyn ContributionRepository>) -> Self {
        Self { repository }
    }

    pub fn into_tandang_event_payload(contribution: &Contribution) -> serde_json::Value {
        serde_json::json!({
            "event_type": "contribution_created",
            "actor": {
                "user_id": contribution.author_id,
                "username": contribution.author_username,
            },
            "subject": {
                "mode": contribution.mode.as_str(),
                "contribution_type": to_contribution_type_name(&contribution.contribution_type),
                "title": contribution.title,
                "description": contribution.description,
                "evidence_url": contribution.evidence_url,
                "skill_ids": contribution.skill_ids,
                "metadata": contribution.metadata,
            },
            "event_id": contribution_to_event_id(&contribution.contribution_id),
            "timestamp": format_rfc3339(contribution.created_at_ms),
        })
    }

    pub async fn create(
        &self,
        actor: ActorIdentity,
        request_id: String,
        correlation_id: String,
        input: ContributionCreate,
    ) -> DomainResult<Contribution> {
        let payload = validate_contribution_create(&input)?;
        let now = now_ms();
        let contribution = Contribution {
            contribution_id: crate::util::uuid_v7_without_dashes(),
            author_id: actor.user_id,
            author_username: actor.username,
            mode: payload.mode,
            contribution_type: payload.contribution_type,
            title: payload.title,
            description: payload.description,
            evidence_url: payload.evidence_url,
            skill_ids: payload.skill_ids,
            metadata: payload.metadata,
            request_id,
            correlation_id,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.repository.create(&contribution).await
    }

    pub async fn get(&self, contribution_id: &str) -> DomainResult<Contribution> {
        self.repository
            .get(contribution_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn list_by_author(&self, author_id: &str) -> DomainResult<Vec<Contribution>> {
        self.repository.list_by_author(author_id).await
    }
}

fn contribution_to_event_id(contribution_id: &str) -> String {
    if contribution_id.is_empty() {
        "evt_0000000000000000".to_string()
    } else {
        let short = contribution_id
            .chars()
            .filter(|ch| ch.is_ascii_hexdigit())
            .collect::<String>();
        let short = short.chars().take(16).collect::<String>();
        let short = if short.len() < 16 {
            format!("{}{}", short, "0".repeat(16 - short.len()))
        } else {
            short
        };
        format!("evt_{short}")
    }
}

fn to_contribution_type_name(ty: &ContributionType) -> &'static str {
    match ty {
        ContributionType::TaskCompletion => "task_completion",
        ContributionType::CodeReview => "code_review",
        ContributionType::Documentation => "documentation",
        ContributionType::Mentoring => "mentoring",
        ContributionType::EventOrganization => "event_organization",
        ContributionType::CommunityService => "community_service",
        ContributionType::Custom => "custom",
    }
}

fn format_rfc3339(epoch_ms: i64) -> String {
    crate::util::format_ms_rfc3339(epoch_ms)
}

fn validate_contribution_create(
    input: &ContributionCreate,
) -> Result<ContributionCreate, DomainError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(DomainError::Validation("title is required".into()));
    }
    if title.chars().count() > MAX_TITLE_LENGTH {
        return Err(DomainError::Validation(format!(
            "title exceeds max length of {MAX_TITLE_LENGTH}"
        )));
    }

    if let Some(description) = input.description.as_ref() {
        if description.chars().count() > MAX_DESCRIPTION_LENGTH {
            return Err(DomainError::Validation(format!(
                "description exceeds max length of {MAX_DESCRIPTION_LENGTH}"
            )));
        }
    }

    if input.skill_ids.len() > MAX_SKILL_IDS {
        return Err(DomainError::Validation(format!(
            "skill_ids exceeds max of {MAX_SKILL_IDS}"
        )));
    }

    if input
        .metadata
        .as_ref()
        .is_some_and(|metadata| metadata.len() > MAX_METADATA_KEYS)
    {
        return Err(DomainError::Validation(format!(
            "metadata exceeds max of {MAX_METADATA_KEYS} keys"
        )));
    }

    Ok(ContributionCreate {
        mode: input.mode.clone(),
        contribution_type: input.contribution_type.clone(),
        title: title.to_string(),
        description: input.description.clone(),
        evidence_url: input.evidence_url.clone(),
        skill_ids: dedupe_and_trim(&input.skill_ids),
        metadata: input.metadata.clone(),
    })
}

fn dedupe_and_trim(skill_ids: &[String]) -> Vec<String> {
    let mut deduped = Vec::with_capacity(skill_ids.len());
    let mut seen = std::collections::HashSet::new();
    for raw in skill_ids {
        let value = raw.trim().to_string();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_and_trim_skills() {
        let input = vec![
            "skill-a".to_string(),
            "skill-b".to_string(),
            "  ".to_string(),
            "skill-a".to_string(),
            "".to_string(),
            "skill-c".to_string(),
        ];
        let output = dedupe_and_trim(&input);
        assert_eq!(
            output,
            vec![
                "skill-a".to_string(),
                "skill-b".to_string(),
                "skill-c".to_string()
            ]
        );
    }

    #[test]
    fn validate_contribution_rejects_empty_title() {
        let result = validate_contribution_create(&ContributionCreate {
            mode: Mode::Komunitas,
            contribution_type: ContributionType::TaskCompletion,
            title: "   ".to_string(),
            description: None,
            evidence_url: None,
            skill_ids: vec![],
            metadata: None,
        });
        assert!(result.is_err());
    }
}
