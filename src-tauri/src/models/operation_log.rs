use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationManifest {
    pub manifest_id: String,
    pub timestamp: i64,
    pub operation_type: String,
    pub completed: bool,
    pub rolled_back: bool,
    pub rollback_details: Option<RollbackResult>,
    pub operations: Vec<FileOperation>,
    pub total_files: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<OperationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub status: OperationStatus,
    pub backup_path: Option<PathBuf>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationError {
    pub file: PathBuf,
    pub error: String,
    pub error_code: ErrorCode,
    pub recovery_suggestion: Option<String>,
    pub is_retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PermissionDenied,
    FileLocked,
    DiskFull,
    PathNotFound,
    AlreadyExists,
    Unknown,
}

impl ErrorCode {
    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            ErrorCode::PermissionDenied => {
                "Try running the application as administrator or check file permissions."
            }
            ErrorCode::FileLocked => {
                "Close any programs that might be using this file and try again."
            }
            ErrorCode::DiskFull => "Free up disk space and retry the operation.",
            ErrorCode::PathNotFound => {
                "Ensure the file or folder exists and hasn't been moved or deleted."
            }
            ErrorCode::AlreadyExists => "Choose a different destination or enable overwrite mode.",
            ErrorCode::Unknown => "Check the error message for details and retry the operation.",
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            ErrorCode::FileLocked | ErrorCode::DiskFull | ErrorCode::Unknown => true,
            ErrorCode::PermissionDenied | ErrorCode::PathNotFound | ErrorCode::AlreadyExists => {
                false
            }
        }
    }
}

impl OperationError {
    pub fn new(file: PathBuf, error: String, error_code: ErrorCode) -> Self {
        let recovery_suggestion = Some(error_code.recovery_suggestion().to_string());
        let is_retryable = error_code.is_retryable();

        Self {
            file,
            error,
            error_code,
            recovery_suggestion,
            is_retryable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub manifest_id: String,
    pub restored_count: usize,
    pub failed_count: usize,
    pub errors: Vec<OperationError>,
}
