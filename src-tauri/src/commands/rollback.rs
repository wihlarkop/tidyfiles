use crate::models::{
    ErrorCode, OperationError, OperationManifest, OperationStatus, RollbackResult,
};
use crate::storage::ManifestStore;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_operation_history(app: AppHandle) -> Result<Vec<OperationManifest>, String> {
    let store = ManifestStore::new(&app)?;
    store.list_all()
}

#[tauri::command]
pub async fn rollback_operation(
    app: AppHandle,
    manifest_id: String,
) -> Result<RollbackResult, String> {
    let store = ManifestStore::new(&app)?;
    let manifest = store.load(&manifest_id)?;

    let mut restored_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    // Reverse operations in LIFO order (last operation first)
    for operation in manifest.operations.iter().rev() {
        // Only restore successful operations
        if !matches!(operation.status, OperationStatus::Success) {
            continue;
        }

        let result: Result<(), std::io::Error> = (|| {
            match manifest.operation_type.as_str() {
                "move" | "mixed" => {
                    // For move operations, move files back to the original location
                    if !operation.destination.exists() {
                        // File might have been deleted or moved elsewhere
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!(
                                "Destination file not found: {}",
                                operation.destination.display()
                            ),
                        ));
                    }

                    // Ensure source parent directory exists
                    if let Some(parent) = operation.source.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    // Try to rename it first (faster if on the same filesystem)
                    if fs::rename(&operation.destination, &operation.source).is_err() {
                        // Fallback to copy and delete
                        fs::copy(&operation.destination, &operation.source)?;
                        fs::remove_file(&operation.destination)?;
                    }
                    Ok(())
                }
                "copy" => {
                    // For copy operations, delete the copied file
                    if operation.destination.exists() {
                        fs::remove_file(&operation.destination)?;
                    }
                    // File already deleted, that's fine
                    Ok(())
                }
                _ => Ok(()),
            }
        })();

        match result {
            Ok(_) => restored_count += 1,
            Err(e) => {
                failed_count += 1;
                errors.push(OperationError::new(
                    operation.source.clone(),
                    e.to_string(),
                    ErrorCode::Unknown,
                ));
            }
        }
    }

    let rollback_result = RollbackResult {
        manifest_id: manifest_id.clone(),
        restored_count,
        failed_count,
        errors: errors.clone(),
    };

    // Mark manifest as rolled back and save rollback details
    if restored_count > 0 {
        let mut updated_manifest = manifest.clone();
        updated_manifest.rolled_back = true;
        updated_manifest.rollback_details = Some(rollback_result.clone());
        store.save(&updated_manifest)?;
    }

    Ok(rollback_result)
}

#[tauri::command]
pub async fn export_operation_log(
    app: AppHandle,
    manifest_id: String,
    export_path: String,
) -> Result<(), String> {
    let store = ManifestStore::new(&app)?;
    let manifest = store.load(&manifest_id)?;

    // Create CSV format
    let mut csv = String::from("Source,Destination,Status,Backup Path\n");

    for operation in &manifest.operations {
        let status = match operation.status {
            OperationStatus::Success => "Success",
            OperationStatus::Failed => "Failed",
            OperationStatus::Skipped => "Skipped",
        };

        let backup = operation
            .backup_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        csv.push_str(&format!(
            "\"{}\",\"{}\",{},{}\n",
            operation.source.display(),
            operation.destination.display(),
            status,
            backup
        ));
    }

    fs::write(&export_path, csv).map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_operation_log(app: AppHandle, manifest_id: String) -> Result<(), String> {
    let store = ManifestStore::new(&app)?;
    store.delete(&manifest_id)
}
