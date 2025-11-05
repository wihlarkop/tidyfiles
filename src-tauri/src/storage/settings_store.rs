use crate::models::AppSettings;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct SettingsStore {
    settings_file: PathBuf,
}

impl SettingsStore {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        // Create an app data directory if it doesn't exist
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;

        let settings_file = app_data_dir.join("settings.json");

        Ok(Self { settings_file })
    }

    pub fn load(&self) -> Result<AppSettings, String> {
        if !self.settings_file.exists() {
            // Return default settings if a file doesn't exist
            return Ok(AppSettings::default());
        }

        let json = fs::read_to_string(&self.settings_file)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: AppSettings = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize settings: {}", e))?;

        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.settings_file, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    pub fn reset(&self) -> Result<AppSettings, String> {
        let default_settings = AppSettings::default();
        self.save(&default_settings)?;
        Ok(default_settings)
    }
}
