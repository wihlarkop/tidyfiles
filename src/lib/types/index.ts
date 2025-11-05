// File Information
export interface FileInfo {
    path: string;
    name: string;
    extension: string | null;
    size: number;
    created: number | null;
    modified: number | null;
    is_hidden: boolean;
}

// Scan Options
export interface ScanOptions {
    extensions: string[] | null;
    max_depth: number | null;
    include_hidden: boolean;
    follow_symlinks: boolean;
}

// Scan Result
export interface ScanResult {
    files: FileInfo[];
    total_count: number;
    total_size: number;
    scan_duration_ms: number;
}

// Rule Condition
export interface RuleCondition {
    field: ConditionField;
    operator: ConditionOperator;
    value: string;
    case_sensitive: boolean;
}

export type ConditionField = 'filename' | 'extension' | 'filesize' | 'created' | 'modified';

export type ConditionOperator =
    | 'equals'
    | 'not_equals'
    | 'contains'
    | 'not_contains'
    | 'starts_with'
    | 'ends_with'
    | 'regex_match'
    | 'greater_than'
    | 'less_than';

export type ConditionLogic = 'AND' | 'OR';

// Rule Action
export interface RuleAction {
    type: ActionType;
    destination: string;
    rename_pattern: string | null;
    create_folders: boolean;
}

export type ActionType = 'move' | 'copy';

export type ConflictResolution = 'skip' | 'rename' | 'overwrite';

// Rule
export interface Rule {
    id: string;
    name: string;
    priority: number;
    enabled: boolean;
    conditions: RuleCondition[];
    condition_logic: ConditionLogic;
    action: RuleAction;
    conflict_resolution: ConflictResolution;
}

// Rule Match
export interface RuleMatch {
    file_path: string;
    destination_path: string;
    rule_id: string;
    rule_name: string;
}

// Organize Options
export interface OrganizeOptions {
    operation_mode: string;
    create_backup: boolean;
    dry_run: boolean;
}

// Operation Manifest
export interface OperationManifest {
    manifest_id: string;
    timestamp: number;
    operation_type: string;
    completed: boolean;
    rolled_back: boolean;
    rollback_details: RollbackResult | null;
    operations: FileOperation[];
    total_files: number;
    success_count: number;
    error_count: number;
    errors: OperationError[];
}

export interface FileOperation {
    source: string;
    destination: string;
    status: OperationStatus;
    backup_path: string | null;
    file_size: number;
}

export type OperationStatus = 'success' | 'failed' | 'skipped';

export interface OperationError {
    file: string;
    error: string;
    error_code: ErrorCode;
    recovery_suggestion?: string;
    is_retryable: boolean;
}

export type ErrorCode =
    | 'permission_denied'
    | 'file_locked'
    | 'disk_full'
    | 'path_not_found'
    | 'already_exists'
    | 'unknown';

// Rollback Result
export interface RollbackResult {
    manifest_id: string;
    restored_count: number;
    failed_count: number;
    errors: OperationError[];
}

// Duplicate Detection
export interface DuplicateGroup {
    hash: string;
    file_size: number;
    files: DuplicateFile[];
}

export interface DuplicateFile {
    path: string;
    name: string;
    modified: number | null;
    is_selected: boolean;
}

export interface DuplicateScanResult {
    total_duplicates: number;
    total_groups: number;
    wasted_space: number;
    groups: DuplicateGroup[];
}

// Settings
export interface AppSettings {
    scan_defaults: ScanDefaults;
    organize_defaults: OrganizeDefaults;
    duplicate_defaults: DuplicateDefaults;
    ui_preferences: UiPreferences;
    general: GeneralSettings;
}

export interface ScanDefaults {
    include_hidden: boolean;
    follow_symlinks: boolean;
    max_depth: number | null;
    extensions: string[] | null;
}

export interface OrganizeDefaults {
    conflict_resolution: string;
    create_folders: boolean;
    auto_save_rules: boolean;
}

export interface DuplicateDefaults {
    min_file_size: number;
    use_partial_hash: boolean;
    partial_hash_threshold: number;
}

export interface UiPreferences {
    theme: 'light' | 'dark' | 'system';
    items_per_page: number;
    show_file_preview: boolean;
    confirm_deletions: boolean;
}

export interface GeneralSettings {
    recent_folders: string[];
    max_recent_folders: number;
    favorite_rules: string[];
    last_used_rule_id: string | null;
}
