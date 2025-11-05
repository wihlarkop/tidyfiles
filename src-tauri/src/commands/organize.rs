use crate::engine::RuleEngine;
use crate::models::{
    ConflictResolution, ErrorCode, FileInfo, FileOperation, OperationError, OperationManifest,
    OperationStatus, OrganizeOptions, Rule, RuleMatch,
};
use crate::storage::ManifestStore;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

#[derive(Clone, Serialize)]
struct OrganizeProgress {
    current: usize,
    total: usize,
    message: String,
}

#[tauri::command]
pub async fn preview_rule_matches(
    rules: Vec<Rule>,
    files: Vec<FileInfo>,
) -> Result<Vec<RuleMatch>, String> {
    Ok(RuleEngine::match_files(&rules, &files))
}

#[tauri::command]
pub async fn organize_files(
    app: AppHandle,
    files: Vec<FileInfo>,
    rules: Vec<Rule>,
    options: OrganizeOptions,
) -> Result<OperationManifest, String> {
    let manifest_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    // Get rule matches
    let matches = RuleEngine::match_files(&rules, &files);

    let mut operations = Vec::new();
    let mut errors = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    let total_files = matches.len();

    // Track counter per rule for pattern-based renaming
    let mut rule_counters: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    // Create a backup directory if backup mode is enabled
    let backup_dir = if options.create_backup {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        let backup_path = app_data_dir.join("backups").join(&manifest_id);

        fs::create_dir_all(&backup_path)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        Some(backup_path)
    } else {
        None
    };

    for (index, rule_match) in matches.iter().enumerate() {
        // Emit progress every 10 files
        if index % 10 == 0 {
            let _ = app.emit(
                "organize-progress",
                OrganizeProgress {
                    current: index,
                    total: total_files,
                    message: format!("Organizing {}/{} files...", index, total_files),
                },
            );
        }

        let source = &rule_match.file_path;
        let mut destination = rule_match.destination_path.clone();

        // Handle counter-placeholders in the destination path
        if let Some(dest_str) = destination.to_str() {
            if dest_str.contains("{counter") {
                // Increment counter for this rule
                let counter = rule_counters.entry(rule_match.rule_id.clone()).or_insert(0);
                *counter += 1;

                // Replace counter-patterns like {counter:03d} or {counter:05d}
                let counter_regex = regex::Regex::new(r"\{counter:(\d+)d}").unwrap();
                let dest_str_updated =
                    counter_regex.replace_all(dest_str, |caps: &regex::Captures| {
                        let width: usize = caps[1].parse().unwrap_or(3);
                        format!("{:0width$}", counter, width = width)
                    });

                destination = PathBuf::from(dest_str_updated.to_string());
            }
        }

        // Get file size
        let file_size = fs::metadata(source).map(|m| m.len()).unwrap_or(0);

        // Find the rule to get conflict resolution strategy
        let rule = rules
            .iter()
            .find(|r| r.id == rule_match.rule_id)
            .ok_or("Rule not found")?;

        // Handle conflict resolution
        if destination.exists() {
            match rule.conflict_resolution {
                ConflictResolution::Skip => {
                    operations.push(FileOperation {
                        source: source.clone(),
                        destination: destination.clone(),
                        status: OperationStatus::Skipped,
                        backup_path: None,
                        file_size,
                    });
                    continue;
                }
                ConflictResolution::Rename => {
                    destination = generate_unique_path(&destination);
                }
                ConflictResolution::Overwrite => {
                    // Will overwrite, handled by copy/move operation
                }
            }
        }

        // Dry run mode - don't perform operations
        if options.dry_run {
            operations.push(FileOperation {
                source: source.clone(),
                destination: destination.clone(),
                status: OperationStatus::Success,
                backup_path: None,
                file_size,
            });
            success_count += 1;
            continue;
        }

        // Create a destination directory if needed
        if rule.action.create_folders {
            if let Some(parent) = destination.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        errors.push(OperationError::new(
                            source.clone(),
                            format!("Failed to create directory: {}", e),
                            ErrorCode::PermissionDenied,
                        ));
                        error_count += 1;
                        continue;
                    }
                }
            }
        }

        // Create a backup if enabled
        let backup_path = if let Some(ref backup_dir) = backup_dir {
            let file_name = source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            let backup_file_path: PathBuf = backup_dir.join(file_name);

            // Copy to back-up location
            match fs::copy(source, &backup_file_path) {
                Ok(_) => Some(backup_file_path),
                Err(e) => {
                    errors.push(OperationError::new(
                        source.clone(),
                        format!("Failed to create backup: {}", e),
                        ErrorCode::Unknown,
                    ));
                    error_count += 1;
                    continue;
                }
            }
        } else {
            None
        };

        // Perform the operation based on the rule's action type
        let result = match rule.action.action_type {
            crate::models::ActionType::Copy => perform_copy(source, &destination),
            crate::models::ActionType::Move => perform_move(source, &destination),
        };

        match result {
            Ok(_) => {
                operations.push(FileOperation {
                    source: source.clone(),
                    destination: destination.clone(),
                    status: OperationStatus::Success,
                    backup_path,
                    file_size,
                });
                success_count += 1;
            }
            Err(e) => {
                let error_code = classify_error(&e);
                errors.push(OperationError::new(
                    source.clone(),
                    e.to_string(),
                    error_code,
                ));
                operations.push(FileOperation {
                    source: source.clone(),
                    destination: destination.clone(),
                    status: OperationStatus::Failed,
                    backup_path: None,
                    file_size,
                });
                error_count += 1;
            }
        }
    }

    let manifest = OperationManifest {
        manifest_id,
        timestamp,
        operation_type: "mixed".to_string(), // Can be moved, copy, or mixed
        completed: true,
        rolled_back: false,
        rollback_details: None,
        operations,
        total_files,
        success_count,
        error_count,
        errors,
    };

    // Save manifest to disk for history/undo
    let store = ManifestStore::new(&app)?;
    store.save(&manifest)?;

    Ok(manifest)
}

fn perform_copy(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    fs::copy(source, destination)?;
    Ok(())
}

fn perform_move(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    // Try to rename it first (faster if on the same filesystem)
    if fs::rename(source, destination).is_ok() {
        return Ok(());
    }

    // Fallback to copy and delete
    fs::copy(source, destination)?;
    fs::remove_file(source)?;
    Ok(())
}

fn generate_unique_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or(Path::new(""));
    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut counter = 1;
    loop {
        let new_name = if extension.is_empty() {
            format!("{}({})", filename, counter)
        } else {
            format!("{}({}).{}", filename, counter, extension)
        };

        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn classify_error(error: &std::io::Error) -> ErrorCode {
    use std::io::ErrorKind;

    match error.kind() {
        ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        ErrorKind::NotFound => ErrorCode::PathNotFound,
        ErrorKind::AlreadyExists => ErrorCode::AlreadyExists,
        _ => {
            let error_str = error.to_string().to_lowercase();
            if error_str.contains("lock") || error_str.contains("in use") {
                ErrorCode::FileLocked
            } else if error_str.contains("space") || error_str.contains("disk full") {
                ErrorCode::DiskFull
            } else {
                ErrorCode::Unknown
            }
        }
    }
}
