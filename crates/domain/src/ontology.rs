use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    #[serde(rename = "schema:InformAction")]
    InformAction,
    #[serde(rename = "schema:RepairAction")]
    RepairAction,
    #[serde(rename = "schema:CreateAction")]
    CreateAction,
    #[serde(rename = "schema:SearchAction")]
    SearchAction,
    #[serde(rename = "schema:AchieveAction")]
    AchieveAction,
    #[serde(rename = "schema:AssessAction")]
    AssessAction,
    #[serde(rename = "schema:AlertAction")]
    AlertAction,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InformAction => "schema:InformAction",
            Self::RepairAction => "schema:RepairAction",
            Self::CreateAction => "schema:CreateAction",
            Self::SearchAction => "schema:SearchAction",
            Self::AchieveAction => "schema:AchieveAction",
            Self::AssessAction => "schema:AssessAction",
            Self::AlertAction => "schema:AlertAction",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OntologyEdgeKind {
    About,
    LocatedAt,
    HasAction,
    Broader,
    InstanceOf,
    Vouches,
    Challenges,
}

impl OntologyEdgeKind {
    pub fn as_table_name(&self) -> &'static str {
        match self {
            Self::About => "ABOUT",
            Self::LocatedAt => "LOCATED_AT",
            Self::HasAction => "HAS_ACTION",
            Self::Broader => "BROADER",
            Self::InstanceOf => "INSTANCE_OF",
            Self::Vouches => "VOUCHES",
            Self::Challenges => "CHALLENGES",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OntologyConcept {
    pub concept_id: String,
    pub qid: String,
    pub label_id: Option<String>,
    pub label_en: Option<String>,
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OntologyNote {
    pub note_id: String,
    pub content: String,
    pub author_id: String,
    pub community_id: String,
    pub temporal_class: String,
    pub ttl_expires_ms: Option<i64>,
    pub ai_readable: bool,
    pub rahasia_level: i64,
    pub confidence: f64,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OntologyNoteCreate {
    pub note_id: Option<String>,
    pub content: String,
    pub author_id: String,
    pub community_id: String,
    pub temporal_class: String,
    pub ttl_expires_ms: Option<i64>,
    pub ai_readable: bool,
    pub rahasia_level: i64,
    pub confidence: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OntologyTripleCreate {
    pub edge: OntologyEdgeKind,
    pub from_id: String,
    pub to_id: String,
    pub predicate: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteFeedbackCounts {
    pub vouch_count: usize,
    pub challenge_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_type_roundtrips_known_values() {
        let encoded = r#""schema:InformAction""#;
        let parsed: ActionType =
            serde_json::from_str(encoded).expect("parse known action type");
        assert_eq!(parsed, ActionType::InformAction);
        let encoded_back = serde_json::to_string(&parsed).expect("serialize action type");
        assert_eq!(encoded_back, encoded);
        assert_eq!(parsed.as_str(), "schema:InformAction");
    }

    #[test]
    fn action_type_rejects_unknown_schema_value() {
        let parsed = serde_json::from_str::<ActionType>(r#""schema:UnknownAction""#);
        assert!(parsed.is_err());
    }
}
