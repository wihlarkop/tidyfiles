use super::file_info::FileInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanOptions {
    pub extensions: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<FileInfo>,
    pub total_count: usize,
    pub total_size: u64,
    pub scan_duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeOptions {
    #[serde(default = "default_operation_mode")]
    pub operation_mode: String, // "move" or "copy"
    #[serde(default = "default_true")]
    pub create_backup: bool,
    #[serde(default)]
    pub dry_run: bool,
}

fn default_operation_mode() -> String {
    "move".to_string()
}

fn default_true() -> bool {
    true
}
