use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub created: Option<i64>,
    pub modified: Option<i64>,
    pub is_hidden: bool,
}

impl FileInfo {
    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        let created = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        #[cfg(unix)]
        let is_hidden = name.starts_with('.');

        #[cfg(windows)]
        let is_hidden = {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
        };

        #[cfg(not(any(unix, windows)))]
        let is_hidden = false;

        Ok(FileInfo {
            path,
            name,
            extension,
            size: metadata.len(),
            created,
            modified,
            is_hidden,
        })
    }

    pub fn filename_without_ext(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    }
}
