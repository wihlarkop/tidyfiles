use crate::models::{
    DuplicateFile, DuplicateGroup, DuplicateScanResult, FileOperation, OperationError,
    OperationManifest, OperationStatus,
};
use crate::storage::{ManifestStore, SettingsStore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Clone, Serialize)]
struct DuplicateScanProgress {
    current: usize,
    message: String,
}

#[tauri::command]
pub async fn scan_for_duplicates(
    app: AppHandle,
    directory: String,
    min_size: Option<u64>,
) -> Result<DuplicateScanResult, String> {
    let dir_path = PathBuf::from(&directory);

    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    // Map: file_size -> Vec <(path, hash)>
    // First group by size for performance (skip hashing files with unique sizes)
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut files_found = 0;

    // Collect files grouped by size
    for entry in WalkDir::new(&dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();

                // Skip files smaller than min_size
                if let Some(min) = min_size {
                    if size < min {
                        continue;
                    }
                }

                files_found += 1;

                // Emit progress every 100 files
                if files_found % 100 == 0 {
                    let _ = app.emit(
                        "duplicate-scan-progress",
                        DuplicateScanProgress {
                            current: files_found,
                            message: format!("Scanning... found {} files", files_found),
                        },
                    );
                }

                size_map
                    .entry(size)
                    .or_default()
                    .push(entry.path().to_path_buf());
            }
        }
    }

    // Load settings once before hashing
    let settings_store = SettingsStore::new(&app).ok();
    let settings = settings_store.as_ref().and_then(|s| s.load().ok());
    let use_partial_hash = settings
        .as_ref()
        .map(|s| s.duplicate_defaults.use_partial_hash)
        .unwrap_or(true);
    let partial_threshold = settings
        .as_ref()
        .map(|s| s.duplicate_defaults.partial_hash_threshold)
        .unwrap_or(10 * 1024 * 1024); // 10MB default

    // Now hash files that have the same size
    let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut hashed_count = 0;

    for (_size, paths) in size_map.iter() {
        // Only hash if there are multiple files with the same size
        if paths.len() > 1 {
            for path in paths {
                hashed_count += 1;

                // Emit progress every 50 hashes
                if hashed_count % 50 == 0 {
                    let _ = app.emit(
                        "duplicate-scan-progress",
                        DuplicateScanProgress {
                            current: hashed_count,
                            message: format!(
                                "Hashing duplicates... {}/{}",
                                hashed_count, files_found
                            ),
                        },
                    );
                }

                if let Ok(hash) = calculate_smart_hash(path, use_partial_hash, partial_threshold) {
                    hash_map.entry(hash).or_default().push(path.clone());
                }
            }
        }
    }

    // Build result
    let mut groups = Vec::new();
    let mut total_duplicates = 0;
    let mut wasted_space = 0u64;

    for (hash, paths) in hash_map.iter() {
        if paths.len() > 1 {
            let file_size = fs::metadata(&paths[0]).map(|m| m.len()).unwrap_or(0);

            let mut duplicate_files: Vec<DuplicateFile> = paths
                .iter()
                .map(|path| {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    let modified =
                        fs::metadata(path)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|t| {
                                t.duration_since(std::time::UNIX_EPOCH)
                                    .ok()
                                    .map(|d| d.as_secs() as i64)
                            });

                    DuplicateFile {
                        path: path.clone(),
                        name,
                        modified,
                        is_selected: false,
                    }
                })
                .collect();

            // Sort files by modification time (newest first)
            // If times are equal, sort alphabetically by path for consistency
            duplicate_files.sort_by(|a, b| match (a.modified, b.modified) {
                (Some(a_time), Some(b_time)) => {
                    b_time.cmp(&a_time).then_with(|| a.path.cmp(&b.path))
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.path.cmp(&b.path),
            });

            total_duplicates += duplicate_files.len() - 1; // -1 because we keep one
            wasted_space += file_size * (duplicate_files.len() as u64 - 1);

            groups.push(DuplicateGroup {
                hash: hash.clone(),
                file_size,
                files: duplicate_files,
            });
        }
    }

    // Sort groups by wasted space (largest first)
    groups.sort_by(|a, b| {
        let a_waste = a.file_size * (a.files.len() as u64 - 1);
        let b_waste = b.file_size * (b.files.len() as u64 - 1);
        b_waste.cmp(&a_waste)
    });

    Ok(DuplicateScanResult {
        total_duplicates,
        total_groups: groups.len(),
        wasted_space,
        groups,
    })
}

