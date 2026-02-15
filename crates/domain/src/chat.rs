use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::chat::ChatRepository;

const MAX_BODY_LENGTH: usize = 2_000;
const MAX_ATTACHMENT_COUNT: usize = 20;
const MAX_MESSAGES_PER_REQUEST: usize = 200;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatMemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ChatThread {
    pub thread_id: String,
    pub scope_id: String,
    pub created_by: String,
    pub privacy_level: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatThreadWithMembers {
    pub thread: ChatThread,
    pub member_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMember {
    pub thread_id: String,
    pub user_id: String,
    pub role: ChatMemberRole,
    pub joined_at_ms: i64,
    pub left_at_ms: Option<i64>,
    pub mute_until_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub thread_id: String,
    pub message_id: String,
    pub author_id: String,
    pub body: String,
    pub attachments: Vec<serde_json::Value>,
    pub created_at_ms: i64,
    pub edited_at_ms: Option<i64>,
    pub deleted_at_ms: Option<i64>,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatReadCursor {
    pub thread_id: String,
    pub user_id: String,
    pub last_read_message_id: String,
    pub last_read_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatDeliveryEvent {
    pub event_id: String,
    pub thread_id: String,
    pub message_id: String,
    pub event_type: String,
    pub occurred_at_ms: i64,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatThreadCreate {
    pub scope_id: String,
    pub privacy_level: String,
}

#[derive(Clone, Debug)]
pub struct ChatThreadQuery {
    pub scope_id: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SendMessageInput {
    pub thread_id: String,
    pub body: String,
    pub attachments: Vec<serde_json::Value>,
    pub request_id: String,
    pub correlation_id: String,
    pub occurred_at_ms: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct MessageCatchup {
    pub since_created_at_ms: Option<i64>,
    pub since_message_id: Option<String>,
    pub limit: usize,
}

#[derive(Clone)]
pub struct ChatService {
    repository: Arc<dyn ChatRepository>,
}

impl ChatService {
    pub fn new(repository: Arc<dyn ChatRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_thread(
        &self,
        actor: &ActorIdentity,
        request_id: String,
        correlation_id: String,
        input: ChatThreadCreate,
    ) -> DomainResult<ChatThread> {
        let input = validate_thread_create_input(input)?;
        let now = now_ms();
        let thread_id = crate::util::uuid_v7_without_dashes();

        let thread = ChatThread {
            thread_id: thread_id.clone(),
            scope_id: input.scope_id,
            created_by: actor.user_id.clone(),
            privacy_level: input.privacy_level,
            created_at_ms: now,
            updated_at_ms: now,
        };

        let thread = self.repository.create_thread(&thread).await?;
        let thread_id = thread.thread_id.clone();
        let owner = ChatMember {
            thread_id: thread_id.clone(),
            user_id: actor.user_id.clone(),
            role: ChatMemberRole::Owner,
            joined_at_ms: now,
            left_at_ms: None,
            mute_until_ms: None,
        };
        let _ = self.repository.create_member(&owner).await?;

        let _ = self
            .repository
            .create_delivery_event(&ChatDeliveryEvent {
                event_id: crate::util::uuid_v7_without_dashes(),
                thread_id: thread_id.clone(),
                message_id: format!("thread:{thread_id}"),
                event_type: "thread_created".to_string(),
                occurred_at_ms: now,
                request_id,
                correlation_id,
            })
            .await?;

        Ok(thread)
    }

    pub async fn get_thread(&self, thread_id: &str) -> DomainResult<ChatThread> {
        self.repository
            .get_thread(thread_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn list_threads_by_user(
        &self,
        actor: &ActorIdentity,
    ) -> DomainResult<Vec<ChatThread>> {
        let threads = self.repository.list_threads_by_user(&actor.user_id).await?;
        Ok(threads.into_iter().map(|item| item.thread).collect())
    }

    pub async fn list_threads_by_user_with_members(
        &self,
        actor: &ActorIdentity,
    ) -> DomainResult<Vec<ChatThreadWithMembers>> {
        self.repository.list_threads_by_user(&actor.user_id).await
    }

    pub async fn list_threads_by_scope(
        &self,
        actor: &ActorIdentity,
        scope_id: &str,
    ) -> DomainResult<Vec<ChatThread>> {
        let query = ChatThreadQuery {
            scope_id: Some(scope_id.to_string()),
            actor_id: Some(actor.user_id.clone()),
        };
        self.repository.list_threads_by_scope(&query).await
    }

    pub async fn join_thread(
        &self,
        actor: &ActorIdentity,
        thread_id: &str,
    ) -> DomainResult<ChatMember> {
        let now = now_ms();
        self.get_thread(thread_id).await?;
        let existing = self
            .repository
            .get_member(thread_id, &actor.user_id)
            .await?;

        let Some(member) = existing else {
            let member = ChatMember {
                thread_id: thread_id.to_string(),
                user_id: actor.user_id.clone(),
                role: ChatMemberRole::Member,
                joined_at_ms: now,
                left_at_ms: None,
                mute_until_ms: None,
            };
            return self.repository.create_member(&member).await;
        };

        if member.left_at_ms.is_none() {
            return Ok(member);
        }

        let member = ChatMember {
            thread_id: thread_id.to_string(),
            user_id: actor.user_id.clone(),
            role: member.role,
            joined_at_ms: now,
            left_at_ms: None,
            mute_until_ms: member.mute_until_ms,
        };
        self.repository.create_member(&member).await
    }

    pub async fn leave_thread(
        &self,
        actor: &ActorIdentity,
        thread_id: &str,
    ) -> DomainResult<ChatMember> {
        let now = now_ms();
        let mut member = self
            .repository
            .get_member(thread_id, &actor.user_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if member.left_at_ms.is_some() {
            return Ok(member);
        }

        member.left_at_ms = Some(now);
        self.repository.create_member(&member).await
    }

    pub async fn list_members(&self, thread_id: &str) -> DomainResult<Vec<ChatMember>> {
        self.repository.list_members(thread_id).await
    }

    pub async fn send_message(
        &self,
        actor: &ActorIdentity,
        payload: SendMessageInput,
    ) -> DomainResult<ChatMessage> {
        self.get_thread(&payload.thread_id).await?;
        self.assert_actor_can_send_message(&payload.thread_id, actor)
            .await?;

        let body = payload.body.trim().to_string();
        validate_message_input(&body, &payload.attachments)?;
        if let Some(existing) = self
            .repository
            .get_message_by_request_id(&payload.thread_id, &payload.request_id)
            .await?
        {
            return Ok(existing);
        }

        let occurred_at_ms = payload.occurred_at_ms.unwrap_or_else(now_ms);
        let message = ChatMessage {
            thread_id: payload.thread_id,
            message_id: crate::util::uuid_v7_without_dashes(),
            author_id: actor.user_id.clone(),
            body,
            attachments: payload.attachments,
            created_at_ms: occurred_at_ms,
            edited_at_ms: None,
            deleted_at_ms: None,
            request_id: payload.request_id,
            correlation_id: payload.correlation_id,
        };

        let message = self.repository.create_message(&message).await?;

        let _ = self
            .repository
            .create_delivery_event(&ChatDeliveryEvent {
                event_id: crate::util::uuid_v7_without_dashes(),
                thread_id: message.thread_id.clone(),
                message_id: message.message_id.clone(),
                event_type: "message_created".to_string(),
                occurred_at_ms: message.created_at_ms,
                request_id: message.request_id.clone(),
                correlation_id: message.correlation_id.clone(),
            })
            .await;

        Ok(message)
    }

    pub async fn list_messages(
        &self,
        thread_id: &str,
        actor: &ActorIdentity,
        cursor: MessageCatchup,
    ) -> DomainResult<Vec<ChatMessage>> {
        self.assert_actor_can_send_message(thread_id, actor).await?;
        self.repository.list_messages(thread_id, &cursor).await
    }

    pub async fn mark_read(
        &self,
        actor: &ActorIdentity,
        thread_id: &str,
        message_id: String,
    ) -> DomainResult<ChatReadCursor> {
        self.assert_actor_can_send_message(thread_id, actor).await?;
        let last_message = self
            .repository
            .get_message(thread_id, &message_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        let cursor = ChatReadCursor {
            thread_id: thread_id.to_string(),
            user_id: actor.user_id.clone(),
            last_read_message_id: last_message.message_id,
            last_read_at_ms: now_ms(),
        };
        self.repository.set_read_cursor(&cursor).await
    }

    pub async fn get_read_cursor(
        &self,
        actor: &ActorIdentity,
        thread_id: &str,
    ) -> DomainResult<ChatReadCursor> {
        self.assert_actor_can_send_message(thread_id, actor).await?;
        self.repository
            .get_read_cursor(thread_id, &actor.user_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn assert_actor_is_member(
        &self,
        actor: &ActorIdentity,
        thread_id: &str,
    ) -> DomainResult<()> {
        self.assert_actor_can_send_message(thread_id, actor).await
    }

    async fn assert_actor_can_send_message(
        &self,
        thread_id: &str,
        actor: &ActorIdentity,
    ) -> DomainResult<()> {
        let member = self
            .repository
            .get_member(thread_id, &actor.user_id)
            .await?
            .ok_or_else(|| DomainError::Validation("user is not a member of this thread".into()))?;

        if member.left_at_ms.is_some() {
            return Err(DomainError::Validation(
                "membership in thread has ended".into(),
            ));
        }

        if let Some(mute_until_ms) = member.mute_until_ms {
            if now_ms() < mute_until_ms {
                return Err(DomainError::Validation(
                    "member is currently muted in this thread".into(),
                ));
            }
        }
        Ok(())
    }
}

fn validate_thread_create_input(mut input: ChatThreadCreate) -> DomainResult<ChatThreadCreate> {
    input.scope_id = input.scope_id.trim().to_string();
    input.privacy_level = input.privacy_level.trim().to_lowercase();

    if input.scope_id.is_empty() {
        return Err(DomainError::Validation("scope_id is required".into()));
    }

    match input.privacy_level.as_str() {
        "public" | "private" => Ok(input),
        _ => Err(DomainError::Validation(
            "privacy_level must be public or private".into(),
        )),
    }
}

fn validate_message_input(body: &str, attachments: &[serde_json::Value]) -> DomainResult<()> {
    if body.is_empty() {
        return Err(DomainError::Validation("body is required".into()));
    }

    if body.chars().count() > MAX_BODY_LENGTH {
        return Err(DomainError::Validation(format!(
            "body exceeds max length of {MAX_BODY_LENGTH}"
        )));
    }

    if attachments.len() > MAX_ATTACHMENT_COUNT {
        return Err(DomainError::Validation(format!(
            "attachments exceeds max of {MAX_ATTACHMENT_COUNT}"
        )));
    }

    Ok(())
}

pub fn build_message_catchup(
    limit: Option<usize>,
    since_created_at_ms: Option<i64>,
    since_message_id: Option<String>,
) -> MessageCatchup {
    let safe_limit = limit.unwrap_or(50).clamp(1, MAX_MESSAGES_PER_REQUEST);
    MessageCatchup {
        since_created_at_ms,
        since_message_id,
        limit: safe_limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use crate::ports::chat::ChatRepository;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockChatRepo {
        threads: Arc<RwLock<HashMap<String, ChatThread>>>,
        members: Arc<RwLock<HashMap<(String, String), ChatMember>>>,
        messages: Arc<RwLock<HashMap<(String, String), ChatMessage>>>,
        by_request: Arc<RwLock<HashMap<(String, String), String>>>,
        cursors: Arc<RwLock<HashMap<(String, String), ChatReadCursor>>>,
        events: Arc<RwLock<HashMap<(String, String), ChatDeliveryEvent>>>,
    }

    impl ChatRepository for MockChatRepo {
        fn create_thread(&self, thread: &ChatThread) -> BoxFuture<'_, DomainResult<ChatThread>> {
            let thread = thread.clone();
            let threads = self.threads.clone();
            Box::pin(async move {
                let mut threads = threads.write().await;
                if threads.contains_key(&thread.thread_id) {
                    return Err(DomainError::Conflict);
                }
                threads.insert(thread.thread_id.clone(), thread.clone());
                Ok(thread)
            })
        }

        fn get_thread(&self, thread_id: &str) -> BoxFuture<'_, DomainResult<Option<ChatThread>>> {
            let thread_id = thread_id.to_string();
            let threads = self.threads.clone();
            Box::pin(async move {
                let threads = threads.read().await;
                Ok(threads.get(&thread_id).cloned())
            })
        }

        fn list_threads_by_scope(
            &self,
            query: &ChatThreadQuery,
        ) -> BoxFuture<'_, DomainResult<Vec<ChatThread>>> {
            let query = query.clone();
            let threads = self.threads.clone();
            let members = self.members.clone();
            Box::pin(async move {
                let allowed_thread_ids = if let Some(actor_id) = query.actor_id.as_ref() {
                    let members = members.read().await;
                    let allowed: std::collections::HashSet<String> = members
                        .iter()
                        .filter_map(|((thread_id, member_user), member)| {
                            if member_user == actor_id && member.left_at_ms.is_none() {
                                Some(thread_id.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    Some(allowed)
                } else {
                    None
                };

                let mut output: Vec<_> = threads
                    .read()
                    .await
                    .values()
                    .filter(|thread| {
                        if query.scope_id.as_ref() != Some(&thread.scope_id) {
                            return false;
                        }

                        if thread.privacy_level == "public" {
                            return true;
                        }

                        if let Some(allowed_thread_ids) = allowed_thread_ids.as_ref() {
                            return allowed_thread_ids.contains(&thread.thread_id);
                        }

                        false
                    })
                    .cloned()
                    .collect();

                output.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
                Ok(output)
            })
        }

        fn list_threads_by_user(
            &self,
            user_id: &str,
        ) -> BoxFuture<'_, DomainResult<Vec<ChatThreadWithMembers>>> {
            let user_id = user_id.to_string();
            let threads = self.threads.clone();
            let members = self.members.clone();
            Box::pin(async move {
                let members = members.read().await;
                let thread_ids: Vec<String> = members
                    .iter()
                    .filter(|((_, member_user), member)| {
                        member_user == &user_id && member.left_at_ms.is_none()
                    })
                    .map(|((thread_id, _), _)| thread_id.clone())
                    .collect();

                let threads = threads.read().await;
                let mut thread_list = Vec::with_capacity(thread_ids.len());
                for thread_id in thread_ids {
                    let Some(thread) = threads.get(&thread_id).cloned() else {
                        continue;
                    };
                    let member_count = members
                        .values()
                        .filter(|member| {
                            member.thread_id == thread_id && member.left_at_ms.is_none()
                        })
                        .count();
                    thread_list.push(ChatThreadWithMembers {
                        thread,
                        member_count,
                    });
                }
                thread_list.sort_by(|a, b| b.thread.created_at_ms.cmp(&a.thread.created_at_ms));
                Ok(thread_list)
            })
        }

        fn create_member(&self, member: &ChatMember) -> BoxFuture<'_, DomainResult<ChatMember>> {
            let member = member.clone();
            let members = self.members.clone();
            Box::pin(async move {
                let key = (member.thread_id.clone(), member.user_id.clone());
                let mut members = members.write().await;
                if let Some(existing) = members.get(&key) {
                    if existing.left_at_ms.is_none() {
                        return Err(DomainError::Conflict);
                    }
                }
                members.insert(key, member.clone());
                Ok(member)
            })
        }

        fn list_members(&self, thread_id: &str) -> BoxFuture<'_, DomainResult<Vec<ChatMember>>> {
            let thread_id = thread_id.to_string();
            let members = self.members.clone();
            Box::pin(async move {
                let members: Vec<_> = members
                    .read()
                    .await
                    .values()
                    .filter(|member| member.thread_id == thread_id && member.left_at_ms.is_none())
                    .cloned()
                    .collect();
                Ok(members)
            })
        }

        fn get_member(
            &self,
            thread_id: &str,
            user_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ChatMember>>> {
            let key = (thread_id.to_string(), user_id.to_string());
            let members = self.members.clone();
            Box::pin(async move {
                let members = members.read().await;
                Ok(members.get(&key).cloned())
            })
        }

        fn create_message(
            &self,
            message: &ChatMessage,
        ) -> BoxFuture<'_, DomainResult<ChatMessage>> {
            let message = message.clone();
            let messages = self.messages.clone();
            let by_request = self.by_request.clone();
            Box::pin(async move {
                let key = (message.thread_id.clone(), message.request_id.clone());
                let mut by_request = by_request.write().await;
                if let Some(existing_id) = by_request.get(&key) {
                    let messages = messages.read().await;
                    if let Some(existing) =
                        messages.get(&(message.thread_id.clone(), existing_id.clone()))
                    {
                        return Ok(existing.clone());
                    }
                    by_request.remove(&key);
                }

                let mut messages = messages.write().await;
                let message_key = (message.thread_id.clone(), message.message_id.clone());
                if messages.contains_key(&message_key) {
                    return Err(DomainError::Conflict);
                }
                messages.insert(message_key, message.clone());
                by_request.insert(key, message.message_id.clone());
                Ok(message)
            })
        }

        fn get_message(
            &self,
            thread_id: &str,
            message_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
            let key = (thread_id.to_string(), message_id.to_string());
            let messages = self.messages.clone();
            Box::pin(async move {
                let messages = messages.read().await;
                Ok(messages.get(&key).cloned())
            })
        }

        fn get_message_by_request_id(
            &self,
            thread_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
            let thread_id = thread_id.to_string();
            let request_id = request_id.to_string();
            let by_request = self.by_request.clone();
            let messages = self.messages.clone();
            Box::pin(async move {
                let Some(message_id) = by_request
                    .read()
                    .await
                    .get(&(thread_id.clone(), request_id))
                    .cloned()
                else {
                    return Ok(None);
                };
                let messages = messages.read().await;
                Ok(messages.get(&(thread_id, message_id)).cloned())
            })
        }

        fn list_messages(
            &self,
            thread_id: &str,
            cursor: &MessageCatchup,
        ) -> BoxFuture<'_, DomainResult<Vec<ChatMessage>>> {
            let thread_id = thread_id.to_string();
            let cursor = cursor.clone();
            let messages = self.messages.clone();
            Box::pin(async move {
                let mut messages: Vec<_> = messages
                    .read()
                    .await
                    .values()
                    .filter(|message| message.thread_id == thread_id)
                    .cloned()
                    .collect();

                messages.sort_by(|a, b| {
                    a.created_at_ms
                        .cmp(&b.created_at_ms)
                        .then_with(|| a.message_id.cmp(&b.message_id))
                });

                let limit = cursor.limit;
                let after = message_cursor_filter(cursor)?;
                let messages = messages.into_iter().filter(after).take(limit).collect();
                Ok(messages)
            })
        }

        fn set_read_cursor(
            &self,
            cursor: &ChatReadCursor,
        ) -> BoxFuture<'_, DomainResult<ChatReadCursor>> {
            let cursor = cursor.clone();
            let cursors = self.cursors.clone();
            Box::pin(async move {
                let mut cursors = cursors.write().await;
                cursors.insert(
                    (cursor.thread_id.clone(), cursor.user_id.clone()),
                    cursor.clone(),
                );
                Ok(cursor)
            })
        }

        fn get_read_cursor(
            &self,
            thread_id: &str,
            user_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ChatReadCursor>>> {
            let key = (thread_id.to_string(), user_id.to_string());
            let cursors = self.cursors.clone();
            Box::pin(async move {
                let cursors = cursors.read().await;
                Ok(cursors.get(&key).cloned())
            })
        }

        fn create_delivery_event(
            &self,
            event: &ChatDeliveryEvent,
        ) -> BoxFuture<'_, DomainResult<ChatDeliveryEvent>> {
            let event = event.clone();
            let events = self.events.clone();
            Box::pin(async move {
                let mut events = events.write().await;
                let key = (event.thread_id.clone(), event.request_id.clone());
                if events.contains_key(&key) {
                    return Err(DomainError::Conflict);
                }
                events.insert(key, event.clone());
                Ok(event)
            })
        }

        fn get_delivery_event_by_request(
            &self,
            thread_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ChatDeliveryEvent>>> {
            let key = (thread_id.to_string(), request_id.to_string());
            let events = self.events.clone();
            Box::pin(async move {
                let events = events.read().await;
                Ok(events.get(&key).cloned())
            })
        }
    }

    fn message_cursor_filter(
        cursor: MessageCatchup,
    ) -> DomainResult<impl FnMut(&ChatMessage) -> bool> {
        if cursor.since_created_at_ms.is_none() && cursor.since_message_id.is_some() {
            return Err(DomainError::Validation(
                "since_message_id requires since_created_at_ms".into(),
            ));
        }
        Ok(
            move |message: &ChatMessage| match cursor.since_created_at_ms {
                None => true,
                Some(since_created_at_ms) => {
                    message.created_at_ms > since_created_at_ms
                        || message.created_at_ms == since_created_at_ms
                            && cursor
                                .since_message_id
                                .as_ref()
                                .is_none_or(|message_id| message.message_id > *message_id)
                }
            },
        )
    }

    #[tokio::test]
    async fn message_send_replay_uses_request_id() {
        let repo = Arc::new(MockChatRepo::default());
        let service = ChatService::new(repo);
        let actor = ActorIdentity {
            user_id: "u-1".to_string(),
            username: "alice".to_string(),
        };
        let thread = service
            .create_thread(
                &actor,
                "req-thread".to_string(),
                "corr-1".to_string(),
                ChatThreadCreate {
                    scope_id: "scope-1".to_string(),
                    privacy_level: "public".to_string(),
                },
            )
            .await
            .expect("thread");

        let payload = SendMessageInput {
            thread_id: thread.thread_id.clone(),
            body: "hello world".to_string(),
            attachments: vec![],
            request_id: "msg-1".to_string(),
            correlation_id: "corr-m".to_string(),
            occurred_at_ms: Some(1_000),
        };

        let first = service
            .send_message(&actor, payload.clone())
            .await
            .expect("first");
        let second = service.send_message(&actor, payload).await.expect("second");

        assert_eq!(first.message_id, second.message_id);
        assert_eq!(first.thread_id, second.thread_id);
    }

    #[tokio::test]
    async fn list_threads_includes_private_only_for_members() {
        let service = {
            let repo = Arc::new(MockChatRepo::default());
            let actor = ActorIdentity {
                user_id: "alice".to_string(),
                username: "alice".to_string(),
            };
            let service = ChatService::new(repo.clone());

            let public = service
                .create_thread(
                    &actor,
                    "req-public".to_string(),
                    "corr".to_string(),
                    ChatThreadCreate {
                        scope_id: "scope-1".to_string(),
                        privacy_level: "public".to_string(),
                    },
                )
                .await
                .expect("public");
            let private = service
                .create_thread(
                    &actor,
                    "req-private".to_string(),
                    "corr-2".to_string(),
                    ChatThreadCreate {
                        scope_id: "scope-1".to_string(),
                        privacy_level: "private".to_string(),
                    },
                )
                .await
                .expect("private");

            assert_eq!(public.privacy_level, "public");
            assert_eq!(private.privacy_level, "private");
            service
        };
        let actor = ActorIdentity {
            user_id: "alice".to_string(),
            username: "alice".to_string(),
        };
        let public_threads = service
            .list_threads_by_scope(&actor, "scope-1")
            .await
            .expect("threads");
        assert_eq!(public_threads.len(), 2);
    }

    #[test]
    fn message_validation_rejects_empty_body() {
        assert!(validate_message_input("", &[]).is_err());
        assert!(validate_message_input(&"x".repeat(2001), &[]).is_err());
    }

    #[test]
    fn thread_create_rejects_missing_scope() {
        let err = validate_thread_create_input(ChatThreadCreate {
            scope_id: "   ".to_string(),
            privacy_level: "public".to_string(),
        })
        .unwrap_err();
        assert!(matches!(err, DomainError::Validation(msg) if msg == "scope_id is required"));
    }

    #[test]
    fn cursor_filter_rejects_incomplete_args() {
        let result = message_cursor_filter(MessageCatchup {
            since_created_at_ms: None,
            since_message_id: Some("msg-1".to_string()),
            limit: 10,
        });
        assert!(result.is_err());
    }
}
