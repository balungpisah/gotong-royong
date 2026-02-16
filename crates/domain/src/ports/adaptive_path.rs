use crate::DomainResult;
use crate::adaptive_path::{
    AdaptivePathEvent, AdaptivePathPlan, AdaptivePathSuggestion, SuggestionDecisionStatus,
};
use crate::ports::BoxFuture;

#[allow(clippy::needless_pass_by_value)]
pub trait AdaptivePathRepository: Send + Sync {
    fn create_plan(&self, plan: &AdaptivePathPlan)
    -> BoxFuture<'_, DomainResult<AdaptivePathPlan>>;

    fn get_plan(&self, plan_id: &str) -> BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>>;

    fn get_plan_by_entity(
        &self,
        entity_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>>;

    fn get_plan_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>>;

    fn update_plan(&self, plan: &AdaptivePathPlan)
    -> BoxFuture<'_, DomainResult<AdaptivePathPlan>>;

    fn create_event(
        &self,
        event: &AdaptivePathEvent,
    ) -> BoxFuture<'_, DomainResult<AdaptivePathEvent>>;

    fn list_events(&self, plan_id: &str) -> BoxFuture<'_, DomainResult<Vec<AdaptivePathEvent>>>;

    fn get_event_by_request_id(
        &self,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<AdaptivePathEvent>>>;

    fn create_suggestion(
        &self,
        suggestion: &AdaptivePathSuggestion,
    ) -> BoxFuture<'_, DomainResult<AdaptivePathSuggestion>>;

    fn list_suggestions(
        &self,
        plan_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<AdaptivePathSuggestion>>>;

    fn get_suggestion(
        &self,
        suggestion_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>>;

    fn get_suggestion_by_request_id(
        &self,
        plan_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>>;

    fn update_suggestion_status(
        &self,
        suggestion_id: &str,
        status: SuggestionDecisionStatus,
    ) -> BoxFuture<'_, DomainResult<AdaptivePathSuggestion>>;
}
