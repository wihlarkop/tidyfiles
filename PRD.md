# 🗂️ Product Requirements Document (PRD)

## Project: TidyFiles

**Type:** Desktop Application
**Tech Stack:** Tauri (Rust Backend) + Svelte (Frontend)
**Status:** Concept / MVP Planning
**Author:** Wihlarko Prasdegdho

---

## 1. Overview

**TidyFiles** is a cross-platform desktop application that automatically organizes user documents (PDF, Word, Excel,
etc.) into structured folders based on predefined or AI-generated rules. The goal is to reduce manual file management
and increase productivity for individuals who frequently handle digital documents.

---

## 2. Objectives

- Enable users to quickly scan, preview, and organize large sets of files.
- Allow custom rule-based sorting (by filename patterns, metadata, or file content).
- Provide a clean, responsive, and native-like UI for desktop environments.
- Support intelligent renaming and classification using optional AI integrations.

---

## 3. Key Features

### 3.1 File Scanning

- Recursive folder scanning with filtering by extension or file type.
- Metadata extraction: file size, extension, creation/modification date.
- Display file summary table in the frontend UI.

### 3.2 Rule-Based Organization

- Users can define rules such as:
    - `If filename contains "invoice" → move to /Invoices`
    - `If extension is ".docx" → move to /Contracts`
- Rules are applied sequentially and can be saved for reuse.
- Support for “copy” or “move” operations.

### 3.3 Real-Time Progress and Logging

- Show progress bar during scanning and file movement.
- Emit live updates from Rust backend to Svelte UI via Tauri events.
- Maintain an activity log with timestamps, results, and error details.

### 3.4 Rule Templates

- Users can save and load rule sets (JSON-based).
- Default templates included (Invoices, Reports, Legal, Personal).

### 3.5 Persistent Preferences

- Store user settings such as:
    - Last used directory
    - Default destination path
    - Move/Copy preference
    - UI theme (light/dark)

### 3.6 (Optional) AI Enhancements

- **AI Rename:** Suggest or automatically rename files using a local or API-based LLM.
- **AI Classification:** Categorize documents by semantic content (e.g., detect “invoice”, “resume”, “report”).
- Local inference integration via `llama.cpp` or external API (e.g., OpenAI, Ollama).

---

## 4. Advanced Features (Future Iterations)

| Feature                    | Description                                                | Priority |
|----------------------------|------------------------------------------------------------|----------|
| **PDF Preview**            | Display first page or thumbnail preview of PDF documents.  | Medium   |
| **Undo / Rollback**        | Allow users to revert the last organization session.       | High     |
| **Auto-Watch Mode**        | Continuously monitor a folder and auto-organize new files. | Medium   |
| **OCR Integration**        | Extract text from scanned documents (via Tesseract).       | Low      |
| **Multi-Language Support** | Localize UI and AI classification labels.                  | Low      |

---

## 5. User Flow

### 5.1 Onboarding

1. User launches the app.
2. Sees a drag-and-drop area or “Select Folder” button.

### 5.2 Scanning Phase

1. User selects a folder.
2. App scans and lists files with metadata.
3. User reviews and optionally filters files.

### 5.3 Rule Setup Phase

1. User creates sorting rules using a rule builder UI.
2. Can test or preview which files will be affected by each rule.

### 5.4 Organizing Phase

1. User clicks “Organize.”
2. Rust backend executes file operations (move/copy).
3. Progress and logs update in real time.
4. Upon completion, user sees a summary and can export results.

---

## 6. Technical Design

### 6.1 Frontend (Svelte)

- **Framework:** Svelte + TailwindCSS
- **Responsibilities:**
    - File list rendering and filtering
    - Rule builder UI
    - Progress display and event handling
    - Persist user preferences via `tauri-plugin-store`
- **Communication:**
    - Uses `invoke()` to call Rust commands
    - Uses `listen()` to receive progress and logs

### 6.2 Backend (Rust)

