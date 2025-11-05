pub mod duplicates;
pub mod file_operations;
pub mod organize;
pub mod rollback;
pub mod scan;
pub mod settings;

pub use duplicates::{delete_duplicate_files, scan_for_duplicates};
pub use file_operations::read_file_content;
pub use organize::{organize_files, preview_rule_matches};
pub use rollback::{
    delete_operation_log, export_operation_log, get_operation_history, rollback_operation,
};
pub use scan::{scan_directory, validate_paths};
pub use settings::{
    add_recent_folder, load_settings, reset_settings, save_settings, set_last_used_rule,
    toggle_favorite_rule,
};
