use super::file_info::FileInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub extensions: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            extensions: None,
            max_depth: None,
            include_hidden: false,
            follow_symlinks: false,
        }
    }
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
    pub operation_mode: String, // "move" or "copy"
    pub create_backup: bool,
    pub dry_run: bool,
}

impl Default for OrganizeOptions {
    fn default() -> Self {
        Self {
            operation_mode: "move".to_string(),
            create_backup: true,
            dry_run: false,
        }
    }
}
