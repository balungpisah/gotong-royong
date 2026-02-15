use crate::DomainResult;
use crate::discovery::{FeedItem, InAppNotification};
use crate::ports::BoxFuture;

#[derive(Clone, Debug)]
pub struct FeedRepositoryQuery {
    pub actor_id: String,
    pub cursor_occurred_at_ms: Option<i64>,
    pub cursor_feed_id: Option<String>,
    pub limit: usize,
    pub scope_id: Option<String>,
    pub track: Option<String>,
    pub stage: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: bool,
}

#[derive(Clone, Debug)]
pub struct FeedRepositorySearchQuery {
    pub actor_id: String,
    pub limit: usize,
    pub scope_id: Option<String>,
    pub track: Option<String>,
    pub stage: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: bool,
    pub exclude_vault: bool,
    pub query_text: String,
}

#[derive(Clone, Debug)]
pub struct NotificationRepositoryListQuery {
    pub user_id: String,
    pub cursor_created_at_ms: Option<i64>,
    pub cursor_notification_id: Option<String>,
    pub limit: usize,
    pub include_read: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub trait FeedRepository: Send + Sync {
    fn create_feed_item(&self, item: &FeedItem) -> BoxFuture<'_, DomainResult<FeedItem>>;

    fn get_by_source_request(
        &self,
        source_type: &str,
        source_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<FeedItem>>>;

    fn list_feed(&self, query: &FeedRepositoryQuery) -> BoxFuture<'_, DomainResult<Vec<FeedItem>>>;

    fn search_feed(
        &self,
        query: &FeedRepositorySearchQuery,
    ) -> BoxFuture<'_, DomainResult<Vec<FeedItem>>>;
}

#[allow(clippy::needless_pass_by_value)]
pub trait NotificationRepository: Send + Sync {
    fn create_notification(
        &self,
        notification: &InAppNotification,
    ) -> BoxFuture<'_, DomainResult<InAppNotification>>;

    fn get_by_dedupe_key(
        &self,
        user_id: &str,
        dedupe_key: &str,
    ) -> BoxFuture<'_, DomainResult<Option<InAppNotification>>>;

    fn list_notifications(
        &self,
        query: &NotificationRepositoryListQuery,
    ) -> BoxFuture<'_, DomainResult<Vec<InAppNotification>>>;

    fn list_notifications_in_window(
        &self,
        user_id: &str,
        window_start_ms: i64,
        window_end_ms: i64,
    ) -> BoxFuture<'_, DomainResult<Vec<InAppNotification>>>;

    fn mark_as_read(
        &self,
        user_id: &str,
        notification_id: &str,
        read_at_ms: i64,
    ) -> BoxFuture<'_, DomainResult<InAppNotification>>;

    fn unread_count(&self, user_id: &str) -> BoxFuture<'_, DomainResult<usize>>;
}
