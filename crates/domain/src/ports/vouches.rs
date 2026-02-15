use crate::ports::BoxFuture;
use crate::vouches::Vouch;

use crate::DomainResult;

#[allow(clippy::needless_pass_by_value)]
pub trait VouchRepository: Send + Sync {
    fn create(&self, vouch: &Vouch) -> BoxFuture<'_, DomainResult<Vouch>>;

    fn list_by_vouchee(&self, vouchee_id: &str) -> BoxFuture<'_, DomainResult<Vec<Vouch>>>;

    fn list_by_voucher(&self, voucher_id: &str) -> BoxFuture<'_, DomainResult<Vec<Vouch>>>;
}
