use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub file_size: u64,
    pub files: Vec<DuplicateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateFile {
    pub path: PathBuf,
    pub name: String,
    pub modified: Option<i64>,
    pub is_selected: bool, // For UI to track which file to keep
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateScanResult {
    pub total_duplicates: usize,
    pub total_groups: usize,
    pub wasted_space: u64,
    pub groups: Vec<DuplicateGroup>,
}
