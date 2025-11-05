use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub priority: i32,
    pub enabled: bool,
    pub conditions: Vec<RuleCondition>,
    pub condition_logic: ConditionLogic,
    pub action: RuleAction,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: ConditionField,
    pub operator: ConditionOperator,
    pub value: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionField {
    Filename,
    Extension,
    FileSize,
    Created,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    RegexMatch,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConditionLogic {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub destination: String,
    pub rename_pattern: Option<String>,
    pub create_folders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Move,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictResolution {
    Skip,
    Rename,
    Overwrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    pub file_path: PathBuf,
    pub destination_path: PathBuf,
    pub rule_id: String,
    pub rule_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule: Rule,
}
