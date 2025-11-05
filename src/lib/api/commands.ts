import { invoke } from '@tauri-apps/api/core';
import type {
  ScanOptions,
  ScanResult,
  Rule,
  FileInfo,
  RuleMatch,
  OrganizeOptions,
  OperationManifest,
  RollbackResult,
  DuplicateScanResult,
  AppSettings,
} from '$lib/types';

export async function scanDirectory(
  path: string,
  options: ScanOptions
): Promise<ScanResult> {
  return await invoke<ScanResult>('scan_directory', { path, options });
}

export async function previewRuleMatches(
  rules: Rule[],
  files: FileInfo[]
): Promise<RuleMatch[]> {
  return await invoke<RuleMatch[]>('preview_rule_matches', { rules, files });
}

export async function organizeFiles(
  files: FileInfo[],
  rules: Rule[],
  options: OrganizeOptions
): Promise<OperationManifest> {
  return await invoke<OperationManifest>('organize_files', { files, rules, options });
}

export async function getOperationHistory(): Promise<OperationManifest[]> {
  return await invoke<OperationManifest[]>('get_operation_history');
}

export async function rollbackOperation(manifestId: string): Promise<RollbackResult> {
  return await invoke<RollbackResult>('rollback_operation', { manifestId });
}

export async function exportOperationLog(manifestId: string, exportPath: string): Promise<void> {
  return await invoke<void>('export_operation_log', { manifestId, exportPath });
}

export async function deleteOperationLog(manifestId: string): Promise<void> {
  return await invoke<void>('delete_operation_log', { manifestId });
}

export async function scanForDuplicates(directory: string, minSize?: number): Promise<DuplicateScanResult> {
  return await invoke<DuplicateScanResult>('scan_for_duplicates', { directory, minSize });
}

export async function deleteDuplicateFiles(filePaths: string[]): Promise<OperationManifest> {
  return await invoke<OperationManifest>('delete_duplicate_files', { filePaths });
}

export async function readFileContent(filePath: string, maxBytes?: number): Promise<string> {
  return await invoke<string>('read_file_content', { filePath, maxBytes });
}

// Settings
export async function loadSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('load_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return await invoke<void>('save_settings', { settings });
}

export async function resetSettings(): Promise<AppSettings> {
  return await invoke<AppSettings>('reset_settings');
}
