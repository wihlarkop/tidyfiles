mod commands;
mod engine;
mod models;
mod storage;

use commands::{
    add_recent_folder, delete_duplicate_files, delete_operation_log, export_operation_log,
    get_operation_history, load_settings, organize_files, preview_rule_matches, read_file_content,
    reset_settings, rollback_operation, save_settings, scan_directory, scan_for_duplicates,
    set_last_used_rule, toggle_favorite_rule, validate_paths,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            validate_paths,
            preview_rule_matches,
            organize_files,
            get_operation_history,
            rollback_operation,
            export_operation_log,
            delete_operation_log,
            scan_for_duplicates,
            delete_duplicate_files,
            read_file_content,
            load_settings,
            save_settings,
            reset_settings,
            add_recent_folder,
            toggle_favorite_rule,
            set_last_used_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
