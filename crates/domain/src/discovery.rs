use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::jobs::now_ms;
use crate::ports::discovery::{
    FeedRepository, FeedRepositoryQuery, FeedRepositorySearchQuery, NotificationRepository,
    NotificationRepositoryListQuery,
};
use crate::{DomainResult, error::DomainError, identity::ActorIdentity};

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 50;
const MAX_SEARCH_FETCH_LIMIT: usize = 1_024;
const DEFAULT_SUGGESTION_LIMIT: usize = 6;
const MAX_SUGGESTION_LIMIT: usize = 20;
const SUGGESTION_FETCH_MULTIPLIER: usize = 8;
const SUGGESTION_FETCH_CAP: usize = 200;
const ONE_WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;

pub const FEED_SOURCE_CONTRIBUTION: &str = "contribution";
pub const FEED_SOURCE_VAULT: &str = "vault";
pub const FEED_SOURCE_SIAGA: &str = "siaga";
pub const FEED_SOURCE_MODERATION: &str = "moderation";
pub const FEED_SOURCE_VOUCH: &str = "vouch";
pub const FEED_SOURCE_ONTOLOGY_NOTE: &str = "ontology_note";

pub const NOTIF_TYPE_VOUCH: &str = "vouch";
pub const NOTIF_TYPE_VAULT: &str = "vault";
pub const NOTIF_TYPE_SIAGA: &str = "siaga";
pub const NOTIF_TYPE_SYSTEM: &str = "system";

