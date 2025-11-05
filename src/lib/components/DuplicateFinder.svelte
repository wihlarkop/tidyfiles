<script lang="ts">
    import {scanForDuplicates, deleteDuplicateFiles} from '$lib/api/commands';
    import {open} from '@tauri-apps/plugin-dialog';
    import type {DuplicateScanResult, FileInfo} from '$lib/types';
    import {toastStore} from '$lib/stores/toast.svelte';
    import {settingsStore} from '$lib/stores/settings.svelte';
    import {quickError} from '$lib/utils/errorHandler';
    import ProgressBar from '$lib/components/ProgressBar.svelte';
    import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
    import FilePreview from '$lib/components/FilePreview.svelte';
    import Tooltip from '$lib/components/Tooltip.svelte';
    import EmptyState from '$lib/components/EmptyState.svelte';
    import {onMount} from 'svelte';

    let selectedPath = $state('');
    let isScanning = $state(false);
    let isDeleting = $state(false);
    let showConfirmDialog = $state(false);
    let filesToDelete = $state<string[]>([]);
    let scanResult = $state<DuplicateScanResult | null>(null);
    let useDefaultSettings = $state(true);

    // Default hardcoded value (used when toggle is OFF)
    const defaultMinSize = 0;

    let minSize = $state(defaultMinSize); // Minimum file size in bytes
    let scanProgress = $state({ current: 0, message: '' });
    let previewFile = $state<FileInfo | null>(null);

    // Load settings on mount
    onMount(async () => {
        await settingsStore.load();
        if (settingsStore.settings && useDefaultSettings) {
            minSize = settingsStore.settings.duplicate_defaults.min_file_size;
        }
    });

    // Watch for changes in useDefaultSettings
    $effect(() => {
        if (useDefaultSettings && settingsStore.settings) {
            // Use settings from store
            minSize = settingsStore.settings.duplicate_defaults.min_file_size;
        } else if (!useDefaultSettings) {
            // Reset to hardcoded default
            minSize = defaultMinSize;
        }
    });

    async function selectDirectory() {
        const result = await open({
            directory: true,
            multiple: false,
        });

        if (result) {
            selectedPath = result;
        }
    }

    async function startScan(silent: boolean = false) {
        if (!selectedPath) {
            toastStore.warning('Please select a directory first');
            return;
        }

        isScanning = true;
        try {
            const result = await scanForDuplicates(selectedPath, minSize > 0 ? minSize : undefined);
            scanResult = result;

            if (!silent) {
                if (result.total_groups === 0) {
                    toastStore.success('No duplicate files found');
                } else {
                    toastStore.success(`Found ${result.total_duplicates} duplicate files in ${result.total_groups} groups`);
                }
            }
        } catch (error) {
            quickError('Duplicate scan', error);
        } finally {
            isScanning = false;
        }
    }

    function toggleFileSelection(groupIndex: number, fileIndex: number) {
        if (!scanResult) return;

        scanResult.groups[groupIndex].files[fileIndex].is_selected =
            !scanResult.groups[groupIndex].files[fileIndex].is_selected;
    }

    function selectAllInGroup(groupIndex: number, keepFirst: boolean = true) {
        if (!scanResult) return;

        scanResult.groups[groupIndex].files.forEach((file, index) => {
            file.is_selected = keepFirst ? index !== 0 : true;
        });
    }

    function selectOldestInAllGroups() {
        if (!scanResult) return;

        for (const group of scanResult.groups) {
            // Find the oldest file (smallest modified timestamp)
            let oldestIndex = 0;
            let oldestTime = group.files[0].modified || Infinity;

            group.files.forEach((file, index) => {
                const fileTime = file.modified || Infinity;
                if (fileTime < oldestTime) {
                    oldestTime = fileTime;
                    oldestIndex = index;
                }
            });

            // Select all except the oldest
            group.files.forEach((file, index) => {
                file.is_selected = index !== oldestIndex;
            });
        }
        toastStore.info('Selected for deletion: All except oldest in each group');
    }

    function selectNewestInAllGroups() {
        if (!scanResult) return;

        for (const group of scanResult.groups) {
            // Find the newest file (the largest modified timestamp)
            let newestIndex = 0;
            let newestTime = group.files[0].modified || 0;

            group.files.forEach((file, index) => {
                const fileTime = file.modified || 0;
                if (fileTime > newestTime) {
                    newestTime = fileTime;
                    newestIndex = index;
                }
            });

            // Select all except the newest
            group.files.forEach((file, index) => {
                file.is_selected = index !== newestIndex;
            });
        }
        toastStore.info('Selected for deletion: All except newest in each group');
    }

    function selectAllDuplicates() {
        if (!scanResult) return;

        for (const group of scanResult.groups) {
            // Select all except the first in each group
            group.files.forEach((file, index) => {
                file.is_selected = index !== 0;
            });
        }
        toastStore.info('Selected for deletion: All except first in each group');
    }

    function clearAllSelections() {
        if (!scanResult) return;

        for (const group of scanResult.groups) {
            group.files.forEach(file => {
                file.is_selected = false;
            });
        }
        toastStore.info('Cleared all selections');
    }

    function prepareDelete() {
        if (!scanResult || isDeleting) return;

        const files: string[] = [];
        for (const group of scanResult.groups) {
            for (const file of group.files) {
                if (file.is_selected) {
                    files.push(file.path);
                }
            }
        }

        if (files.length === 0) {
            toastStore.warning('No files selected for deletion');
            return;
        }

        // Set files and show a confirmation dialog
        filesToDelete = files;
        showConfirmDialog = true;
    }

    async function confirmDelete() {
        // Close dialog first
        showConfirmDialog = false;

        // Now proceed with deletion
        isDeleting = true;
        try {
            const manifest = await deleteDuplicateFiles(filesToDelete);

            if (manifest.error_count > 0) {
                toastStore.warning(`Deleted ${manifest.success_count} file${manifest.success_count !== 1 ? 's' : ''}, ${manifest.error_count} failed`);
            } else {
                toastStore.success(`Successfully deleted ${manifest.success_count} duplicate file${manifest.success_count !== 1 ? 's' : ''}`);
            }

            // Re-scan to update the list (silent to avoid double notification)
            await startScan(true);
        } catch (error) {
            quickError('Delete duplicate files', error);
        } finally {
            isDeleting = false;
            filesToDelete = [];
        }
    }

    function cancelDelete() {
        showConfirmDialog = false;
        filesToDelete = [];
    }

    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    function formatDate(timestamp: number | null): string {
        if (!timestamp) return 'N/A';
        return new Date(timestamp * 1000).toLocaleString();
    }

    function getSelectedCount(): number {
        if (!scanResult) return 0;

        let count = 0;
        for (const group of scanResult.groups) {
            for (const file of group.files) {
                if (file.is_selected) count++;
            }
        }
        return count;
    }

    function getSelectedSize(): number {
        if (!scanResult) return 0;

        let size = 0;
        for (const group of scanResult.groups) {
            for (const file of group.files) {
                if (file.is_selected) size += group.file_size;
            }
        }
        return size;
    }

    function createFileInfoForPreview(file: any, size: number): FileInfo {
        const extension = file.name.includes('.') ? file.name.split('.').pop() || null : null;
        return {
            path: file.path,
            name: file.name,
            extension,
            size,
            created: null,
            modified: file.modified,
            is_hidden: false
        };
    }

    function handlePreview(file: any, size: number) {
        previewFile = createFileInfoForPreview(file, size);
    }

