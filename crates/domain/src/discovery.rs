use std::collections::HashSet;
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
const ONE_WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;

pub const FEED_SOURCE_CONTRIBUTION: &str = "contribution";
pub const FEED_SOURCE_TRANSITION: &str = "transition";
pub const FEED_SOURCE_VAULT: &str = "vault";
pub const FEED_SOURCE_SIAGA: &str = "siaga";
pub const FEED_SOURCE_MODERATION: &str = "moderation";
pub const FEED_SOURCE_VOUCH: &str = "vouch";

pub const NOTIF_TYPE_TRANSITION: &str = "transition";
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
    pub track: Option<String>,
    pub stage: Option<String>,
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
    pub track: Option<String>,
    pub stage: Option<String>,
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
    pub track: Option<String>,
    pub stage: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: bool,
}

#[derive(Clone)]
pub struct SearchListQuery {
    pub actor_id: String,
    pub query_text: String,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub track: Option<String>,
    pub stage: Option<String>,
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
            track: input.track,
            stage: input.stage,
            scope_id: input.scope_id,
            privacy_level: input.privacy_level,
            occurred_at_ms: input.occurred_at_ms.unwrap_or_else(now_ms),
            created_at_ms: input.request_ts_ms.unwrap_or_else(now_ms),
            request_id: input.request_id,
            correlation_id: input.correlation_id,
            participant_ids: dedupe_vec(input.participant_ids),
            payload: input.payload,
        };

        match self.feed_repo.create_feed_item(&item).await {
            Ok(item) => Ok(item),
            Err(DomainError::Conflict) => self
                .feed_repo
                .get_by_source_request(&item.source_type, &item.source_id, &item.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
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
                track: query.track.clone(),
                stage: query.stage.clone(),
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
                track: query.track.clone(),
                stage: query.stage.clone(),
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
            .get(limit)
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
                    track: None,
                    stage: None,
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
    if item.track.is_some() {
        score += 1;
    }
    score
}

fn is_open_privacy_level(level: Option<&str>) -> bool {
    let level = level.unwrap_or("public").trim().to_ascii_lowercase();
    OPEN_PRIVACY_LEVELS.contains(&level.as_str())
}

fn is_visible_to_actor(actor_id: &str, item: &FeedItem) -> bool {
    is_open_privacy_level(item.privacy_level.as_deref())
        || actor_id == item.actor_id
        || item.participant_ids.iter().any(|id| id == actor_id)
}

fn is_visible_notification(actor_id: &str, notification: &InAppNotification) -> bool {
    is_open_privacy_level(notification.privacy_level.as_deref())
        || actor_id == notification.user_id
        || actor_id == notification.actor_id
}

#[cfg(test)]
mod tests {
    use super::*;

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
            track: None,
            stage: None,
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
            track: None,
            stage: None,
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
    fn search_cursor_ordering_filters() {
        let rows = vec![
            SearchResult {
                item: FeedItem {
                    feed_id: "a".into(),
                    source_type: FEED_SOURCE_TRANSITION.into(),
                    source_id: "s1".into(),
                    actor_id: "a1".into(),
                    actor_username: "a".into(),
                    title: "hello".into(),
                    summary: None,
                    track: None,
                    stage: None,
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
                    source_type: FEED_SOURCE_TRANSITION.into(),
                    source_id: "s2".into(),
                    actor_id: "a1".into(),
                    actor_username: "a".into(),
                    title: "hello".into(),
                    summary: None,
                    track: None,
                    stage: None,
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
}
