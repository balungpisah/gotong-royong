use crate::ports::BoxFuture;
use crate::webhook::{
    WebhookDeliveryLog, WebhookOutboxEvent, WebhookOutboxListQuery, WebhookOutboxUpdate,
};

use crate::DomainResult;

#[allow(clippy::needless_pass_by_value)]
pub trait WebhookOutboxRepository: Send + Sync {
    fn create(&self, event: &WebhookOutboxEvent)
    -> BoxFuture<'_, DomainResult<WebhookOutboxEvent>>;

    fn get(&self, event_id: &str) -> BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>>;

    fn get_by_request_id(
        &self,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>>;

    fn list(
        &self,
        query: &WebhookOutboxListQuery,
    ) -> BoxFuture<'_, DomainResult<Vec<WebhookOutboxEvent>>>;

    fn update(
        &self,
        event_id: &str,
        update: &WebhookOutboxUpdate,
    ) -> BoxFuture<'_, DomainResult<WebhookOutboxEvent>>;

    fn append_log(
        &self,
        log: &WebhookDeliveryLog,
    ) -> BoxFuture<'_, DomainResult<WebhookDeliveryLog>>;

    fn list_logs(&self, event_id: &str) -> BoxFuture<'_, DomainResult<Vec<WebhookDeliveryLog>>>;
}
