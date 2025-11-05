use crate::models::OperationManifest;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct ManifestStore {
    storage_dir: PathBuf,
}

impl ManifestStore {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        let storage_dir = app_data_dir.join("manifests");

        // Create manifests directory if it doesn't exist
        fs::create_dir_all(&storage_dir)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        Ok(Self { storage_dir })
    }

    pub fn save(&self, manifest: &OperationManifest) -> Result<PathBuf, String> {
        let file_path = self
            .storage_dir
            .join(format!("{}.json", manifest.manifest_id));

        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        fs::write(&file_path, json).map_err(|e| format!("Failed to write manifest file: {}", e))?;

        Ok(file_path)
    }

    pub fn load(&self, manifest_id: &str) -> Result<OperationManifest, String> {
        let file_path = self.storage_dir.join(format!("{}.json", manifest_id));

        if !file_path.exists() {
            return Err(format!("Manifest {} not found", manifest_id));
        }

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;

        let manifest: OperationManifest = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize manifest: {}", e))?;

        Ok(manifest)
    }

    pub fn list_all(&self) -> Result<Vec<OperationManifest>, String> {
        let mut manifests = Vec::new();

        let entries = fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage directory: {}", e))?;

        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "json" {
                    if let Ok(manifest) =
                        self.load(entry.path().file_stem().unwrap().to_str().unwrap())
                    {
                        manifests.push(manifest);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        manifests.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(manifests)
    }

    pub fn delete(&self, manifest_id: &str) -> Result<(), String> {
        let file_path = self.storage_dir.join(format!("{}.json", manifest_id));

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| format!("Failed to delete manifest: {}", e))?;
        }

        Ok(())
    }
}
