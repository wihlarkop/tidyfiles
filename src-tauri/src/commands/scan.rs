use crate::models::{FileInfo, ScanOptions, ScanResult};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

#[derive(Clone, Serialize)]
struct ScanProgress {
    current: usize,
    message: String,
}

#[tauri::command]
pub async fn scan_directory(
    app: AppHandle,
    path: String,
    options: ScanOptions,
) -> Result<ScanResult, String> {
    let start_time = Instant::now();
    let root_path = PathBuf::from(&path);

    if !root_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !root_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut walker = WalkDir::new(&root_path).follow_links(options.follow_symlinks);

    if let Some(max_depth) = options.max_depth {
        walker = walker.max_depth(max_depth);
    }

    let mut files = Vec::new();
    let mut total_size: u64 = 0;
    let mut processed_count = 0;

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        processed_count += 1;

        // Emit progress every 50 files
        if processed_count % 50 == 0 {
            let _ = app.emit(
                "scan-progress",
                ScanProgress {
                    current: files.len(),
                    message: format!("Scanning... found {} files", files.len()),
                },
            );
        }

        let file_path = entry.path().to_path_buf();

        // Try to get file info, skip if there's an error
        let file_info = match FileInfo::from_path(file_path.clone()) {
            Ok(info) => info,
            Err(_) => continue, // Skip files we can't read
        };

        // Filter by hidden files
        if !options.include_hidden && file_info.is_hidden {
            continue;
        }

        // Filter by extensions if specified
        if let Some(ref exts) = options.extensions {
            if let Some(ref ext) = file_info.extension {
                let ext_lower = ext.to_lowercase();
                if !exts.iter().any(|e| e.to_lowercase() == ext_lower) {
                    continue;
                }
            } else {
                // No extension, skip if extensions filter is active
                continue;
            }
        }

        total_size += file_info.size;
        files.push(file_info);
    }

    let scan_duration = start_time.elapsed().as_millis();

    Ok(ScanResult {
        total_count: files.len(),
        total_size,
        files,
        scan_duration_ms: scan_duration,
    })
}

#[tauri::command]
pub async fn validate_paths(
    source: String,
    destinations: Vec<String>,
) -> Result<PathValidation, String> {
    let source_path = PathBuf::from(&source);
    let mut validation = PathValidation {
        source_valid: false,
        source_readable: false,
        destinations_valid: Vec::new(),
    };

    // Validate source
    if source_path.exists() {
        validation.source_valid = true;
        // Try to read the directory to check permissions
        if std::fs::read_dir(&source_path).is_ok() {
            validation.source_readable = true;
        }
    }

    // Validate destinations
    for dest in destinations {
        let dest_path = PathBuf::from(&dest);
        let dest_validation = DestinationValidation {
            path: dest.clone(),
            exists: dest_path.exists(),
            writable: check_writable(&dest_path),
            parent_exists: dest_path.parent().map(|p| p.exists()).unwrap_or(false),
        };
        validation.destinations_valid.push(dest_validation);
    }

    Ok(validation)
}

fn check_writable(path: &Path) -> bool {
    if path.exists() {
        // Try to create a temp file to test write permission
        let test_file = path.join(".tidyfiles_write_test");
        if std::fs::write(&test_file, b"test").is_ok() {
            let _ = std::fs::remove_file(&test_file);
            return true;
        }
        false
    } else {
        // Check if a parent is writable
        if let Some(parent) = path.parent() {
            if parent.exists() {
                return check_writable(parent);
            }
        }
        false
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PathValidation {
    pub source_valid: bool,
    pub source_readable: bool,
    pub destinations_valid: Vec<DestinationValidation>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DestinationValidation {
    pub path: String,
    pub exists: bool,
    pub writable: bool,
    pub parent_exists: bool,
}