- **Responsibilities:**
    - File system scanning (via `walkdir`)
    - Metadata extraction (`std::fs::Metadata`)
    - File operations (move/copy/delete)
    - Event emission (`emit_all`) for progress and logs
    - Rule engine: parse and apply JSON-defined rules
- **Structure:**

```
src-tauri/
  ├── src/
  │ ├── main.rs
  │ ├── commands/
  │ │ ├── scan.rs
  │ │ ├── organize.rs
  │ │ └── utils.rs
  │ ├── models/
  │ │ ├── file_info.rs
  │ │ ├── rule.rs
  │ │ └── operation_log.rs
  │ ├── engine/
  │ │ ├── rule_engine.rs
  │ │ └── conflict_resolver.rs
  │ ├── storage/
  │ │ └── store.rs
  │ └── events.rs
  └── tauri.conf.json
```

### 6.3 Data Storage

- **Rule Storage:** SQLite database (`tauri-plugin-sql`) for storing user-defined rules, templates, and preferences
- **Operation Logs:** JSON-based transaction logs stored locally for rollback/undo functionality
- **User Preferences:** Key-value store via `tauri-plugin-store`
- **Cache:** In-memory caching for frequently accessed rules and file metadata during operations

### 6.4 Tauri Command API Reference

#### Scanning Commands

```rust
#[tauri::command]
async fn scan_directory(path: String, options: ScanOptions) -> Result<ScanResult, String>
```
- **Description:** Recursively scan directory and return file metadata
- **Parameters:**
  - `path`: Absolute path to directory
  - `options`: Filter options (extensions, max_depth, include_hidden)
- **Returns:** List of FileInfo objects with metadata
- **Events Emitted:** `scan-progress` (progress updates), `scan-complete`

```rust
#[tauri::command]
async fn validate_paths(source: String, destinations: Vec<String>) -> Result<PathValidation, String>
```
- **Description:** Validate that paths exist and app has necessary permissions
- **Returns:** Validation result with permission details

#### Rule Management Commands

```rust
#[tauri::command]
async fn create_rule(rule: Rule) -> Result<i64, String>
```
- **Description:** Create and persist a new organization rule
- **Returns:** Rule ID

```rust
#[tauri::command]
async fn preview_rule_matches(rule: Rule, files: Vec<FileInfo>) -> Result<Vec<RuleMatch>, String>
```
- **Description:** Dry-run to show which files would be affected by a rule
- **Returns:** List of files with destination paths (no actual file operations)

```rust
#[tauri::command]
async fn get_rule_templates() -> Result<Vec<RuleTemplate>, String>
```
- **Description:** Retrieve built-in and user-saved rule templates

#### Organization Commands

```rust
#[tauri::command]
async fn organize_files(files: Vec<FileInfo>, rules: Vec<Rule>, options: OrganizeOptions) -> Result<OperationManifest, String>
```
- **Description:** Execute file organization based on rules
- **Parameters:**
  - `files`: Files to organize
  - `rules`: Rules to apply (in priority order)
  - `options`: Operation mode (move/copy), conflict resolution strategy, create_backup
- **Returns:** Manifest with operation results and transaction log ID
- **Events Emitted:** `organize-progress`, `organize-complete`, `organize-error`

```rust
#[tauri::command]
async fn rollback_operation(manifest_id: String) -> Result<RollbackResult, String>
```
- **Description:** Revert a previous organization operation
- **Returns:** List of files restored and any errors

### 6.5 Event System

**Event Emission Strategy:**
- Batch updates every 100ms or every 50 files (whichever comes first) to prevent UI overload
- Throttle events when processing large file sets (>5000 files)
- Use dedicated event channels for progress vs. errors

**Event Types:**

