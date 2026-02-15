use crate::DomainResult;
use crate::chat::{
    ChatDeliveryEvent, ChatMember, ChatMessage, ChatReadCursor, ChatThread, ChatThreadQuery,
    ChatThreadWithMembers, MessageCatchup,
};

#[allow(clippy::needless_pass_by_value)]
pub trait ChatRepository: Send + Sync {
    fn create_thread(
        &self,
        thread: &ChatThread,
    ) -> crate::ports::BoxFuture<'_, DomainResult<ChatThread>>;

    fn get_thread(
        &self,
        thread_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatThread>>>;

    fn list_threads_by_scope(
        &self,
        query: &ChatThreadQuery,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Vec<ChatThread>>>;

    fn list_threads_by_user(
        &self,
        user_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Vec<ChatThreadWithMembers>>>;

    fn create_member(
        &self,
        member: &ChatMember,
    ) -> crate::ports::BoxFuture<'_, DomainResult<ChatMember>>;

    fn list_members(
        &self,
        thread_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Vec<ChatMember>>>;

    fn get_member(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatMember>>>;

    fn create_message(
        &self,
        message: &ChatMessage,
    ) -> crate::ports::BoxFuture<'_, DomainResult<ChatMessage>>;

    fn get_message(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>>;

    fn get_message_by_request_id(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>>;

    fn list_messages(
        &self,
        thread_id: &str,
        cursor: &MessageCatchup,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Vec<ChatMessage>>>;

    fn set_read_cursor(
        &self,
        cursor: &ChatReadCursor,
    ) -> crate::ports::BoxFuture<'_, DomainResult<ChatReadCursor>>;

    fn get_read_cursor(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatReadCursor>>>;

    fn create_delivery_event(
        &self,
        event: &ChatDeliveryEvent,
    ) -> crate::ports::BoxFuture<'_, DomainResult<ChatDeliveryEvent>>;

    fn get_delivery_event_by_request(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> crate::ports::BoxFuture<'_, DomainResult<Option<ChatDeliveryEvent>>>;
}
