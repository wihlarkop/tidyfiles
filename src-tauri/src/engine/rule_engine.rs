use crate::models::{
    ConditionField, ConditionLogic, ConditionOperator, FileInfo, Rule, RuleCondition, RuleMatch,
};
use chrono::{DateTime, Datelike};
use regex::Regex;
use std::path::PathBuf;

pub struct RuleEngine;

impl RuleEngine {
    fn format_file_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.2}GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.2}MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.2}KB", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }

    pub fn evaluate_rule(rule: &Rule, file: &FileInfo) -> bool {
        if !rule.enabled {
            return false;
        }

        // If no conditions, match all files
        if rule.conditions.is_empty() {
            return true;
        }

        let results: Vec<bool> = rule
            .conditions
            .iter()
            .map(|condition| Self::evaluate_condition(condition, file))
            .collect();

        match rule.condition_logic {
            ConditionLogic::And => results.iter().all(|&r| r),
            ConditionLogic::Or => results.iter().any(|&r| r),
        }
    }

    fn evaluate_condition(condition: &RuleCondition, file: &FileInfo) -> bool {
        let field_value = match condition.field {
            ConditionField::Filename => file.name.clone(),
            ConditionField::Extension => file.extension.clone().unwrap_or_default(),
            ConditionField::FileSize => file.size.to_string(),
            ConditionField::Created => file.created.unwrap_or(0).to_string(),
            ConditionField::Modified => file.modified.unwrap_or(0).to_string(),
        };

        Self::apply_operator(
            &condition.operator,
            &field_value,
            &condition.value,
            condition.case_sensitive,
        )
    }

    fn apply_operator(
        operator: &ConditionOperator,
        field_value: &str,
        target_value: &str,
        case_sensitive: bool,
    ) -> bool {
        let field_val = if case_sensitive {
            field_value.to_string()
        } else {
            field_value.to_lowercase()
        };

        let target_val = if case_sensitive {
            target_value.to_string()
        } else {
            target_value.to_lowercase()
        };

        match operator {
            ConditionOperator::Equals => field_val == target_val,
            ConditionOperator::NotEquals => field_val != target_val,
            ConditionOperator::Contains => field_val.contains(&target_val),
            ConditionOperator::NotContains => !field_val.contains(&target_val),
            ConditionOperator::StartsWith => field_val.starts_with(&target_val),
            ConditionOperator::EndsWith => field_val.ends_with(&target_val),
            ConditionOperator::RegexMatch => {
                if let Ok(regex) = Regex::new(target_value) {
                    regex.is_match(field_value)
                } else {
                    false
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Ok(field_num), Ok(target_num)) =
                    (field_value.parse::<f64>(), target_value.parse::<f64>())
                {
                    field_num > target_num
                } else {
                    false
                }
            }
            ConditionOperator::LessThan => {
                if let (Ok(field_num), Ok(target_num)) =
                    (field_value.parse::<f64>(), target_value.parse::<f64>())
                {
                    field_num < target_num
                } else {
                    false
                }
            }
        }
    }

    pub fn build_destination_path(rule: &Rule, file: &FileInfo) -> Result<PathBuf, String> {
        let mut destination = rule.action.destination.clone();

        // Replace basic placeholders
        destination = destination.replace("{filename}", &file.filename_without_ext());
        destination = destination.replace("{ext}", &file.extension.clone().unwrap_or_default());
        destination = destination.replace("{name}", &file.name);

        // Parent folder placeholder
        if let Some(parent) = file.path.parent() {
            if let Some(parent_name) = parent.file_name() {
                if let Some(parent_str) = parent_name.to_str() {
                    destination = destination.replace("{parent}", parent_str);
                }
            }
        }

        // File size placeholder (human readable)
        let size_str = Self::format_file_size(file.size);
        destination = destination.replace("{size}", &size_str);

        // Modified date placeholders
        if let Some(modified) = file.modified {
            use chrono::{DateTime, Datelike, Utc};
            let dt: DateTime<Utc> =
                DateTime::from_timestamp(modified, 0).ok_or("Invalid timestamp")?;

            destination = destination.replace("{year}", &dt.year().to_string());
            destination = destination.replace("{month}", &format!("{:02}", dt.month()));
            destination = destination.replace("{day}", &format!("{:02}", dt.day()));
            destination = destination.replace("{modified_year}", &dt.year().to_string());
            destination = destination.replace("{modified_month}", &format!("{:02}", dt.month()));
            destination = destination.replace("{modified_day}", &format!("{:02}", dt.day()));
            destination = destination.replace("{modified_date}", &format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()));
        }

        // Created date placeholders
        if let Some(created) = file.created {
            use chrono::{DateTime, Datelike, Utc};
            let dt: DateTime<Utc> =
                DateTime::from_timestamp(created, 0).ok_or("Invalid timestamp")?;

            destination = destination.replace("{created_year}", &dt.year().to_string());
            destination = destination.replace("{created_month}", &format!("{:02}", dt.month()));
            destination = destination.replace("{created_day}", &format!("{:02}", dt.day()));
            destination = destination.replace("{created_date}", &format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()));
        }

        let dest_path = PathBuf::from(destination);

        // Apply a rename pattern if specified
        if let Some(rename_pattern) = &rule.action.rename_pattern {
            let mut new_name = rename_pattern
                .replace("{filename}", &file.filename_without_ext())
                .replace("{ext}", &file.extension.clone().unwrap_or_default())
                .replace("{name}", &file.name);

            // Parent folder placeholder
            if let Some(parent) = file.path.parent() {
                if let Some(parent_name) = parent.file_name() {
                    if let Some(parent_str) = parent_name.to_str() {
                        new_name = new_name.replace("{parent}", parent_str);
                    }
                }
            }

            // File size placeholder
            let size_str = Self::format_file_size(file.size);
            new_name = new_name.replace("{size}", &size_str);

            // Modified date placeholders
            if let Some(modified) = file.modified {
                if let Some(dt) = DateTime::from_timestamp(modified, 0) {
                    new_name = new_name.replace("{year}", &dt.year().to_string());
                    new_name = new_name.replace("{month}", &format!("{:02}", dt.month()));
                    new_name = new_name.replace("{day}", &format!("{:02}", dt.day()));
                    new_name = new_name.replace("{modified_year}", &dt.year().to_string());
                    new_name = new_name.replace("{modified_month}", &format!("{:02}", dt.month()));
                    new_name = new_name.replace("{modified_day}", &format!("{:02}", dt.day()));
                    new_name = new_name.replace("{modified_date}", &format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()));
                }
            }

            // Created date placeholders
            if let Some(created) = file.created {
                if let Some(dt) = DateTime::from_timestamp(created, 0) {
                    new_name = new_name.replace("{created_year}", &dt.year().to_string());
                    new_name = new_name.replace("{created_month}", &format!("{:02}", dt.month()));
                    new_name = new_name.replace("{created_day}", &format!("{:02}", dt.day()));
                    new_name = new_name.replace("{created_date}", &format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()));
                }
            }

            // Handle counter-placeholder - Note: counter will need to be passed as context
            // For now, we'll keep the placeholder as-is, actual counter-logic should be in organize command
            // The counter should increment per batch of files being processed

            Ok(dest_path.join(new_name))
        } else {
            Ok(dest_path.join(&file.name))
        }
    }

    pub fn match_files(rules: &[Rule], files: &[FileInfo]) -> Vec<RuleMatch> {
        let mut matches = Vec::new();

        // Sort rules by priority (ascending)
        let mut sorted_rules = rules.to_vec();
        sorted_rules.sort_by_key(|r| r.priority);

        for file in files {
            // Find the first matching rule
            if let Some(rule) = sorted_rules.iter().find(|r| Self::evaluate_rule(r, file)) {
                if let Ok(dest_path) = Self::build_destination_path(rule, file) {
                    matches.push(RuleMatch {
                        file_path: file.path.clone(),
                        destination_path: dest_path,
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                    });
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_condition_contains() {
        let file = FileInfo {
            path: PathBuf::from("/test/invoice_2024.pdf"),
            name: "invoice_2024.pdf".to_string(),
            extension: Some("pdf".to_string()),
            size: 1024,
            created: None,
            modified: None,
            is_hidden: false,
        };

        let condition = RuleCondition {
            field: ConditionField::Filename,
            operator: ConditionOperator::Contains,
            value: "invoice".to_string(),
            case_sensitive: false,
        };

        assert!(RuleEngine::apply_operator(
            &condition.operator,
            &file.name,
            &condition.value,
            condition.case_sensitive
        ));
    }
}
