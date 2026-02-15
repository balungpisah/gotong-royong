use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Anonymous,
    User,
    Moderator,
    Admin,
    System,
}

impl Role {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "anonymous" | "guest" => Some(Role::Anonymous),
            "user" => Some(Role::User),
            "moderator" => Some(Role::Moderator),
            "admin" => Some(Role::Admin),
            "system" => Some(Role::System),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Anonymous => "anonymous",
            Role::User => "user",
            Role::Moderator => "moderator",
            Role::Admin => "admin",
            Role::System => "system",
        }
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, Role::Moderator | Role::Admin | Role::System)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin | Role::System)
    }
}
