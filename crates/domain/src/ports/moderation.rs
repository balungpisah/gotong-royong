use crate::DomainResult;
use crate::moderation::{ContentModeration, ModerationDecision};
use crate::ports::BoxFuture;

#[allow(clippy::needless_pass_by_value)]
pub trait ModerationRepository: Send + Sync {
    fn upsert_content_moderation(
        &self,
        content: &ContentModeration,
    ) -> BoxFuture<'_, DomainResult<ContentModeration>>;

    fn get_content_moderation(
        &self,
        content_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<ContentModeration>>>;

    fn list_content_by_status(
        &self,
        status: &str,
        limit: usize,
    ) -> BoxFuture<'_, DomainResult<Vec<ContentModeration>>>;

    fn create_decision(
        &self,
        decision: &ModerationDecision,
    ) -> BoxFuture<'_, DomainResult<ModerationDecision>>;

    fn get_decision_by_request(
        &self,
        content_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<ModerationDecision>>>;

    fn list_decisions(
        &self,
        content_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<ModerationDecision>>>;
}
