use crate::DomainResult;
use crate::ports::BoxFuture;
use crate::transitions::TrackStateTransition;

#[allow(clippy::needless_pass_by_value)]
pub trait TrackTransitionRepository: Send + Sync {
    fn create(
        &self,
        transition: &TrackStateTransition,
    ) -> BoxFuture<'_, DomainResult<TrackStateTransition>>;

    fn get_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<TrackStateTransition>>>;

    fn get_by_transition_id(
        &self,
        transition_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<TrackStateTransition>>>;

    fn list_by_entity(
        &self,
        entity_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<TrackStateTransition>>>;
}
