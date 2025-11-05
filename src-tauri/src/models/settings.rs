use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default scan options
    pub scan_defaults: ScanDefaults,

    /// Default organize options
    pub organize_defaults: OrganizeDefaults,

    /// Duplicate finder defaults
    pub duplicate_defaults: DuplicateDefaults,

    /// UI preferences
    pub ui_preferences: UiPreferences,

    /// General settings
    pub general: GeneralSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDefaults {
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub max_depth: Option<u32>,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeDefaults {
    pub conflict_resolution: String, // "rename", "skip", "overwrite"
    pub create_folders: bool,
    pub auto_save_rules: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDefaults {
    pub min_file_size: u64,          // Minimum file size in bytes
    pub use_partial_hash: bool,      // Use partial hashing for large files
    pub partial_hash_threshold: u64, // File size threshold for partial hashing (bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String, // "light", "dark", "system"
    pub items_per_page: usize,
    pub show_file_preview: bool,
    pub confirm_deletions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub recent_folders: Vec<String>,
    pub max_recent_folders: usize,
    pub favorite_rules: Vec<String>, // Rule IDs
    pub last_used_rule_id: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_defaults: ScanDefaults {
                include_hidden: false,
                follow_symlinks: false,
                max_depth: None,
                extensions: None,
            },
            organize_defaults: OrganizeDefaults {
                conflict_resolution: "rename".to_string(),
                create_folders: true,
                auto_save_rules: false,
            },
            duplicate_defaults: DuplicateDefaults {
                min_file_size: 1024, // 1KB default
                use_partial_hash: true,
                partial_hash_threshold: 10 * 1024 * 1024, // 10MB
            },
            ui_preferences: UiPreferences {
                theme: "system".to_string(),
                items_per_page: 20,
                show_file_preview: true,
                confirm_deletions: true,
            },
            general: GeneralSettings {
                recent_folders: Vec::new(),
                max_recent_folders: 10,
                favorite_rules: Vec::new(),
                last_used_rule_id: None,
            },
        }
    }
}
