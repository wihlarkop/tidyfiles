<script lang="ts">
    import { errorLogStore } from '$lib/stores/errorLog.svelte';
    import { toastStore } from '$lib/stores/toast.svelte';
    import { quickError } from '$lib/utils/errorHandler';
    import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

    interface Props {
        onRetry?: (retryData: any) => Promise<void>;
    }

    let { onRetry }: Props = $props();
    let isExpanded = $state(false);
    let isRetrying = $state(false);
    let retryingId = $state<string | null>(null);
    let showClearAllDialog = $state(false);

    const errors = $derived(errorLogStore.errors);

    async function handleRetry(error: any) {
        if (!onRetry || !error.retryData || isRetrying) return;

        retryingId = error.id;
        isRetrying = true;

        try {
            await onRetry(error.retryData);
            errorLogStore.clear(error.id);
            toastStore.success('Operation retried successfully');
        } catch (err) {
            quickError('Retry operation', err);
        } finally {
            isRetrying = false;
            retryingId = null;
        }
    }

    function prepareClearAll() {
        showClearAllDialog = true;
    }

    function confirmClearAll() {
        errorLogStore.clearAll();
        toastStore.success('Error log cleared');
        showClearAllDialog = false;
    }

    function cancelClearAll() {
        showClearAllDialog = false;
    }

    function formatTime(timestamp: number): string {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now.getTime() - date.getTime();

        // Less than 1 minute
        if (diff < 60000) {
            return 'Just now';
        }

        // Less than 1 hour
        if (diff < 3600000) {
            const minutes = Math.floor(diff / 60000);
            return `${minutes}m ago`;
        }

        // Less than 24 hours
        if (diff < 86400000) {
            const hours = Math.floor(diff / 3600000);
            return `${hours}h ago`;
        }

        // Show the full date
        return date.toLocaleString();
    }
</script>

{#if errors.length > 0}
    <div class="fixed bottom-4 right-4 z-50 max-w-md">
        {#if !isExpanded}
            <!-- Collapsed view - just show error count -->
            <button
                onclick={() => isExpanded = true}
                class="flex items-center gap-2 px-4 py-3 bg-red-600 text-white rounded-lg shadow-lg hover:bg-red-700 transition-colors"
                aria-label="Expand error log"
            >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                </svg>
                <span class="font-medium">{errors.length} Error{errors.length > 1 ? 's' : ''}</span>
                <span class="w-6 h-6 bg-white/20 rounded-full flex items-center justify-center text-xs font-bold">
                    {errors.length}
                </span>
            </button>
        {:else}
            <!-- Expanded view - show error details -->
            <div class="bg-white dark:bg-gray-800 border-2 border-red-600 rounded-lg shadow-xl overflow-hidden max-h-96 flex flex-col">
                <!-- Header -->
                <div class="p-4 bg-red-600 text-white flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                        </svg>
                        <span class="font-semibold">Error Log ({errors.length})</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            onclick={prepareClearAll}
                            class="text-xs px-2 py-1 bg-white/20 hover:bg-white/30 rounded"
                            title="Clear all errors"
                        >
                            Clear
                        </button>
                        <button
                            onclick={() => isExpanded = false}
                            class="hover:bg-white/20 rounded p-1"
                            aria-label="Collapse error log"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>
                </div>

                <!-- Error list -->
                <div class="flex-1 overflow-y-auto">
                    {#each errors as error (error.id)}
                        <div class="p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50">
                            <div class="flex items-start justify-between gap-2 mb-2">
                                <div class="flex-1 min-w-0">
                                    <div class="font-semibold text-gray-900 dark:text-white text-sm">
                                        {error.operation}
                                    </div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">
                                        {formatTime(error.timestamp)}
                                    </div>
                                </div>
                                {#if error.retryable && onRetry}
                                    <button
                                        onclick={() => handleRetry(error)}
                                        disabled={isRetrying && retryingId === error.id}
                                        class="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                                    >
                                        {isRetrying && retryingId === error.id ? 'Retrying...' : 'Retry'}
                                    </button>
                                {/if}
                            </div>

                            <div class="text-sm text-red-600 dark:text-red-400 mb-2">
                                {error.error}
                            </div>

                            {#if error.details}
                                <div class="text-xs text-gray-600 dark:text-gray-400 mb-2 font-mono bg-gray-100 dark:bg-gray-900 p-2 rounded">
                                    {error.details}
                                </div>
                            {/if}

                            {#if error.recovery}
                                <div class="text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 p-2 rounded flex items-start gap-2">
                                    <svg class="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                    </svg>
                                    <span>{error.recovery}</span>
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    </div>

    <!-- Clear All Confirmation Dialog -->
    <ConfirmDialog
        show={showClearAllDialog}
        title="Clear all errors?"
        message="<p>This will remove all errors from the error log.</p><p class='mt-2'>Recovery suggestions and retry data will be lost.</p>"
        confirmText="Clear All"
        variant="warning"
        onConfirm={confirmClearAll}
        onCancel={cancelClearAll}
    />
{/if}
