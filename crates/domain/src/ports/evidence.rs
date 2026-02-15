use crate::evidence::Evidence;
use crate::ports::BoxFuture;

use crate::DomainResult;

#[allow(clippy::needless_pass_by_value)]
pub trait EvidenceRepository: Send + Sync {
    fn create(&self, evidence: &Evidence) -> BoxFuture<'_, DomainResult<Evidence>>;

    fn get(&self, evidence_id: &str) -> BoxFuture<'_, DomainResult<Option<Evidence>>>;

    fn list_by_contribution(
        &self,
        contribution_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<Evidence>>>;
}
