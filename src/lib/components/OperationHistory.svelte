<script lang="ts">
    import {getOperationHistory, rollbackOperation, exportOperationLog, deleteOperationLog} from '$lib/api/commands';
    import {save} from '@tauri-apps/plugin-dialog';
    import type {OperationManifest, RollbackResult} from '$lib/types';
    import {onMount} from 'svelte';
    import {toastStore} from '$lib/stores/toast.svelte';
    import {quickError} from '$lib/utils/errorHandler';
    import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
    import Tooltip from '$lib/components/Tooltip.svelte';
    import EmptyState from '$lib/components/EmptyState.svelte';

    let history = $state<OperationManifest[]>([]);
    let isLoading = $state(true);
    let selectedManifest = $state<OperationManifest | null>(null);
    let isRollingBack = $state(false);
    let rollbackResult = $state<RollbackResult | null>(null);
    let showRollbackResultModal = $state(false);

    // Search and filter state
    let searchQuery = $state('');
    let filterType = $state<string>('all');
    let filterStatus = $state<string>('all');

    // Modal states
    let showRollbackDialog = $state(false);
    let showDeleteDialog = $state(false);
    let pendingActionManifestId = $state<string | null>(null);

    onMount(async () => {
        await loadHistory();
    });

    async function loadHistory() {
        isLoading = true;
        try {
            history = await getOperationHistory();
        } catch (error) {
            quickError('Load operation history', error);
        } finally {
            isLoading = false;
        }
    }

    function prepareRollback(manifestId: string) {
        pendingActionManifestId = manifestId;
        showRollbackDialog = true;
    }

    async function confirmRollback() {
        if (!pendingActionManifestId) return;

        showRollbackDialog = false;
        const manifestId = pendingActionManifestId;
        pendingActionManifestId = null;

        isRollingBack = true;
        try {
            const result = await rollbackOperation(manifestId);
            rollbackResult = result;

            // Reload history after rollback
            await loadHistory();

            // Show the result modal
            showRollbackResultModal = true;

            if (result.failed_count > 0) {
                toastStore.warning(`Rollback complete: ${result.restored_count} restored, ${result.failed_count} failed`);
            } else {
                toastStore.success(`Rollback complete: ${result.restored_count} files restored`);
            }
        } catch (error) {
            quickError('Rollback operation', error);
        } finally {
            isRollingBack = false;
        }
    }

    function cancelRollback() {
        showRollbackDialog = false;
        pendingActionManifestId = null;
    }

    async function handleExport(manifestId: string) {
        const exportPath = await save({
            defaultPath: `operation-${manifestId}.csv`,
            filters: [{
                name: 'CSV',
                extensions: ['csv']
            }]
        });

        if (exportPath) {
            try {
                await exportOperationLog(manifestId, exportPath);
                toastStore.success('Operation log exported successfully!');
            } catch (error) {
                quickError('Export operation log', error);
            }
        }
    }

    function prepareDelete(manifestId: string) {
        pendingActionManifestId = manifestId;
        showDeleteDialog = true;
    }

    async function confirmDelete() {
        if (!pendingActionManifestId) return;

        showDeleteDialog = false;
        const manifestId = pendingActionManifestId;
        pendingActionManifestId = null;

        try {
            await deleteOperationLog(manifestId);
            await loadHistory();
            toastStore.success('Operation log deleted');
        } catch (error) {
            quickError('Delete operation log', error);
        }
    }

    function cancelDelete() {
        showDeleteDialog = false;
        pendingActionManifestId = null;
    }

    function formatDate(timestamp: number): string {
        return new Date(timestamp * 1000).toLocaleString();
    }

    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    function formatOperationType(type: string): string {
        switch (type) {
            case 'duplicate_deletion':
                return 'Duplicate Deletion';
            case 'move':
                return 'Move Files';
            case 'copy':
                return 'Copy Files';
            case 'mixed':
                return 'Mixed Operations';
            default:
                return type;
        }
    }

    function canRollback(manifest: OperationManifest): boolean {
        // Can't roll back deletions or already rolled back operations
        return manifest.operation_type !== 'duplicate_deletion' && !manifest.rolled_back;
    }

    function getRollbackTooltip(manifest: OperationManifest): string {
        if (manifest.operation_type === 'duplicate_deletion') {
            return 'Deleted files cannot be recovered';
        }
        if (manifest.rolled_back) {
            return 'This operation has already been undone';
        }
        return 'Undo this operation';
    }

    // Filtered history based on search and filters
    let filteredHistory = $derived.by(() => {
        let result = history;

        // Apply search filter
        if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            result = result.filter(manifest => {
                const dateStr = formatDate(manifest.timestamp).toLowerCase();
                const typeStr = formatOperationType(manifest.operation_type).toLowerCase();
                return dateStr.includes(query) || typeStr.includes(query);
            });
        }

        // Apply operation type filter
        if (filterType !== 'all') {
            result = result.filter(manifest => manifest.operation_type === filterType);
        }

        // Apply status filter
        if (filterStatus !== 'all') {
            result = result.filter(manifest => {
                if (filterStatus === 'success') return manifest.error_count === 0 && !manifest.rolled_back;
                if (filterStatus === 'errors') return manifest.error_count > 0;
                if (filterStatus === 'rolled_back') return manifest.rolled_back;
                return true;
            });
        }

        return result;
    });

    // Calculate statistics
    let stats = $derived.by(() => {
        const totalOperations = history.length;
        const totalFiles = history.reduce((sum, m) => sum + m.total_files, 0);
        const successfulFiles = history.reduce((sum, m) => sum + m.success_count, 0);
        const failedFiles = history.reduce((sum, m) => sum + m.error_count, 0);
        const successRate = totalFiles > 0 ? (successfulFiles / totalFiles * 100) : 0;

        // Calculate total space saved from duplicate deletions
        let spaceSaved = 0;
        for (const manifest of history) {
            if (manifest.operation_type === 'duplicate_deletion') {
                for (const op of manifest.operations) {
                    if (op.status === 'success') {
                        spaceSaved += op.file_size;
                    }
                }
            }
        }

        // Count rolled back operations
        const rolledBackCount = history.filter(m => m.rolled_back).length;

        return {
            totalOperations,
            totalFiles,
            successfulFiles,
            failedFiles,
            successRate,
            spaceSaved,
            rolledBackCount
        };
    });
