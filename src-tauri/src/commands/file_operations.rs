use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn read_file_content(
    file_path: String,
    max_bytes: Option<usize>,
) -> Result<String, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", file_path));
    }

    // Read the file with a size limit to prevent loading huge files
    let max_size = max_bytes.unwrap_or(1024 * 1024); // Default 1MB limit

    match fs::read(&path) {
        Ok(bytes) => {
            // Check if a file is too large
            if bytes.len() > max_size {
                return Err(format!(
                    "File is too large to preview ({} bytes). Maximum size is {} bytes.",
                    bytes.len(),
                    max_size
                ));
            }

            // Try to convert to UTF-8 string
            match String::from_utf8(bytes) {
                Ok(content) => Ok(content),
                Err(_) => Err(
                    "File contains invalid UTF-8 characters and cannot be displayed as text."
                        .to_string(),
                ),
            }
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}
