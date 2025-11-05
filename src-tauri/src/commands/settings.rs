use crate::models::AppSettings;
use crate::storage::SettingsStore;
use tauri::AppHandle;

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = SettingsStore::new(&app)?;
    store.load()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let store = SettingsStore::new(&app)?;
    store.save(&settings)
}

#[tauri::command]
pub async fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = SettingsStore::new(&app)?;
    store.reset()
}

#[tauri::command]
pub async fn add_recent_folder(app: AppHandle, folder_path: String) -> Result<AppSettings, String> {
    let store = SettingsStore::new(&app)?;
    let mut settings = store.load()?;

    // Remove if already exists to avoid duplicates
    settings
        .general
        .recent_folders
        .retain(|f| f != &folder_path);

    // Add to front
    settings.general.recent_folders.insert(0, folder_path);

    // Keep only max_recent_folders
    let max = settings.general.max_recent_folders;
    if settings.general.recent_folders.len() > max {
        settings.general.recent_folders.truncate(max);
    }

    store.save(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn toggle_favorite_rule(app: AppHandle, rule_id: String) -> Result<AppSettings, String> {
    let store = SettingsStore::new(&app)?;
    let mut settings = store.load()?;

    if settings.general.favorite_rules.contains(&rule_id) {
        settings.general.favorite_rules.retain(|id| id != &rule_id);
    } else {
        settings.general.favorite_rules.push(rule_id);
    }

    store.save(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn set_last_used_rule(app: AppHandle, rule_id: String) -> Result<(), String> {
    let store = SettingsStore::new(&app)?;
    let mut settings = store.load()?;

    settings.general.last_used_rule_id = Some(rule_id);

    store.save(&settings)
}
