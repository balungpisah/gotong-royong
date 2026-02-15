use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorIdentity {
    pub user_id: String,
    pub username: String,
}

impl ActorIdentity {
    pub fn with_user_id(user_id: impl Into<String>) -> Self {
        let user_id = user_id.into();
        Self {
            user_id: user_id.clone(),
            username: user_id,
        }
    }
}
