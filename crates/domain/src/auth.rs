use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Coarse authentication roles carried by API tokens.
///
/// These are intentionally kept stable for authN/Z paths and should not be
/// conflated with track/gov-specific actor roles in workflow logic.
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

/// Track-domain role context captured at command time for governance workflows.
///
/// Unlike [`Role`], these roles model membership/position within a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackRole {
    /// Original submitter / owner of the tracked item.
    Author,
    /// Person-in-charge responsible for execution and operational progress.
    Pic,
    /// A participant or collaborator on the track.
    Participant,
    /// Saksi/witness/reviewer role used in verification and voting phases.
    Saksi,
}

impl TrackRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Pic => "pic",
            Self::Participant => "participant",
            Self::Saksi => "saksi",
        }
    }

    pub fn supports(&self, action: &str) -> bool {
        match action {
            "propose" => matches!(self, Self::Author | Self::Pic),
            "object" => {
                matches!(
                    self,
                    Self::Author | Self::Pic | Self::Participant | Self::Saksi
                )
            }
            "vote" => {
                matches!(
                    self,
                    Self::Author | Self::Pic | Self::Participant | Self::Saksi
                )
            }
            _ => false,
        }
    }
}
