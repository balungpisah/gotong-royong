use crate::contributions::Contribution;
use crate::ports::BoxFuture;

use crate::DomainResult;

#[allow(clippy::needless_pass_by_value)]
pub trait ContributionRepository: Send + Sync {
    fn create(&self, contribution: &Contribution) -> BoxFuture<'_, DomainResult<Contribution>>;

    fn get(&self, contribution_id: &str) -> BoxFuture<'_, DomainResult<Option<Contribution>>>;

    fn list_by_author(&self, author_id: &str) -> BoxFuture<'_, DomainResult<Vec<Contribution>>>;

    fn list_recent(
        &self,
        author_id: &str,
        limit: usize,
    ) -> BoxFuture<'_, DomainResult<Vec<Contribution>>>;
}