```typescript
// Progress events
interface ScanProgressEvent {
  scanned: number;
  total_estimated: number;
  current_path: string;
}

interface OrganizeProgressEvent {
  processed: number;
  total: number;
  current_file: string;
  operation: 'move' | 'copy';
  success_count: number;
  error_count: number;
}

// Error events
interface OperationErrorEvent {
  file_path: string;
  error_type: 'permission_denied' | 'file_locked' | 'disk_full' | 'path_not_found';
  error_message: string;
  recoverable: boolean;
}
```

### 6.6 Rule Engine Specification

**Rule Structure:**
```json
{
  "id": "uuid-v4",
  "name": "Organize Invoices",
  "priority": 1,
  "enabled": true,
  "conditions": [
    {
      "field": "filename",
      "operator": "contains",
      "value": "invoice",
      "case_sensitive": false
    },
    {
      "field": "extension",
      "operator": "equals",
      "value": ".pdf"
    }
  ],
  "condition_logic": "AND",
  "action": {
    "type": "move",
    "destination": "/Invoices/{year}/{month}",
    "rename_pattern": null,
    "create_folders": true
  },
  "conflict_resolution": "rename"
}
```

**Supported Operators:**
- `equals`, `not_equals`
- `contains`, `not_contains`
- `starts_with`, `ends_with`
- `regex_match`
- `greater_than`, `less_than` (for file size, dates)

**Conflict Resolution Strategies:**
- `skip`: Skip the file if destination exists
- `rename`: Append number suffix (e.g., `file(1).pdf`)
- `overwrite`: Replace existing file (requires confirmation)
- `merge_folder`: Merge contents if destination is a folder

**Rule Priority:**
- Rules are applied in ascending priority order (1 = highest)
- First matching rule wins (unless `allow_multiple_matches` is enabled)
- Rules can be organized into rule sets for batch application

---

## 7. Non-Functional Requirements

| Category            | Requirement                                                                        |
|---------------------|------------------------------------------------------------------------------------|
| **Performance**     | Should handle at least 10,000 files per scan within reasonable time (<10s on SSD). |
| **Security**        | App should never delete or overwrite files without explicit user confirmation.     |
| **Usability**       | Minimal learning curve; all key actions within 2 clicks.                           |
| **Portability**     | Works on Windows, macOS, and Linux.                                                |
| **Reliability**     | File operations must be atomic; rollback supported on failure.                     |
| **Maintainability** | Rust and Svelte code modularized by function; each module testable independently.  |

---

## 8. Success Metrics

- 🧩 **MVP Goal:** Successfully organize a folder of 100+ files using 2–3 simple user rules.
- ⚡ **Performance Goal:** Scan <10s for 10,000 files.
- 💬 **User Feedback Goal:** Average usability rating ≥ 4.5/5 in pilot testing.
- 🧠 **AI Goal (Optional):** 80% accuracy in auto classification of document types.

---

## 9. Risks and Mitigation

| Risk                          | Description                                         | Mitigation                                                   |
|-------------------------------|-----------------------------------------------------|--------------------------------------------------------------|
| **File Loss**                 | Incorrect rule deletes or overwrites a file.        | Implement backup mode, transaction logs, and rollback.       |
| **Performance Bottlenecks**   | Scanning large directories may be slow.             | Use async Rust + parallel iteration with `rayon`.            |
| **Complex Rules**             | User confusion creating rules.                      | Provide templates, rule preview mode, and validation.        |
| **Platform Inconsistency**    | Path handling differs across OS.                    | Use `tauri::api::path` and `std::path` abstractions.         |
| **Permission Errors**         | App lacks permission to read/write files.           | Pre-validate paths, show clear error messages, skip files.   |
| **File Lock Conflicts**       | Files locked by other applications.                 | Implement retry logic with backoff, allow user to skip.      |
| **Disk Space Issues**         | Insufficient space for copy operations.             | Check available space before operations, show warnings.      |
| **Memory Exhaustion**         | Loading 10,000+ file metadata into memory.          | Stream processing, batch operations, pagination in UI.       |
| **Transaction Failures**      | Operation fails mid-process (power loss, crash).    | Write-ahead logging, atomic operations where possible.       |
| **Symlink/Junction Handling** | Symbolic links may cause infinite loops or errors.  | Detect symlinks, skip or follow based on user preference.    |
| **Unicode & Special Chars**   | Filenames with special characters cause errors.     | Proper UTF-8 handling, sanitize paths, validate filenames.   |
| **Circular Rule Logic**       | Rules that create infinite loops or conflicts.      | Rule validation engine, dependency graph analysis.           |
| **Large File Operations**     | Moving GB-sized files blocks UI or times out.       | Background processing, chunked operations, timeout handling. |