#[tauri::command]
pub async fn delete_duplicate_files(
    app: AppHandle,
    file_paths: Vec<String>,
) -> Result<OperationManifest, String> {
    let manifest_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    let mut operations = Vec::new();
    let mut errors = Vec::new();
    let mut success_count = 0;
    let mut error_count = 0;

    for path_str in file_paths {
        let path = PathBuf::from(&path_str);

        // Get file size before deletion
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        if path.exists() {
            match fs::remove_file(&path) {
                Ok(_) => {
                    operations.push(FileOperation {
                        source: path.clone(),
                        destination: path.clone(), // Same as a source for deletions
                        status: OperationStatus::Success,
                        backup_path: None,
                        file_size,
                    });
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(OperationError::new(
                        path.clone(),
                        e.to_string(),
                        crate::models::ErrorCode::Unknown,
                    ));
                    operations.push(FileOperation {
                        source: path.clone(),
                        destination: path.clone(),
                        status: OperationStatus::Failed,
                        backup_path: None,
                        file_size,
                    });
                    error_count += 1;
                }
            }
        } else {
            errors.push(OperationError::new(
                path.clone(),
                "File not found".to_string(),
                crate::models::ErrorCode::PathNotFound,
            ));
            operations.push(FileOperation {
                source: path.clone(),
                destination: path.clone(),
                status: OperationStatus::Failed,
                backup_path: None,
                file_size,
            });
            error_count += 1;
        }
    }

    let total_files = operations.len();

    let manifest = OperationManifest {
        manifest_id,
        timestamp,
        operation_type: "duplicate_deletion".to_string(),
        completed: true,
        rolled_back: false,
        rollback_details: None,
        operations,
        total_files,
        success_count,
        error_count,
        errors,
    };

    // Save manifest to disk for history
    let store = ManifestStore::new(&app)?;
    store.save(&manifest)?;

    Ok(manifest)
}

/// Smart hash function that uses partial hashing for large files
fn calculate_smart_hash(
    path: &PathBuf,
    use_partial: bool,
    threshold: u64,
) -> Result<String, std::io::Error> {
    let metadata = fs::metadata(path)?;
    let file_size = metadata.len();

    // If a file is smaller than a threshold or partial hashing is disabled, hash the entire file
    if !use_partial || file_size <= threshold {
        return calculate_full_hash(path);
    }

    // For large files, use partial hashing: first 64KB + middle 64KB + last 64KB
    calculate_partial_hash(path, file_size)
}

/// Calculate the full file hash
fn calculate_full_hash(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate partial hash for large files (first 64KB + middle 64KB + last 64KB)
fn calculate_partial_hash(path: &PathBuf, file_size: u64) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();

    const CHUNK_SIZE: usize = 64 * 1024; // 64KB
    let mut buffer = vec![0u8; CHUNK_SIZE];

    // Hash first 64KB
    let bytes_read = file.read(&mut buffer)?;
    hasher.update(&buffer[..bytes_read]);

    // Hash middle 64KB (if a file is large enough)
    if file_size > (CHUNK_SIZE * 2) as u64 {
        let middle_pos = (file_size / 2) - (CHUNK_SIZE as u64 / 2);
        file.seek(SeekFrom::Start(middle_pos))?;
        let bytes_read = file.read(&mut buffer)?;
        hasher.update(&buffer[..bytes_read]);
    }

    // Hash lasts 64KB (if the file is large enough)
    if file_size > CHUNK_SIZE as u64 {
        let last_pos = file_size.saturating_sub(CHUNK_SIZE as u64);
        file.seek(SeekFrom::Start(last_pos))?;
        let bytes_read = file.read(&mut buffer)?;
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// Keep the old function for backward compatibility
#[allow(dead_code)]
fn calculate_file_hash(path: &PathBuf) -> Result<String, std::io::Error> {
    calculate_full_hash(path)
}