</script>

<div class="h-full flex flex-col">
    <div class="p-6 border-b border-border">
        <div>
            <h2 class="text-2xl font-bold">Duplicate File Finder</h2>
            <p class="text-sm text-muted-foreground">Find and remove duplicate files to free up disk space</p>
        </div>
    </div>

    <div class="flex-1 overflow-auto p-6">
        <div class="max-w-6xl mx-auto space-y-6">
            <!-- Scan Controls -->
            <div class="space-y-4 p-4 border border-border rounded-lg">
                <div>
                    <label for="scan-directory-input" class="text-sm font-medium">Scan Directory</label>
                    <div class="flex gap-2 mt-1">
                        <input
                                id="scan-directory-input"
                                type="text"
                                value={selectedPath}
                                readonly
                                placeholder="Select a directory to scan"
                                class="flex-1 px-3 py-2 rounded border border-input bg-background"
                        />
                        <button
                                onclick={selectDirectory}
                                class="px-4 py-2 border border-border rounded hover:bg-secondary"
                        >
                            Browse
                        </button>
                    </div>
                </div>

                <div>
                    <div class="flex items-center justify-between mb-1">
                        <label for="min-size" class="text-sm font-medium">Minimum File Size (bytes)</label>
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="checkbox" bind:checked={useDefaultSettings} class="rounded"/>
                            <span class="text-xs text-muted-foreground">Use default settings</span>
                        </label>
                    </div>
                    <input
                            id="min-size"
                            type="number"
                            bind:value={minSize}
                            min="0"
                            disabled={useDefaultSettings}
                            placeholder="0 (scan all files)"
                            class="w-full px-3 py-2 rounded border border-input bg-background"
                    />
                    <p class="text-xs text-muted-foreground mt-1">
                        Skip files smaller than this size (0 = scan all files). 1 MB = 1,048,576 bytes
                    </p>
                </div>

                <button
                        onclick={() => startScan()}
                        disabled={!selectedPath || isScanning}
                        class="w-full px-4 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50"
                >
                    {isScanning ? 'Scanning...' : 'Scan for Duplicates'}
                </button>

                {#if isScanning}
                    <div class="space-y-2">
                        <ProgressBar message={scanProgress.message || 'Scanning directory and calculating hashes...'} />
                    </div>
                {/if}
            </div>

            <!-- Results -->
            {#if scanResult}
                <div class="space-y-4">
                    <!-- Statistics -->
                    <div class="grid grid-cols-3 gap-4">
                        <div class="p-4 bg-secondary rounded-lg">
                            <div class="text-2xl font-bold">{scanResult.total_groups}</div>
                            <div class="text-sm text-muted-foreground">Duplicate Groups</div>
                        </div>
                        <div class="p-4 bg-secondary rounded-lg">
                            <div class="text-2xl font-bold">{scanResult.total_duplicates}</div>
                            <div class="text-sm text-muted-foreground">Duplicate Files</div>
                        </div>
                        <div class="p-4 bg-secondary rounded-lg">
                            <div class="text-2xl font-bold">{formatBytes(scanResult.wasted_space)}</div>
                            <div class="text-sm text-muted-foreground">Wasted Space</div>
                        </div>
                    </div>

                    {#if scanResult.groups.length === 0}
                        <div class="border border-border rounded-lg">
                            <EmptyState
                                icon="✨"
                                title="No Duplicates Found!"
                                description="Your directory is clean - no duplicate files detected. Try scanning a different directory or adjust the minimum file size setting."
                                actionText="Scan Another Directory"
                                onAction={selectDirectory}
                            />
                        </div>
                    {:else}
                        <!-- Bulk Selection Options -->
                        <div class="p-4 bg-secondary/50 rounded-lg space-y-3">
                            <div class="flex items-center justify-between">
                                <div>
                                    <div class="text-sm font-medium">Quick Select:</div>
                                    <div class="text-xs text-muted-foreground">✓ = Will be deleted | ☐ = Will be kept</div>
                                </div>
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <Tooltip text="Select all duplicate files except the first one in each group for deletion">
                                    {#snippet children()}
                                        <button
                                                onclick={selectAllDuplicates}
                                                disabled={isDeleting}
                                                class="px-3 py-2 text-sm bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
                                        >
                                            ✓ All Except First
                                        </button>
                                    {/snippet}
                                </Tooltip>
                                <Tooltip text="Keep the oldest file (by modification date) and select all newer duplicates for deletion">
                                    {#snippet children()}
                                        <button
                                                onclick={selectOldestInAllGroups}
                                                disabled={isDeleting}
                                                class="px-3 py-2 text-sm border border-border rounded hover:bg-secondary disabled:opacity-50"
                                        >
                                            ✓ All Except Oldest
                                        </button>
                                    {/snippet}
                                </Tooltip>
                                <Tooltip text="Keep the newest file (by modification date) and select all older duplicates for deletion">
                                    {#snippet children()}
                                        <button
                                                onclick={selectNewestInAllGroups}
                                                disabled={isDeleting}
                                                class="px-3 py-2 text-sm border border-border rounded hover:bg-secondary disabled:opacity-50"
                                        >
                                            ✓ All Except Newest
                                        </button>
                                    {/snippet}
                                </Tooltip>
                                <Tooltip text="Deselect all files">
                                    {#snippet children()}
                                        <button
                                                onclick={clearAllSelections}
                                                disabled={isDeleting}
                                                class="px-3 py-2 text-sm border border-border rounded hover:bg-secondary disabled:opacity-50"
                                        >
                                            ☐ Clear All
                                        </button>
                                    {/snippet}
                                </Tooltip>
                            </div>
                        </div>

                        <!-- Action Bar -->
                        <div class="flex items-center justify-between p-4 bg-secondary rounded-lg">
                            <div class="text-sm">
                                <span class="font-medium">{getSelectedCount()} files selected</span>
                                <span class="text-muted-foreground"> ({formatBytes(getSelectedSize())} to free)</span>
                            </div>
                            <button
                                    onclick={prepareDelete}
                                    disabled={getSelectedCount() === 0 || isDeleting}
                                    class="px-4 py-2 bg-destructive text-destructive-foreground rounded hover:opacity-90 disabled:opacity-50"
                            >
                                {isDeleting ? 'Deleting...' : 'Delete Selected'}
                            </button>
                        </div>

                        {#if isDeleting}
                            <div class="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
                                <div class="flex items-center gap-3">
                                    <div class="w-full space-y-2">
                                        <div class="flex items-center justify-between text-sm">
                                            <span class="font-medium text-destructive">Deleting files...</span>
                                            <span class="text-muted-foreground">Please wait</span>
                                        </div>
                                        <div class="w-full h-2 bg-secondary rounded-full overflow-hidden">
                                            <div class="h-full bg-destructive animate-pulse"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/if}

                        <!-- Duplicate Groups -->
                        <div class="space-y-4">
                            {#each scanResult.groups as group, groupIndex (group.hash)}
                                <div class="border border-border rounded-lg overflow-hidden">
                                    <div class="p-4 bg-secondary/50 border-b border-border">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <div class="font-medium">
                                                    {group.files.length} identical files
                                                </div>
                                                <div class="text-sm text-muted-foreground">
                                                    {formatBytes(group.file_size)} each • Wasting {formatBytes(group.file_size * (group.files.length - 1))}
                                                </div>
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                        onclick={() => selectAllInGroup(groupIndex, true)}
                                                        disabled={isDeleting}
                                                        class="px-3 py-1 text-sm border border-border rounded hover:bg-secondary disabled:opacity-50"
                                                        title="Select all in this group except the first one"
                                                >
                                                    ✓ All Except First
                                                </button>
                                                <button
                                                        onclick={() => selectAllInGroup(groupIndex, false)}
                                                        disabled={isDeleting}
                                                        class="px-3 py-1 text-sm border border-border rounded hover:bg-secondary disabled:opacity-50"
                                                        title="Select all files in this group for deletion"
                                                >
                                                    ✓ All
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="divide-y divide-border">
                                        {#each group.files as file, fileIndex (file.path)}
                                            <div class="p-3 flex items-center gap-3 hover:bg-secondary/30">
                                                <input
                                                        type="checkbox"
                                                        checked={file.is_selected}
                                                        onchange={() => toggleFileSelection(groupIndex, fileIndex)}
                                                        disabled={isDeleting}
                                                        class="rounded disabled:opacity-50"
                                                />
                                                <div class="flex-1 min-w-0">
                                                    <div class="font-medium truncate">{file.name}</div>
                                                    <div class="text-xs text-muted-foreground font-mono truncate" title={file.path}>
                                                        {file.path}
                                                    </div>
                                                    <div class="text-xs text-muted-foreground mt-1">
                                                        Modified: {formatDate(file.modified)}
                                                    </div>
                                                </div>
                                                <button
                                                        onclick={() => handlePreview(file, group.file_size)}
                                                        class="px-3 py-1 text-sm border border-border rounded hover:bg-secondary"
                                                        title="Preview file"
                                                >
                                                    👁️ Preview
                                                </button>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>

    <!-- Confirmation Dialog -->
    <ConfirmDialog
        show={showConfirmDialog}
        title="Delete {filesToDelete.length} file{filesToDelete.length > 1 ? 's' : ''}?"
        message="<p>This will free up <strong>{formatBytes(getSelectedSize())}</strong>.</p><p class='text-destructive font-medium mt-2'>⚠️ This action cannot be undone!</p>"
        confirmText="Delete"
        variant="destructive"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
    />

    <!-- File Preview -->
    <FilePreview file={previewFile} onClose={() => previewFile = null} />
</div>