---

## 10. Edge Cases and Error Handling

### 10.1 File System Edge Cases

| Scenario                              | Handling Strategy                                                      |
|---------------------------------------|------------------------------------------------------------------------|
| **File deleted during operation**     | Skip with warning in log, continue processing remaining files.         |
| **Duplicate filenames at destination**| Apply conflict resolution strategy (rename, skip, or prompt user).     |
| **Destination path doesn't exist**    | Create directory tree if `create_folders` enabled, else error.         |
| **Source and destination are same**   | Detect and skip operation, log as no-op.                               |
| **Hidden/system files**               | Respect `include_hidden` setting, default to skip system files.        |
| **Files without extensions**          | Allow rules based on filename only, handle gracefully in UI.           |
| **Network drives / UNC paths**        | Support but warn about potential latency, handle disconnections.       |
| **Case-sensitive vs insensitive FS**  | Normalize paths based on OS (Windows: case-insensitive, Unix: sensitive).|
| **Very long path names (>260 chars)** | Use extended-length path prefix on Windows (`\\?\`), validate limits. |
| **Locked/open files**                 | Retry 3 times with 1s delay, then skip with detailed error message.   |

### 10.2 Rule Engine Edge Cases

| Scenario                              | Handling Strategy                                                      |
|---------------------------------------|------------------------------------------------------------------------|
| **Multiple rules match same file**    | Apply first matching rule by priority, log conflicts for user review.  |
| **Rule with invalid regex**           | Validate regex on rule creation, show syntax error in UI.              |
| **Circular destination patterns**     | Detect cycles during validation, prevent rule creation.                |
| **Dynamic placeholders fail**         | Fallback to default value or skip operation with clear error.          |
| **Rule references deleted folder**    | Validate destinations before execution, mark rule as invalid.          |

### 10.3 Performance Edge Cases

| Scenario                              | Handling Strategy                                                      |
|---------------------------------------|------------------------------------------------------------------------|
| **>10,000 files scanned**             | Stream results, paginate UI display, show progress continuously.       |
| **Deep directory nesting (>100 levels)**| Limit max depth (configurable), warn user if exceeded.              |
| **Many small files vs few large files**| Adaptive batch sizing based on file sizes and operation type.         |
| **Network drive latency**             | Increase timeouts, show speed statistics, allow user to cancel.        |

### 10.4 Transaction and Rollback Strategy

**Transaction Log Format:**
```json
{
  "manifest_id": "uuid-v4",
  "timestamp": "2025-11-01T10:30:00Z",
  "operation_type": "move",
  "completed": true,
  "operations": [
    {
      "source": "/original/path/file.pdf",
      "destination": "/new/path/file.pdf",
      "status": "success",
      "backup_path": "/backup/.tidyfiles/2025-11-01/file.pdf"
    }
  ],
  "total_files": 150,
  "success_count": 148,
  "error_count": 2,
  "errors": [
    {
      "file": "/path/locked_file.docx",
      "error": "File is locked by another process",
      "error_code": "file_locked"
    }
  ]
}
```

**Rollback Implementation:**
1. Read transaction log by manifest ID
2. Reverse operations in LIFO order (last operation first)
3. For move operations: move files back to original locations
4. For copy operations: delete created copies
5. Restore from backup if original location has conflicts
6. Log rollback results for audit trail

**Safety Guarantees:**
- Never delete files without backup (when backup mode enabled)
- Keep transaction logs for 30 days by default (configurable)
- Atomic directory creation (use temp name + rename)
- Verify checksums for critical operations (optional, for large files)

---

## 11. Testing Strategy

### 11.1 Unit Tests (Rust)

**Core Modules to Test:**
- **Rule Engine:** Test rule matching, priority handling, condition evaluation
- **File Operations:** Test move/copy/rename operations with mocked file system
- **Path Validation:** Test path normalization, permission checks, special characters
- **Conflict Resolution:** Test all conflict strategies (skip, rename, overwrite)
- **Transaction Logs:** Test log creation, parsing, rollback operations

**Target Coverage:** ≥80% code coverage for business logic

### 11.2 Integration Tests

**Test Scenarios:**
- End-to-end scan → organize → verify workflow
- Multi-rule application with priorities
- Rollback operations after successful organize
- Error recovery (simulate locked files, permission errors)
- Cross-platform path handling (Windows vs Unix)
- Large dataset handling (10,000+ files)

### 11.3 UI/Frontend Tests

**Testing Tools:** Vitest + Svelte Testing Library
- Component rendering tests
- User interaction flows (rule builder, file selection)
- Event handling from Rust backend
- State management for file lists and progress

### 11.4 Performance Tests

**Benchmarks:**
- Scan 10,000 files in <10 seconds (SSD)
- Organize 1,000 files with 10 rules in <5 seconds
- Memory usage stays <500MB for 10,000 files
- UI remains responsive during operations

**Profiling Tools:**
- Rust: `cargo flamegraph` for CPU profiling
- Frontend: Browser DevTools Performance tab

### 11.5 Manual Testing Checklist

**Pre-Release Testing:**
- [ ] Test on Windows 10/11, macOS 12+, Ubuntu 22.04+
- [ ] Verify all file types (PDF, DOCX, XLSX, images, etc.)
- [ ] Test with special characters in filenames (Unicode, spaces, symbols)
- [ ] Verify conflict resolution for all strategies
- [ ] Test rollback with various scenarios
- [ ] Verify rule templates load correctly
- [ ] Test error messages are clear and actionable
- [ ] Verify dark/light theme switching
- [ ] Test with network drives and external storage
- [ ] Verify transaction logs are created and readable

---

## 12. Future Roadmap

| Milestone             | Description                             | Target |
|-----------------------|-----------------------------------------|--------|
| **MVP**               | Core scan + organize + rule engine + UI | v0.1   |
| **AI Rename**         | Integrate local model for file renaming | v0.2   |
| **Undo / Rollback**   | Full reversible operations              | v0.3   |
| **Preview + Watcher** | Add PDF preview + folder auto-watch     | v0.4   |
| **Public Release**    | Polished UX, tested on all platforms    | v1.0   |

---

## 13. Summary

**TidyFiles** aims to bring automation and intelligence to desktop document management using a performant
native-Rust core and a sleek, reactive Svelte interface.

**Key Differentiators:**
- **Safety-First Architecture:** Transaction logs, rollback support, and backup modes ensure data safety
- **Robust Error Handling:** Comprehensive edge case handling with clear user feedback
- **Performance Optimized:** Async operations, streaming, and batching for handling large file sets
- **Extensible Rule Engine:** Flexible, priority-based rules with validation and preview modes
- **Cross-Platform Native:** True desktop experience on Windows, macOS, and Linux

The application starts as a rule-based organizer but is architected for future extensibility — AI-powered file
understanding, previews, and semantic categorization. The detailed technical specifications and edge case handling
in this PRD provide a solid foundation for building a reliable, production-ready application.

**Development Philosophy:**
- Build for reliability first, speed second
- Never lose user data - comprehensive backup and rollback
- Clear error messages over silent failures
- Testable, modular architecture
- Progressive enhancement (MVP first, AI features later)

---
