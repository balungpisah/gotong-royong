use crate::DomainResult;
use crate::ports::BoxFuture;
use crate::siaga::{SiagaBroadcast, SiagaTimelineEvent};

#[allow(clippy::needless_pass_by_value)]
pub trait SiagaRepository: Send + Sync {
    fn create_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> BoxFuture<'_, DomainResult<SiagaBroadcast>>;

    fn update_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> BoxFuture<'_, DomainResult<SiagaBroadcast>>;

    fn get_broadcast(&self, siaga_id: &str) -> BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>>;

    fn list_by_scope(&self, scope_id: &str) -> BoxFuture<'_, DomainResult<Vec<SiagaBroadcast>>>;

    fn list_timeline(&self, siaga_id: &str)
    -> BoxFuture<'_, DomainResult<Vec<SiagaTimelineEvent>>>;

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>>;

    fn get_by_request(
        &self,
        siaga_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>>;
}
