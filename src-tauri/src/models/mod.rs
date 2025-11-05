pub mod duplicate;
pub mod file_info;
pub mod operation_log;
pub mod rule;
pub mod scan;
pub mod settings;

pub use duplicate::{DuplicateFile, DuplicateGroup, DuplicateScanResult};
pub use file_info::FileInfo;
pub use operation_log::{
    ErrorCode, FileOperation, OperationError, OperationManifest, OperationStatus, RollbackResult,
};
pub use rule::{
    ActionType, ConditionField, ConditionLogic, ConditionOperator, ConflictResolution, Rule,
    RuleCondition, RuleMatch,
};
pub use scan::{OrganizeOptions, ScanOptions, ScanResult};
pub use settings::{
    AppSettings, DuplicateDefaults, GeneralSettings, OrganizeDefaults, ScanDefaults, UiPreferences,
};
