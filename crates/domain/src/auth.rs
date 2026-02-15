use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    pub fn parse(value: &str) -> Option<Self> {
        Self::from_str(value).ok()
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

impl FromStr for Role {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "anonymous" | "guest" => Ok(Role::Anonymous),
            "user" => Ok(Role::User),
            "moderator" => Ok(Role::Moderator),
            "admin" => Ok(Role::Admin),
            "system" => Ok(Role::System),
            _ => Err("unknown role"),
        }
    }
}
