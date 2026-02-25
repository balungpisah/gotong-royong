use crate::DomainResult;
use crate::ports::BoxFuture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMemberRecord {
    pub user_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupJoinRequestRecord {
    pub request_id: String,
    pub user_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub requested_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupRecord {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub entity_type: String,
    pub join_policy: String,
    pub member_count: usize,
    pub witness_count: usize,
    pub members: Vec<GroupMemberRecord>,
    pub pending_requests: Vec<GroupJoinRequestRecord>,
    pub updated_at_ms: i64,
}

#[allow(clippy::needless_pass_by_value)]
pub trait GroupRepository: Send + Sync {
    fn create_group(&self, group: &GroupRecord) -> BoxFuture<'_, DomainResult<GroupRecord>>;
    fn get_group(&self, group_id: &str) -> BoxFuture<'_, DomainResult<Option<GroupRecord>>>;
    fn list_groups(&self) -> BoxFuture<'_, DomainResult<Vec<GroupRecord>>>;
    fn update_group(&self, group: &GroupRecord) -> BoxFuture<'_, DomainResult<GroupRecord>>;
}