const OPEN_PRIVACY_LEVELS: &[&str] = &["", "public", "open", "unrestricted", "l1", "level1"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedItem {
    pub feed_id: String,
    pub source_type: String,
    pub source_id: String,
    pub actor_id: String,
    pub actor_username: String,
    pub title: String,
    pub summary: Option<String>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub occurred_at_ms: i64,
    pub created_at_ms: i64,
    pub request_id: String,
    pub correlation_id: String,
    pub participant_ids: Vec<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InAppNotification {
    pub notification_id: String,
    pub user_id: String,
    pub actor_id: String,
    pub actor_username: String,
    pub notification_type: String,
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub body: String,
    pub payload: Option<serde_json::Value>,
    pub created_at_ms: i64,
    pub read_at_ms: Option<i64>,
    pub privacy_level: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub dedupe_key: String,
}

#[derive(Clone)]
pub struct FeedIngestInput {
    pub source_type: String,
    pub source_id: String,
    pub actor: ActorIdentity,
    pub title: String,
    pub summary: Option<String>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub occurred_at_ms: Option<i64>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
    pub participant_ids: Vec<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct NotificationIngestInput {
    pub recipient_id: String,
    pub actor: ActorIdentity,
    pub notification_type: String,
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub body: String,
    pub payload: Option<serde_json::Value>,
    pub privacy_level: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
    pub dedupe_key: Option<String>,
}

#[derive(Clone)]
pub struct FeedListQuery {
    pub actor_id: String,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: bool,
}

#[derive(Clone)]
pub struct FeedSuggestionsQuery {
    pub actor_id: String,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedSuggestion {
    pub entity_id: String,
    pub entity_type: String,
    pub label: String,
    pub followed: bool,
    pub description: Option<String>,
    pub witness_count: usize,
    pub follower_count: usize,
}

#[derive(Clone)]
pub struct SearchListQuery {
    pub actor_id: String,
    pub query_text: String,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: bool,
    pub exclude_vault: bool,
}

#[derive(Clone)]
pub struct NotificationListQuery {
    pub actor_id: String,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub include_read: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PagedFeed {
    pub items: Vec<FeedItem>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PagedNotifications {
    pub items: Vec<InAppNotification>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub item: FeedItem,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchPage {
    pub items: Vec<SearchResult>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklyDigest {
    pub user_id: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub generated_at_ms: i64,
    pub unread_count: usize,
    pub events: Vec<SearchResult>,
}

#[derive(Clone)]
pub struct DiscoveryService {
    feed_repo: Arc<dyn FeedRepository>,
    notification_repo: Arc<dyn NotificationRepository>,
}

impl DiscoveryService {
    pub fn new(
        feed_repo: Arc<dyn FeedRepository>,
        notification_repo: Arc<dyn NotificationRepository>,
    ) -> Self {
        Self {
            feed_repo,
            notification_repo,
        }
    }

    pub async fn ingest_feed(&self, input: FeedIngestInput) -> DomainResult<FeedItem> {
        validate_feed_input(&input)?;
        let item = FeedItem {
            feed_id: crate::util::uuid_v7_without_dashes(),
            source_type: input.source_type,
            source_id: input.source_id,
            actor_id: input.actor.user_id,
            actor_username: input.actor.username,
            title: input.title,
            summary: input.summary,
            scope_id: input.scope_id,
            privacy_level: input.privacy_level,
            occurred_at_ms: input.occurred_at_ms.unwrap_or_else(now_ms),
            created_at_ms: input.request_ts_ms.unwrap_or_else(now_ms),
            request_id: input.request_id,
            correlation_id: input.correlation_id,
            participant_ids: dedupe_vec(input.participant_ids),
            payload: input.payload,
        };

        let persisted_item = match self.feed_repo.create_feed_item(&item).await {
            Ok(item) => item,
            Err(DomainError::Conflict) => self
                .feed_repo
                .get_by_source_request(&item.source_type, &item.source_id, &item.request_id)
                .await?
                .ok_or(DomainError::Conflict)?,
            Err(err) => return Err(err),
        };

        self.feed_repo
            .upsert_participant_edges_for_item(&persisted_item)
            .await?;

        Ok(persisted_item)
    }

    pub async fn list_feed(&self, query: FeedListQuery) -> DomainResult<PagedFeed> {
        validate_actor_id(&query.actor_id)?;
        let limit = normalize_limit(query.limit)?;
        let (mut cursor_ms, mut cursor_feed_id) = parse_feed_cursor(query.cursor.as_deref())?;
        let actor_id = query.actor_id.clone();
        let mut items = Vec::new();
        let fetch_limit = limit + 1;

        loop {
            let repo_query = FeedRepositoryQuery {
                actor_id: actor_id.clone(),
                cursor_occurred_at_ms: cursor_ms,
                cursor_feed_id: cursor_feed_id.clone(),
                limit: fetch_limit,
                scope_id: query.scope_id.clone(),
                privacy_level: query.privacy_level.clone(),
                from_ms: query.from_ms,
                to_ms: query.to_ms,
                involvement_only: query.involvement_only,
            };
            let rows = self.feed_repo.list_feed(&repo_query).await?;
            for item in rows.iter() {
                if is_visible_to_actor(&actor_id, item) {
                    items.push(item.clone());
                }
            }

            if items.len() > limit || rows.len() < fetch_limit {
                break;
            }

            if let Some(last_row) = rows.last() {
                if cursor_ms == Some(last_row.occurred_at_ms)
                    && cursor_feed_id.as_deref() == Some(last_row.feed_id.as_str())
                {
                    break;
                }
                cursor_ms = Some(last_row.occurred_at_ms);
                cursor_feed_id = Some(last_row.feed_id.clone());
            } else {
                break;
            }
        }

        let next_cursor = items
            .get(limit.saturating_sub(1))
            .filter(|_| items.len() > limit)
            .map(|item| make_feed_cursor(item.occurred_at_ms, &item.feed_id));
        if items.len() > limit {
            items.truncate(limit);
        }
        Ok(PagedFeed { items, next_cursor })
    }

    pub async fn list_feed_suggestions(
        &self,
        query: FeedSuggestionsQuery,
    ) -> DomainResult<Vec<FeedSuggestion>> {
        validate_actor_id(&query.actor_id)?;
        let limit = normalize_suggestion_limit(query.limit)?;
        let fetch_limit = limit
            .saturating_mul(SUGGESTION_FETCH_MULTIPLIER)
            .min(SUGGESTION_FETCH_CAP)
            .max(limit);
        let repo_query = FeedRepositoryQuery {
            actor_id: query.actor_id.clone(),
            cursor_occurred_at_ms: None,
            cursor_feed_id: None,
            limit: fetch_limit,
            scope_id: query.scope_id,
            privacy_level: query.privacy_level,
            from_ms: query.from_ms,
            to_ms: query.to_ms,
            involvement_only: false,
        };
        let rows = self.feed_repo.list_feed(&repo_query).await?;
        let mut grouped: HashMap<String, SuggestionAggregate> = HashMap::new();

        for item in rows {
            if !is_visible_to_actor(&query.actor_id, &item) {
                continue;
            }
            for candidate in extract_suggestion_candidates(item.payload.as_ref()) {
                if !is_suggestable_entity_type(&candidate.entity_type) {
                    continue;
                }
                let entity_id = normalized_entity_id(
                    candidate.entity_id.as_deref(),
                    &candidate.entity_type,
                    &candidate.label,
                );
                if entity_id.is_empty() {
                    continue;
                }

                let aggregate =
                    grouped
                        .entry(entity_id.clone())
                        .or_insert_with(|| SuggestionAggregate {
                            suggestion: FeedSuggestion {
                                entity_id: entity_id.clone(),
                                entity_type: candidate.entity_type.clone(),
                                label: candidate.label.clone(),
                                followed: false,
                                description: candidate.description.clone(),
                                witness_count: 0,
                                follower_count: candidate.follower_count.unwrap_or(0),
                            },
                            seen_sources: HashSet::new(),
                        });

                if aggregate.suggestion.description.is_none() && candidate.description.is_some() {
                    aggregate.suggestion.description = candidate.description.clone();
                }
                aggregate.suggestion.follower_count = aggregate
                    .suggestion
                    .follower_count
                    .max(candidate.follower_count.unwrap_or(0));
                if candidate.followed {
                    aggregate.suggestion.followed = true;
                }
                if aggregate.seen_sources.insert(item.source_id.clone()) {
                    aggregate.suggestion.witness_count += 1;
                }
            }
        }

        let mut suggestions: Vec<FeedSuggestion> = grouped
            .into_values()
            .map(|aggregate| aggregate.suggestion)
            .filter(|suggestion| !suggestion.followed)
            .map(|mut suggestion| {
                suggestion.followed = false;
                if suggestion.follower_count == 0 {
                    suggestion.follower_count = suggestion.witness_count;
                }
                suggestion
            })
            .collect();

        suggestions.sort_by(|left, right| {
            right
                .witness_count
                .cmp(&left.witness_count)
                .then_with(|| right.follower_count.cmp(&left.follower_count))
                .then_with(|| left.label.cmp(&right.label))
        });
        suggestions.truncate(limit);
        Ok(suggestions)
    }

    pub async fn search(&self, query: SearchListQuery) -> DomainResult<SearchPage> {
        validate_actor_id(&query.actor_id)?;
        if query.query_text.trim().is_empty() {
            return Err(DomainError::Validation("query_text is required".into()));
        }

        let limit = normalize_limit(query.limit)?;
        let query_text = query.query_text.trim().to_string();
        let search_cursor = parse_search_cursor(query.cursor.as_deref())?;
        let mut results = Vec::new();
        let mut seen_feed_ids = std::collections::HashSet::new();
        let fetch_limit = limit + 1;
        let mut query_limit = fetch_limit;

        loop {
            let repo_query = FeedRepositorySearchQuery {
                actor_id: query.actor_id.clone(),
                limit: query_limit,
                scope_id: query.scope_id.clone(),
                privacy_level: query.privacy_level.clone(),
                from_ms: query.from_ms,
                to_ms: query.to_ms,
                involvement_only: query.involvement_only,
                exclude_vault: query.exclude_vault,
                query_text: query_text.clone(),
            };

            let rows = self.feed_repo.search_feed(&repo_query).await?;
            for item in rows.iter() {
                if !is_visible_to_actor(&query.actor_id, item) {
                    continue;
                }
                if query.exclude_vault && item.source_type == FEED_SOURCE_VAULT {
                    continue;
                }
                if !matches_search_text(item, &query_text) {
                    continue;
                }
                if !seen_feed_ids.insert(item.feed_id.clone()) {
                    continue;
                }
                let score = score_query_match(item, &query_text);
                results.push(SearchResult {
                    item: item.clone(),
                    score,
                });
            }

            if results.len() > limit || rows.len() < fetch_limit {
                break;
            }

            if rows.len() < query_limit || query_limit >= MAX_SEARCH_FETCH_LIMIT {
                break;
            }
            query_limit = (query_limit * 2).min(MAX_SEARCH_FETCH_LIMIT);
        }

        results.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| right.item.occurred_at_ms.cmp(&left.item.occurred_at_ms))
                .then_with(|| right.item.feed_id.cmp(&left.item.feed_id))
        });

        if let Some((cursor_score, cursor_occurred_ms, cursor_feed_id)) = search_cursor {
            results.retain(|result| {
                search_result_older(result, cursor_score, cursor_occurred_ms, &cursor_feed_id)
            });
        }

        let next_cursor = results
            .get(limit.saturating_sub(1))
            .filter(|_| results.len() > limit)
            .map(|result| {
                make_search_cursor(
                    result.score,
                    result.item.occurred_at_ms,
                    &result.item.feed_id,
                )
            });
        if results.len() > limit {
            results.truncate(limit);
        }

        Ok(SearchPage {
            items: results,
            next_cursor,
        })
    }

    pub async fn ingest_notification(
        &self,
        input: NotificationIngestInput,
    ) -> DomainResult<InAppNotification> {
        validate_notification_input(&input)?;
        let dedupe_key = input.dedupe_key.unwrap_or_else(|| {
            format!(
                "{}:{}:{}",
                input.source_type, input.source_id, input.request_id
            )
        });
        let notification = InAppNotification {
            notification_id: crate::util::uuid_v7_without_dashes(),
            user_id: input.recipient_id,
            actor_id: input.actor.user_id,
            actor_username: input.actor.username,
            notification_type: input.notification_type,
            source_type: input.source_type,
            source_id: input.source_id,
            title: input.title,
            body: input.body,
            payload: input.payload,
            created_at_ms: input.request_ts_ms.unwrap_or_else(now_ms),
            read_at_ms: None,
            privacy_level: input.privacy_level,
            request_id: input.request_id,
            correlation_id: input.correlation_id,
            dedupe_key,
        };

        match self
            .notification_repo
            .create_notification(&notification)
            .await
        {
            Ok(notification) => Ok(notification),
            Err(DomainError::Conflict) => self
                .notification_repo
                .get_by_dedupe_key(&notification.user_id, &notification.dedupe_key)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn list_notifications(
        &self,
        query: NotificationListQuery,
    ) -> DomainResult<PagedNotifications> {
        validate_actor_id(&query.actor_id)?;
        let limit = normalize_limit(query.limit)?;
        let include_read = query.include_read.unwrap_or(false);
        let (cursor_ms, cursor_notification_id) =
            parse_notification_cursor(query.cursor.as_deref())?;
        let actor_id = query.actor_id.clone();

        let repo_query = NotificationRepositoryListQuery {
            user_id: actor_id.clone(),
            cursor_created_at_ms: cursor_ms,
            cursor_notification_id,
            limit: limit + 1,
            include_read,
        };
        let mut items = self
            .notification_repo
            .list_notifications(&repo_query)
            .await?;
        items.retain(|notification| is_visible_notification(&actor_id, notification));

        let next_cursor = items
            .get(limit.saturating_sub(1))
            .filter(|_| items.len() > limit)
            .map(|item| make_notification_cursor(item.created_at_ms, &item.notification_id));
        if items.len() > limit {
            items.truncate(limit);
        }
        Ok(PagedNotifications { items, next_cursor })
    }

    pub async fn mark_notification_read(
        &self,
        actor_id: &str,
        notification_id: &str,
    ) -> DomainResult<InAppNotification> {
        validate_actor_id(actor_id)?;
        let notification = self
            .notification_repo
            .mark_as_read(actor_id, notification_id, now_ms())
            .await?;
        if notification.user_id != actor_id {
            return Err(DomainError::Forbidden(
                "notification belongs to another user".into(),
            ));
        }
        Ok(notification)
    }

    pub async fn unread_notification_count(&self, actor_id: &str) -> DomainResult<usize> {
        validate_actor_id(actor_id)?;
        self.notification_repo.unread_count(actor_id).await
    }

    pub async fn weekly_digest(
        &self,
        actor_id: &str,
        window_start_ms: Option<i64>,
        window_end_ms: Option<i64>,
    ) -> DomainResult<WeeklyDigest> {
        validate_actor_id(actor_id)?;
        let now_ms = now_ms();
        let window_end_ms = window_end_ms.unwrap_or(now_ms);
        let window_start_ms = window_start_ms.unwrap_or(window_end_ms - ONE_WEEK_MS);
        if window_start_ms > window_end_ms {
            return Err(DomainError::Validation(
                "window_start_ms must be <= window_end_ms".into(),
            ));
        }

        let notifications = self
            .notification_repo
            .list_notifications_in_window(actor_id, window_start_ms, window_end_ms)
            .await?;
        let unread_count = self.unread_notification_count(actor_id).await?;
        let events = notifications
            .into_iter()
            .filter(|notification| is_visible_notification(actor_id, notification))
            .map(|notification| SearchResult {
                item: FeedItem {
                    feed_id: notification.notification_id,
                    source_type: notification.source_type,
                    source_id: notification.source_id,
                    actor_id: notification.actor_id,
                    actor_username: notification.actor_username,
                    title: notification.title,
                    summary: Some(notification.body),
                    scope_id: None,
                    privacy_level: notification.privacy_level,
                    occurred_at_ms: notification.created_at_ms,
                    created_at_ms: notification.created_at_ms,
                    request_id: notification.request_id,
                    correlation_id: notification.correlation_id,
                    participant_ids: Vec::new(),
                    payload: notification.payload,
                },
                score: 0,
            })
            .collect();

        Ok(WeeklyDigest {
            user_id: actor_id.to_string(),
            window_start_ms,
            window_end_ms,
            generated_at_ms: now_ms,
            unread_count,
            events,
        })
    }
}

fn normalize_limit(limit: Option<usize>) -> DomainResult<usize> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        Err(DomainError::Validation(format!(
            "limit must be between 1 and {MAX_LIMIT}"
        )))
    } else {
        Ok(limit)
    }
}

fn normalize_suggestion_limit(limit: Option<usize>) -> DomainResult<usize> {
    let limit = limit.unwrap_or(DEFAULT_SUGGESTION_LIMIT);
    if !(1..=MAX_SUGGESTION_LIMIT).contains(&limit) {
        Err(DomainError::Validation(format!(
            "suggestion limit must be between 1 and {MAX_SUGGESTION_LIMIT}"
        )))
    } else {
        Ok(limit)
    }
}

fn validate_feed_input(input: &FeedIngestInput) -> DomainResult<()> {
    if input.source_type.trim().is_empty() {
        return Err(DomainError::Validation("source_type is required".into()));
    }
    if input.source_id.trim().is_empty() {
        return Err(DomainError::Validation("source_id is required".into()));
    }
    if input.title.trim().is_empty() {
        return Err(DomainError::Validation("title is required".into()));
    }
    validate_actor_id(&input.actor.user_id)?;
    Ok(())
}

fn validate_notification_input(input: &NotificationIngestInput) -> DomainResult<()> {
    if input.recipient_id.trim().is_empty() {
        return Err(DomainError::Validation("recipient_id is required".into()));
    }
    if input.notification_type.trim().is_empty() {
        return Err(DomainError::Validation(
            "notification_type is required".into(),
        ));
    }
    if input.source_type.trim().is_empty() {
        return Err(DomainError::Validation("source_type is required".into()));
    }
    if input.source_id.trim().is_empty() {
        return Err(DomainError::Validation("source_id is required".into()));
    }
    if input.title.trim().is_empty() {
        return Err(DomainError::Validation("title is required".into()));
    }
    if input.body.trim().is_empty() {
        return Err(DomainError::Validation("body is required".into()));
    }
    validate_actor_id(&input.actor.user_id)?;
    Ok(())
}

fn validate_actor_id(actor_id: &str) -> DomainResult<()> {
    if actor_id.trim().is_empty() {
        return Err(DomainError::Validation("actor_id is required".into()));
    }
    Ok(())
}

fn dedupe_vec(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(|value| {
            let normalized = value.trim().to_string();
            if normalized.is_empty() {
                None
            } else if seen.insert(normalized.clone()) {
                Some(normalized)
            } else {
                None
            }
        })
        .collect()
}

fn parse_feed_cursor(value: Option<&str>) -> DomainResult<(Option<i64>, Option<String>)> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok((None, None));
    };
    let mut parts = value.splitn(2, ':');
    let occurred_raw = parts.next().ok_or_else(|| {
        DomainError::Validation("invalid cursor format; expected <occurred_at_ms>:<feed_id>".into())
    })?;
    let id = parts.next().ok_or_else(|| {
        DomainError::Validation("invalid cursor format; expected <occurred_at_ms>:<feed_id>".into())
    })?;
    let occurred_at_ms = occurred_raw
        .parse()
        .map_err(|_| DomainError::Validation("invalid cursor format".into()))?;
    if id.trim().is_empty() {
        return Err(DomainError::Validation("invalid cursor format".into()));
    }
    Ok((Some(occurred_at_ms), Some(id.to_string())))
}

fn parse_search_cursor(value: Option<&str>) -> DomainResult<Option<(i64, i64, String)>> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let mut parts = value.splitn(3, ':');
    let score_raw = parts.next().ok_or_else(|| {
        DomainError::Validation(
            "invalid cursor format; expected <score>:<occurred_at_ms>:<feed_id>".into(),
        )
    })?;
    let occurred_raw = parts.next().ok_or_else(|| {
        DomainError::Validation(
            "invalid cursor format; expected <score>:<occurred_at_ms>:<feed_id>".into(),
        )
    })?;
    let feed_id = parts.next().ok_or_else(|| {
        DomainError::Validation(
            "invalid cursor format; expected <score>:<occurred_at_ms>:<feed_id>".into(),
        )
    })?;
    let score = score_raw
        .parse::<i64>()
        .map_err(|_| DomainError::Validation("invalid cursor format".into()))?;
    let occurred_at_ms = occurred_raw
        .parse::<i64>()
        .map_err(|_| DomainError::Validation("invalid cursor format".into()))?;
    if feed_id.trim().is_empty() {
        return Err(DomainError::Validation("invalid cursor format".into()));
    }
    Ok(Some((score, occurred_at_ms, feed_id.to_string())))
}

fn parse_notification_cursor(value: Option<&str>) -> DomainResult<(Option<i64>, Option<String>)> {
    parse_feed_cursor(value)
}

fn make_feed_cursor(occurred_at_ms: i64, feed_id: &str) -> String {
    format!("{occurred_at_ms}:{feed_id}")
}

fn make_search_cursor(score: i64, occurred_at_ms: i64, feed_id: &str) -> String {
    format!("{score}:{occurred_at_ms}:{feed_id}")
}

fn make_notification_cursor(created_at_ms: i64, notification_id: &str) -> String {
    make_feed_cursor(created_at_ms, notification_id)
}

fn search_result_older(
    result: &SearchResult,
    cursor_score: i64,
    cursor_ms: i64,
    cursor_feed_id: &str,
) -> bool {
    if result.score < cursor_score {
        return true;
    }
    if result.score > cursor_score {
        return false;
    }
    if result.item.occurred_at_ms < cursor_ms {
        return true;
    }
    if result.item.occurred_at_ms > cursor_ms {
        return false;
    }
    result.item.feed_id.as_str() < cursor_feed_id
}

fn matches_search_text(item: &FeedItem, query: &str) -> bool {
    let query = query.to_lowercase();
    let in_title = item.title.to_lowercase().contains(&query);
    let in_summary = item
        .summary
        .as_ref()
        .is_some_and(|summary| summary.to_lowercase().contains(&query));
    let in_actor = item.actor_username.to_lowercase().contains(&query);
    in_title || in_summary || in_actor
}

fn score_query_match(item: &FeedItem, query: &str) -> i64 {
    let query = query.to_lowercase();
    let mut score = 0;
    let title = item.title.to_lowercase();
    if title.contains(&query) {
        score += 20;
    }
    if item
        .summary
        .as_ref()
        .is_some_and(|summary| summary.to_lowercase().contains(&query))
    {
        score += 8;
    }
    if item.actor_username.to_lowercase().contains(&query) {
        score += 2;
    }
    score
}

fn is_open_privacy_level(level: Option<&str>) -> bool {
    let level = level.unwrap_or("public").trim().to_ascii_lowercase();
    OPEN_PRIVACY_LEVELS.contains(&level.as_str())
}

fn is_hidden_feed_item(item: &FeedItem) -> bool {
    item.payload
        .as_ref()
        .and_then(|payload| payload.get("lifecycle"))
        .and_then(|lifecycle| lifecycle.get("hidden"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn is_visible_to_actor(actor_id: &str, item: &FeedItem) -> bool {
    if is_hidden_feed_item(item) {
        return false;
    }
    is_open_privacy_level(item.privacy_level.as_deref())
        || actor_id == item.actor_id
        || item.participant_ids.iter().any(|id| id == actor_id)
}

fn is_visible_notification(actor_id: &str, notification: &InAppNotification) -> bool {
    is_open_privacy_level(notification.privacy_level.as_deref())
        || actor_id == notification.user_id
        || actor_id == notification.actor_id
}

#[derive(Clone, Debug)]
struct SuggestionCandidate {
    entity_id: Option<String>,
    entity_type: String,
    label: String,
    followed: bool,
    description: Option<String>,
    follower_count: Option<usize>,
}

#[derive(Clone, Debug)]
struct SuggestionAggregate {
    suggestion: FeedSuggestion,
    seen_sources: HashSet<String>,
}

fn extract_suggestion_candidates(payload: Option<&serde_json::Value>) -> Vec<SuggestionCandidate> {
    let Some(payload) = payload else {
        return Vec::new();
    };
    let raw_tags = payload
        .get("enrichment")
        .and_then(|value| value.get("entity_tags"))
        .or_else(|| payload.get("entity_tags"))
        .and_then(serde_json::Value::as_array);
    let Some(raw_tags) = raw_tags else {
        return Vec::new();
    };

    raw_tags
        .iter()
        .filter_map(|raw_tag| {
            if let Some(label) = raw_tag
                .as_str()
                .map(str::trim)
                .filter(|label| !label.is_empty())
            {
                return Some(SuggestionCandidate {
                    entity_id: None,
                    entity_type: "topik".to_string(),
                    label: label.to_string(),
                    followed: false,
                    description: None,
                    follower_count: None,
                });
            }

            let tag = raw_tag.as_object()?;
            let label = tag
                .get("label")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|label| !label.is_empty())?
                .to_string();
            let entity_type = normalize_entity_type(
                tag.get("entity_type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("topik"),
            );
            let entity_id = tag
                .get("entity_id")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|entity_id| !entity_id.is_empty())
                .map(str::to_string);
            let followed = tag
                .get("followed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let description = tag
                .get("description")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|description| !description.is_empty())
                .map(str::to_string);
            let follower_count = tag.get("follower_count").and_then(to_usize);

            Some(SuggestionCandidate {
                entity_id,
                entity_type,
                label,
                followed,
                description,
                follower_count,
            })
        })
        .collect()
}

fn to_usize(value: &serde_json::Value) -> Option<usize> {
    if let Some(number) = value.as_u64() {
        return usize::try_from(number).ok();
    }
    value
        .as_str()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
}

fn normalize_entity_type(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "lingkungan" | "topik" | "kelompok" | "lembaga" | "warga" => normalized,
        _ => "topik".to_string(),
    }
}

fn is_suggestable_entity_type(entity_type: &str) -> bool {
    matches!(entity_type, "lingkungan" | "topik")
}

fn normalized_entity_id(entity_id: Option<&str>, entity_type: &str, label: &str) -> String {
    if let Some(entity_id) = entity_id
        .map(str::trim)
        .filter(|entity_id| !entity_id.is_empty())
    {
        return entity_id.to_string();
    }
    let slug = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        String::new()
    } else {
        format!("{entity_type}:{slug}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use crate::ports::discovery::{
        FeedRepository, FeedRepositoryQuery, FeedRepositorySearchQuery, NotificationRepository,
        NotificationRepositoryListQuery,
    };
    use std::sync::{Arc, Mutex};

    struct MockFeedRepository {
        persisted_item: Arc<Mutex<Option<FeedItem>>>,
        feed_rows: Arc<Mutex<Vec<FeedItem>>>,
        create_conflict: bool,
        participant_edge_upsert_calls: Arc<Mutex<Vec<String>>>,
    }

    impl MockFeedRepository {
        fn new(create_conflict: bool) -> Self {
            Self {
                persisted_item: Arc::new(Mutex::new(None)),
                feed_rows: Arc::new(Mutex::new(Vec::new())),
                create_conflict,
                participant_edge_upsert_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn set_persisted_item(&self, item: FeedItem) {
            self.persisted_item
                .lock()
                .expect("persisted_item mutex")
                .replace(item);
        }

        fn upserted_feed_ids(&self) -> Vec<String> {
            self.participant_edge_upsert_calls
                .lock()
                .expect("participant_edge_upsert_calls mutex")
                .clone()
        }

        fn set_feed_rows(&self, rows: Vec<FeedItem>) {
            self.feed_rows
                .lock()
                .expect("feed_rows mutex")
                .clone_from(&rows);
        }
    }

    impl FeedRepository for MockFeedRepository {
        fn create_feed_item(&self, item: &FeedItem) -> BoxFuture<'_, DomainResult<FeedItem>> {
            let item = item.clone();
            let persisted_item = self.persisted_item.clone();
            let create_conflict = self.create_conflict;
            Box::pin(async move {
                if create_conflict {
                    return Err(DomainError::Conflict);
                }
                persisted_item
                    .lock()
                    .expect("persisted_item mutex")
                    .replace(item.clone());
                Ok(item)
            })
        }

        fn upsert_participant_edges_for_item(
            &self,
            item: &FeedItem,
        ) -> BoxFuture<'_, DomainResult<()>> {
            let feed_id = item.feed_id.clone();
            let participant_edge_upsert_calls = self.participant_edge_upsert_calls.clone();
            Box::pin(async move {
                participant_edge_upsert_calls
                    .lock()
                    .expect("participant_edge_upsert_calls mutex")
                    .push(feed_id);
                Ok(())
            })
        }

        fn get_by_source_request(
            &self,
            _source_type: &str,
            _source_id: &str,
            _request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<FeedItem>>> {
            let persisted_item = self.persisted_item.clone();
            Box::pin(
                async move { Ok(persisted_item.lock().expect("persisted_item mutex").clone()) },
            )
        }

        fn get_by_feed_id(&self, feed_id: &str) -> BoxFuture<'_, DomainResult<Option<FeedItem>>> {
            let feed_id = feed_id.to_string();
            let persisted_item = self.persisted_item.clone();
            Box::pin(async move {
                Ok(persisted_item
                    .lock()
                    .expect("persisted_item mutex")
                    .as_ref()
                    .filter(|item| item.feed_id == feed_id)
                    .cloned())
            })
        }

        fn get_latest_by_source(
            &self,
            source_type: &str,
            source_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<FeedItem>>> {
            let source_type = source_type.to_string();
            let source_id = source_id.to_string();
            let persisted_item = self.persisted_item.clone();
            Box::pin(async move {
                Ok(persisted_item
                    .lock()
                    .expect("persisted_item mutex")
                    .as_ref()
                    .filter(|item| item.source_type == source_type && item.source_id == source_id)
                    .cloned())
            })
        }

        fn merge_payload(
            &self,
            feed_id: &str,
            payload_patch: serde_json::Value,
        ) -> BoxFuture<'_, DomainResult<FeedItem>> {
            let feed_id = feed_id.to_string();
            let persisted_item = self.persisted_item.clone();
            Box::pin(async move {
                let mut guard = persisted_item.lock().expect("persisted_item mutex");
                let Some(item) = guard.as_mut() else {
                    return Err(DomainError::NotFound);
                };
                if item.feed_id != feed_id {
                    return Err(DomainError::NotFound);
                }
                item.payload = Some(payload_patch);
                Ok(item.clone())
            })
        }

        fn list_feed(
            &self,
            _query: &FeedRepositoryQuery,
        ) -> BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
            let rows = self.feed_rows.clone();
            Box::pin(async move { Ok(rows.lock().expect("feed_rows mutex").clone()) })
        }

        fn search_feed(
            &self,
            _query: &FeedRepositorySearchQuery,
        ) -> BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
    }

    struct MockNotificationRepository;

    impl NotificationRepository for MockNotificationRepository {
        fn create_notification(
            &self,
            notification: &InAppNotification,
        ) -> BoxFuture<'_, DomainResult<InAppNotification>> {
            let notification = notification.clone();
            Box::pin(async move { Ok(notification) })
        }

        fn get_by_dedupe_key(
            &self,
            _user_id: &str,
            _dedupe_key: &str,
        ) -> BoxFuture<'_, DomainResult<Option<InAppNotification>>> {
            Box::pin(async move { Ok(None) })
        }

        fn list_notifications(
            &self,
            _query: &NotificationRepositoryListQuery,
        ) -> BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn list_notifications_in_window(
            &self,
            _user_id: &str,
            _window_start_ms: i64,
            _window_end_ms: i64,
        ) -> BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn mark_as_read(
            &self,
            _user_id: &str,
            _notification_id: &str,
            _read_at_ms: i64,
        ) -> BoxFuture<'_, DomainResult<InAppNotification>> {
            Box::pin(async move { Err(DomainError::NotFound) })
        }

        fn unread_count(&self, _user_id: &str) -> BoxFuture<'_, DomainResult<usize>> {
            Box::pin(async move { Ok(0) })
        }
    }

    #[test]
    fn parse_feed_cursor_rejects_invalid_shape() {
        assert!(parse_feed_cursor(Some("bad")).is_err());
    }

    #[test]
    fn parse_search_cursor_rejects_invalid_shape() {
        assert!(parse_search_cursor(Some("bad")).is_err());
    }

    #[test]
    fn validate_feed_input_rejects_empty_actor() {
        let actor = ActorIdentity {
            user_id: "".into(),
            username: "".into(),
        };
        let input = FeedIngestInput {
            source_type: FEED_SOURCE_CONTRIBUTION.into(),
            source_id: "s1".into(),
            actor,
            title: "title".into(),
            summary: None,
            scope_id: None,
            privacy_level: None,
            occurred_at_ms: None,
            request_id: "req-1".into(),
            correlation_id: "corr-1".into(),
            request_ts_ms: None,
            participant_ids: vec![],
            payload: None,
        };
        assert!(validate_feed_input(&input).is_err());
    }

    #[test]
    fn filter_visibility_open_is_true_without_actor() {
        let item = FeedItem {
            feed_id: "feed-1".into(),
            source_type: FEED_SOURCE_CONTRIBUTION.into(),
            source_id: "source-1".into(),
            actor_id: "author-1".into(),
            actor_username: "author".into(),
            title: "abc".into(),
            summary: None,
            scope_id: None,
            privacy_level: None,
            occurred_at_ms: 0,
            created_at_ms: 0,
            request_id: "r1".into(),
            correlation_id: "c1".into(),
            participant_ids: vec![],
            payload: None,
        };
        assert!(is_visible_to_actor("somebody", &item));
    }

    #[test]
    fn filter_visibility_hidden_feed_item_is_false() {
        let item = FeedItem {
            feed_id: "feed-1".into(),
            source_type: FEED_SOURCE_CONTRIBUTION.into(),
            source_id: "source-1".into(),
            actor_id: "author-1".into(),
            actor_username: "author".into(),
            title: "abc".into(),
            summary: None,
            scope_id: None,
            privacy_level: Some("public".into()),
            occurred_at_ms: 0,
            created_at_ms: 0,
            request_id: "r1".into(),
            correlation_id: "c1".into(),
            participant_ids: vec!["somebody".into()],
            payload: Some(serde_json::json!({
                "lifecycle": {
                    "hidden": true,
                    "hidden_reason": "ontology_ttl_expired"
                }
            })),
        };
        assert!(!is_visible_to_actor("somebody", &item));
    }

    #[test]
    fn search_cursor_ordering_filters() {
        let rows = vec![
            SearchResult {
                item: FeedItem {
                    feed_id: "a".into(),
                    source_type: FEED_SOURCE_CONTRIBUTION.into(),
                    source_id: "s1".into(),
                    actor_id: "a1".into(),
                    actor_username: "a".into(),
                    title: "hello".into(),
                    summary: None,
                    scope_id: None,
                    privacy_level: None,
                    occurred_at_ms: 100,
                    created_at_ms: 100,
                    request_id: "r".into(),
                    correlation_id: "c".into(),
                    participant_ids: vec![],
                    payload: None,
                },
                score: 3,
            },
            SearchResult {
                item: FeedItem {
                    feed_id: "b".into(),
                    source_type: FEED_SOURCE_CONTRIBUTION.into(),
                    source_id: "s2".into(),
                    actor_id: "a1".into(),
                    actor_username: "a".into(),
                    title: "hello".into(),
                    summary: None,
                    scope_id: None,
                    privacy_level: None,
                    occurred_at_ms: 100,
                    created_at_ms: 100,
                    request_id: "r".into(),
                    correlation_id: "c".into(),
                    participant_ids: vec![],
                    payload: None,
                },
                score: 2,
            },
        ];
        assert!(search_result_older(&rows[1], 3, 100, "a"));
        assert!(!search_result_older(&rows[0], 3, 100, "a"));
    }

    #[tokio::test]
    async fn ingest_feed_upserts_participant_edges_on_create() {
        let feed_repo = Arc::new(MockFeedRepository::new(false));
        let service =
            DiscoveryService::new(feed_repo.clone(), Arc::new(MockNotificationRepository));

        let result = service
            .ingest_feed(FeedIngestInput {
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: "seed-1".to_string(),
                actor: ActorIdentity {
                    user_id: "actor-1".to_string(),
                    username: "actor-1-name".to_string(),
                },
                title: "seed title".to_string(),
                summary: None,
                scope_id: Some("scope-1".to_string()),
                privacy_level: Some("public".to_string()),
                occurred_at_ms: Some(1_000),
                request_id: "req-1".to_string(),
                correlation_id: "corr-1".to_string(),
                request_ts_ms: Some(1_000),
                participant_ids: vec!["participant-1".to_string()],
                payload: None,
            })
            .await
            .expect("ingest feed should succeed");

        let upserted = feed_repo.upserted_feed_ids();
        assert_eq!(upserted.len(), 1);
        assert_eq!(upserted[0], result.feed_id);
    }

    #[tokio::test]
    async fn ingest_feed_upserts_participant_edges_on_replay_conflict() {
        let feed_repo = Arc::new(MockFeedRepository::new(true));
        feed_repo.set_persisted_item(FeedItem {
            feed_id: "feed-existing".to_string(),
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-existing".to_string(),
            actor_id: "actor-existing".to_string(),
            actor_username: "actor-existing-name".to_string(),
            title: "existing".to_string(),
            summary: None,
            scope_id: Some("scope-1".to_string()),
            privacy_level: Some("public".to_string()),
            occurred_at_ms: 2_000,
            created_at_ms: 2_000,
            request_id: "req-existing".to_string(),
            correlation_id: "corr-existing".to_string(),
            participant_ids: vec!["participant-existing".to_string()],
            payload: None,
        });
        let service =
            DiscoveryService::new(feed_repo.clone(), Arc::new(MockNotificationRepository));

        let result = service
            .ingest_feed(FeedIngestInput {
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: "seed-existing".to_string(),
                actor: ActorIdentity {
                    user_id: "actor-1".to_string(),
                    username: "actor-1-name".to_string(),
                },
                title: "seed title".to_string(),
                summary: None,
                scope_id: Some("scope-1".to_string()),
                privacy_level: Some("public".to_string()),
                occurred_at_ms: Some(1_000),
                request_id: "req-existing".to_string(),
                correlation_id: "corr-1".to_string(),
                request_ts_ms: Some(1_000),
                participant_ids: vec!["participant-1".to_string()],
                payload: None,
            })
            .await
            .expect("ingest feed replay should succeed");

        assert_eq!(result.feed_id, "feed-existing");
        let upserted = feed_repo.upserted_feed_ids();
        assert_eq!(upserted.len(), 1);
        assert_eq!(upserted[0], "feed-existing");
    }

    #[tokio::test]
    async fn list_feed_suggestions_aggregates_entity_tags() {
        let feed_repo = Arc::new(MockFeedRepository::new(false));
        feed_repo.set_feed_rows(vec![
            FeedItem {
                feed_id: "feed-public-1".to_string(),
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: "src-1".to_string(),
                actor_id: "author-1".to_string(),
                actor_username: "author-1".to_string(),
                title: "public one".to_string(),
                summary: None,
                scope_id: Some("scope-1".to_string()),
                privacy_level: Some("public".to_string()),
                occurred_at_ms: 1_000,
                created_at_ms: 1_000,
                request_id: "req-1".to_string(),
                correlation_id: "corr-1".to_string(),
                participant_ids: vec![],
                payload: Some(serde_json::json!({
                    "enrichment": {
                        "entity_tags": [
                            {
                                "entity_id": "ent-rt05",
                                "entity_type": "lingkungan",
                                "label": "RT 05 Menteng",
                                "follower_count": 40
                            },
                            {
                                "entity_type": "topik",
                                "label": "Infrastruktur"
                            }
                        ]
                    }
                })),
            },
            FeedItem {
                feed_id: "feed-public-2".to_string(),
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: "src-2".to_string(),
                actor_id: "author-2".to_string(),
                actor_username: "author-2".to_string(),
                title: "public two".to_string(),
                summary: None,
                scope_id: Some("scope-1".to_string()),
                privacy_level: Some("public".to_string()),
                occurred_at_ms: 900,
                created_at_ms: 900,
                request_id: "req-2".to_string(),
                correlation_id: "corr-2".to_string(),
                participant_ids: vec![],
                payload: Some(serde_json::json!({
                    "enrichment": {
                        "entity_tags": [
                            {
                                "entity_id": "ent-rt05",
                                "entity_type": "lingkungan",
                                "label": "RT 05 Menteng"
                            },
                            {
                                "entity_id": "ent-air",
                                "entity_type": "topik",
                                "label": "Saluran Air",
                                "followed": true
                            }
                        ]
                    }
                })),
            },
            FeedItem {
                feed_id: "feed-private".to_string(),
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: "src-3".to_string(),
                actor_id: "hidden-author".to_string(),
                actor_username: "hidden-author".to_string(),
                title: "private".to_string(),
                summary: None,
                scope_id: Some("scope-1".to_string()),
                privacy_level: Some("private".to_string()),
                occurred_at_ms: 800,
                created_at_ms: 800,
                request_id: "req-3".to_string(),
                correlation_id: "corr-3".to_string(),
                participant_ids: vec![],
                payload: Some(serde_json::json!({
                    "enrichment": {
                        "entity_tags": [
                            {
                                "entity_id": "ent-hidden",
                                "entity_type": "topik",
                                "label": "Hidden Topic"
                            }
                        ]
                    }
                })),
            },
        ]);
        let service = DiscoveryService::new(feed_repo, Arc::new(MockNotificationRepository));
        let suggestions = service
            .list_feed_suggestions(FeedSuggestionsQuery {
                actor_id: "reader-1".to_string(),
                limit: Some(6),
                scope_id: Some("scope-1".to_string()),
                privacy_level: None,
                from_ms: None,
                to_ms: None,
            })
            .await
            .expect("list suggestions should succeed");

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].entity_id, "ent-rt05");
        assert_eq!(suggestions[0].entity_type, "lingkungan");
        assert_eq!(suggestions[0].witness_count, 2);
        assert_eq!(suggestions[0].follower_count, 40);
        assert_eq!(suggestions[1].entity_type, "topik");
        assert_eq!(suggestions[1].label, "Infrastruktur");
        assert_eq!(suggestions[1].witness_count, 1);
        assert!(suggestions.iter().all(|item| !item.followed));
        assert!(suggestions.iter().all(|item| item.entity_id != "ent-air"));
        assert!(
            suggestions
                .iter()
                .all(|item| item.entity_id != "ent-hidden")
        );
    }
}
