use crate::DomainResult;
use crate::ports::BoxFuture;
use crate::vault::{VaultEntry, VaultTimelineEvent};

#[allow(clippy::needless_pass_by_value)]
pub trait VaultRepository: Send + Sync {
    fn create_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> BoxFuture<'_, DomainResult<VaultEntry>>;

    fn update_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> BoxFuture<'_, DomainResult<VaultEntry>>;

    fn delete_entry(&self, vault_entry_id: &str) -> BoxFuture<'_, DomainResult<bool>>;

    fn get_entry(&self, vault_entry_id: &str) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>>;

    fn list_by_author(&self, author_id: &str) -> BoxFuture<'_, DomainResult<Vec<VaultEntry>>>;

    fn list_timeline(
        &self,
        vault_entry_id: &str,
    ) -> BoxFuture<'_, DomainResult<Vec<VaultTimelineEvent>>>;

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>>;

    fn get_by_request(
        &self,
        vault_entry_id: &str,
        request_id: &str,
    ) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>>;
}