</script>

<div class="h-full flex flex-col">
    <div class="p-6 border-b border-border space-y-4">
        <div class="flex items-center justify-between">
            <div>
                <h2 class="text-2xl font-bold">Operation History</h2>
                <p class="text-sm text-muted-foreground">View and manage past file organization operations</p>
            </div>
            <button
                    onclick={loadHistory}
                    class="px-4 py-2 border border-border rounded-lg hover:bg-secondary"
            >
                Refresh
            </button>
        </div>

        <!-- Search and Filters -->
        <div class="flex gap-3 flex-wrap">
            <div class="flex-1 min-w-[200px]">
                <input
                        type="text"
                        bind:value={searchQuery}
                        placeholder="Search by date or operation type..."
                        class="w-full px-4 py-2 rounded-lg border border-input bg-background"
                />
            </div>
            <select
                    bind:value={filterType}
                    class="px-4 py-2 rounded-lg border border-input bg-background"
            >
                <option value="all">All Types</option>
                <option value="duplicate_deletion">Duplicate Deletion</option>
                <option value="move">Move Files</option>
                <option value="copy">Copy Files</option>
                <option value="mixed">Mixed Operations</option>
            </select>
            <select
                    bind:value={filterStatus}
                    class="px-4 py-2 rounded-lg border border-input bg-background"
            >
                <option value="all">All Status</option>
                <option value="success">Success Only</option>
                <option value="errors">Has Errors</option>
                <option value="rolled_back">Rolled Back</option>
            </select>
        </div>

        <!-- Results count -->
        {#if searchQuery || filterType !== 'all' || filterStatus !== 'all'}
            <div class="text-sm text-muted-foreground">
                Showing {filteredHistory.length} of {history.length} operations
            </div>
        {/if}
    </div>

    <!-- Statistics Dashboard -->
    {#if history.length > 0 && !isLoading}
        <div class="p-6 bg-secondary/30 border-b border-border">
            <h3 class="text-lg font-semibold mb-4">Statistics</h3>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div class="p-4 bg-background rounded-lg border border-border">
                    <div class="text-2xl font-bold text-primary">{stats.totalOperations}</div>
                    <div class="text-sm text-muted-foreground">Total Operations</div>
                </div>
                <div class="p-4 bg-background rounded-lg border border-border">
                    <div class="text-2xl font-bold text-primary">{stats.totalFiles}</div>
                    <div class="text-sm text-muted-foreground">Files Processed</div>
                </div>
                <div class="p-4 bg-background rounded-lg border border-border">
                    <div class="text-2xl font-bold text-green-600">{stats.successRate.toFixed(1)}%</div>
                    <div class="text-sm text-muted-foreground">Success Rate</div>
                </div>
                <div class="p-4 bg-background rounded-lg border border-border">
                    <div class="text-2xl font-bold text-blue-600">{formatBytes(stats.spaceSaved)}</div>
                    <div class="text-sm text-muted-foreground">Space Freed</div>
                </div>
            </div>
            <div class="grid grid-cols-3 gap-4 mt-4">
                <div class="p-3 bg-green-600/10 rounded-lg border border-green-600/20">
                    <div class="text-xl font-bold text-green-600">{stats.successfulFiles}</div>
                    <div class="text-xs text-muted-foreground">Successful</div>
                </div>
                <div class="p-3 bg-red-600/10 rounded-lg border border-red-600/20">
                    <div class="text-xl font-bold text-red-600">{stats.failedFiles}</div>
                    <div class="text-xs text-muted-foreground">Failed</div>
                </div>
                <div class="p-3 bg-blue-600/10 rounded-lg border border-blue-600/20">
                    <div class="text-xl font-bold text-blue-600">{stats.rolledBackCount}</div>
                    <div class="text-xs text-muted-foreground">Rolled Back</div>
                </div>
            </div>
        </div>
    {/if}

    <div class="flex-1 overflow-auto p-6">
        {#if isLoading}
            <div class="flex items-center justify-center h-64">
                <div class="text-muted-foreground">Loading history...</div>
            </div>
        {:else if history.length === 0}
            <EmptyState
                icon="📋"
                title="No Operation History"
                description="Your file organization history will appear here after you complete file operations. Start by organizing files in the Organizer tab or finding duplicates."
            />
        {:else if filteredHistory.length === 0}
            <EmptyState
                icon="🔍"
                title="No Results Found"
                description="Try adjusting your search query or changing the filter settings above. You can filter by operation type or status."
            />
        {:else}
            <div class="space-y-4 max-w-6xl mx-auto">
                {#each filteredHistory as manifest (manifest.manifest_id)}
                    <div class="border border-border rounded-lg overflow-hidden">
                        <div class="p-4 bg-secondary/50">
                            <div class="flex items-start justify-between">
                                <div class="flex-1">
                                    <div class="flex items-center gap-3">
                                        <div class="text-lg font-semibold">
                                            {formatDate(manifest.timestamp)}
                                        </div>
                                        {#if manifest.rolled_back}
                      <span class="px-2 py-1 bg-blue-600/20 text-blue-600 text-xs rounded">
                        Rolled back
                      </span>
                                        {/if}
                                        {#if manifest.success_count > 0}
                      <span class="px-2 py-1 bg-green-600/20 text-green-600 text-xs rounded">
                        {manifest.success_count} successful
                      </span>
                                        {/if}
                                        {#if manifest.error_count > 0}
                      <span class="px-2 py-1 bg-red-600/20 text-red-600 text-xs rounded">
                        {manifest.error_count} errors
                      </span>
                                        {/if}
                                    </div>
                                    <div class="text-sm text-muted-foreground mt-1">
                                        {manifest.total_files} files • {formatOperationType(manifest.operation_type)}
                                    </div>
                                </div>

                                <div class="flex gap-2">
                                    <Tooltip text={getRollbackTooltip(manifest)}>
                                        {#snippet children()}
                                            <button
                                                    onclick={() => prepareRollback(manifest.manifest_id)}
                                                    disabled={isRollingBack || !canRollback(manifest)}
                                                    class="px-3 py-1 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                                            >
                                                {manifest.operation_type === 'duplicate_deletion' ? 'N/A' : manifest.rolled_back ? 'Undone' : isRollingBack ? 'Undoing...' : 'Undo'}
                                            </button>
                                        {/snippet}
                                    </Tooltip>
                                    <Tooltip text="Export operation log to CSV file for record keeping">
                                        {#snippet children()}
                                            <button
                                                    onclick={() => handleExport(manifest.manifest_id)}
                                                    class="px-3 py-1 border border-border rounded hover:bg-secondary text-sm"
                                            >
                                                Export
                                            </button>
                                        {/snippet}
                                    </Tooltip>
                                    <Tooltip text="Show detailed information about this operation">
                                        {#snippet children()}
                                            <button
                                                    onclick={() => selectedManifest = selectedManifest?.manifest_id === manifest.manifest_id ? null : manifest}
                                                    class="px-3 py-1 border border-border rounded hover:bg-secondary text-sm"
                                            >
                                                {selectedManifest?.manifest_id === manifest.manifest_id ? 'Hide' : 'Details'}
                                            </button>
                                        {/snippet}
                                    </Tooltip>
                                    <Tooltip text="Permanently remove this operation log from history">
                                        {#snippet children()}
                                            <button
                                                    onclick={() => prepareDelete(manifest.manifest_id)}
                                                    class="px-3 py-1 text-destructive hover:bg-destructive/10 rounded text-sm"
                                            >
                                                Delete
                                            </button>
                                        {/snippet}
                                    </Tooltip>
                                </div>
                            </div>
                        </div>

                        {#if selectedManifest?.manifest_id === manifest.manifest_id}
                            <div class="p-4 border-t border-border">
                                <div class="space-y-4">
                                    {#if manifest.errors.length > 0}
                                        <div>
                                            <h4 class="font-medium mb-2">Errors ({manifest.errors.length})</h4>
                                            <div class="space-y-2">
                                                {#each manifest.errors as error}
                                                    <div class="p-2 bg-destructive/10 border border-destructive/20 text-destructive rounded text-sm">
                                                        <div class="font-medium">{error.file}</div>
                                                        <div class="text-xs">{error.error} ({error.error_code})</div>
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    {/if}

                                    {#if manifest.rollback_details}
                                        <div class="p-4 bg-blue-600/10 border border-blue-600/20 rounded-lg">
                                            <h4 class="font-medium text-blue-600 mb-2">Rollback Information</h4>
                                            <div class="grid grid-cols-2 gap-4 text-sm">
                                                <div>
                                                    <span class="text-muted-foreground">Restored:</span>
                                                    <span class="ml-2 font-semibold text-green-600">{manifest.rollback_details.restored_count} files</span>
                                                </div>
                                                <div>
                                                    <span class="text-muted-foreground">Failed:</span>
                                                    <span class="ml-2 font-semibold text-red-600">{manifest.rollback_details.failed_count} files</span>
                                                </div>
                                            </div>
                                            {#if manifest.rollback_details.errors.length > 0}
                                                <div class="mt-3">
                                                    <p class="text-xs text-muted-foreground mb-2">Rollback Errors:</p>
                                                    <div class="space-y-1">
                                                        {#each manifest.rollback_details.errors as error}
                                                            <div class="text-xs text-red-600">
                                                                {error.file}: {error.error}
                                                            </div>
                                                        {/each}
                                                    </div>
                                                </div>
                                            {/if}
                                        </div>
                                    {/if}

                                    <div>
                                        <h4 class="font-medium mb-2">Operations ({manifest.operations.length})</h4>
                                        <div class="max-h-96 overflow-auto border border-border rounded">
                                            <table class="w-full text-sm">
                                                <thead class="bg-muted sticky top-0">
                                                <tr>
                                                    <th class="px-3 py-2 text-left">Source</th>
                                                    <th class="px-3 py-2 text-left">Destination</th>
                                                    <th class="px-3 py-2 text-left">Size</th>
                                                    <th class="px-3 py-2 text-left">Status</th>
                                                </tr>
                                                </thead>
                                                <tbody>
                                                {#each manifest.operations as op}
                                                    <tr class="border-t border-border">
                                                        <td class="px-3 py-2 font-mono text-xs truncate max-w-xs"
                                                            title={op.source}>
                                                            {op.source}
                                                        </td>
                                                        <td class="px-3 py-2 font-mono text-xs truncate max-w-xs"
                                                            title={op.destination}>
                                                            {op.destination}
                                                        </td>
                                                        <td class="px-3 py-2 text-muted-foreground">
                                                            {formatBytes(op.file_size)}
                                                        </td>
                                                        <td class="px-3 py-2">
                                <span class="px-2 py-1 rounded text-xs {
                                  op.status === 'success' ? 'bg-green-600/20 text-green-600' :
                                  op.status === 'failed' ? 'bg-red-600/20 text-red-600' :
                                  'bg-gray-600/20 text-gray-600'
                                }">
                                  {op.status}
                                </span>
                                                        </td>
                                                    </tr>
                                                {/each}
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Confirmation Dialogs -->
    <ConfirmDialog
        show={showRollbackDialog}
        title="Undo this operation?"
        message="<p>All files will be moved back to their original locations.</p><p class='text-blue-600 font-medium mt-2'>This will restore the previous state of the files.</p>"
        confirmText="Undo Operation"
        variant="warning"
        onConfirm={confirmRollback}
        onCancel={cancelRollback}
    />

    <ConfirmDialog
        show={showDeleteDialog}
        title="Delete operation log?"
        message="<p>This will permanently delete this operation log from history.</p><p class='text-destructive font-medium mt-2'>⚠️ This action cannot be undone!</p>"
        confirmText="Delete"
        variant="destructive"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
    />

    <!-- Rollback Result Modal -->
    {#if showRollbackResultModal && rollbackResult}
        <div
            class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
            onclick={() => showRollbackResultModal = false}
            onkeydown={(e) => e.key === 'Escape' && (showRollbackResultModal = false)}
            role="button"
            tabindex="-1"
        >
            <div
                class="bg-background border border-border rounded-lg shadow-2xl max-w-2xl w-full max-h-[90vh] flex flex-col"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="dialog"
                aria-modal="true"
                tabindex="0"
            >
                <!-- Header -->
                <div class="p-6 border-b border-border">
                    <div class="flex items-start justify-between">
                        <div>
                            <h3 class="text-xl font-semibold">Rollback Complete</h3>
                            <p class="text-sm text-muted-foreground mt-1">Operation has been undone</p>
                        </div>
                        <button
                            onclick={() => showRollbackResultModal = false}
                            class="p-2 hover:bg-secondary rounded transition-colors"
                            aria-label="Close rollback results"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>
                </div>

                <!-- Content -->
                <div class="flex-1 overflow-auto p-6">
                    <div class="grid grid-cols-2 gap-4 mb-6">
                        <div class="p-4 bg-green-600/10 border border-green-600/20 rounded-lg">
                            <div class="text-3xl font-bold text-green-600">{rollbackResult.restored_count}</div>
                            <div class="text-sm text-muted-foreground mt-1">Files Restored</div>
                        </div>
                        <div class="p-4 bg-red-600/10 border border-red-600/20 rounded-lg">
                            <div class="text-3xl font-bold text-red-600">{rollbackResult.failed_count}</div>
                            <div class="text-sm text-muted-foreground mt-1">Failed</div>
                        </div>
                    </div>

                    {#if rollbackResult.errors.length > 0}
                        <div class="mb-4">
                            <h4 class="font-semibold mb-3 text-destructive">Errors During Rollback</h4>
                            <div class="space-y-2 max-h-64 overflow-auto">
                                {#each rollbackResult.errors as error}
                                    <div class="p-3 bg-destructive/10 border border-destructive/20 rounded text-sm">
                                        <div class="font-mono text-xs mb-1 truncate" title={error.file}>
                                            {error.file}
                                        </div>
                                        <div class="text-destructive">
                                            {error.error}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {:else}
                        <div class="p-4 bg-green-600/10 border border-green-600/20 rounded-lg text-center">
                            <svg class="w-12 h-12 text-green-600 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                            </svg>
                            <p class="text-green-600 font-semibold">All files successfully restored!</p>
                            <p class="text-sm text-muted-foreground mt-1">The operation has been completely undone.</p>
                        </div>
                    {/if}
                </div>

                <!-- Footer -->
                <div class="p-4 border-t border-border flex justify-end">
                    <button
                        onclick={() => showRollbackResultModal = false}
                        class="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>
