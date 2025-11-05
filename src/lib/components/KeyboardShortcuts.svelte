<script lang="ts">
    interface Props {
        isOpen: boolean;
        onClose: () => void;
    }

    let {isOpen, onClose}: Props = $props();

    const shortcuts = [
        {category: 'Navigation', items: [
            {keys: ['Ctrl', '1'], description: 'Switch to Organizer tab'},
            {keys: ['Ctrl', '2'], description: 'Switch to Duplicates tab'},
            {keys: ['Ctrl', '3'], description: 'Switch to History tab'},
        ]},
        {category: 'Actions', items: [
            {keys: ['Ctrl', 'O'], description: 'Open folder dialog'},
            {keys: ['Ctrl', 'Enter'], description: 'Start scan (when folder selected)'},
            {keys: ['Ctrl', 'Shift', 'T'], description: 'Toggle dark/light theme'},
        ]},
        {category: 'General', items: [
            {keys: ['Escape'], description: 'Close modals and previews'},
            {keys: ['Ctrl', '/'], description: 'Show keyboard shortcuts'},
        ]},
    ];
</script>

{#if isOpen}
    <div
            class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
            onclick={onClose}
            onkeydown={(e) => e.key === 'Escape' && onClose()}
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
            <div class="p-6 border-b border-border flex items-center justify-between">
                <div>
                    <h3 class="text-xl font-semibold">Keyboard Shortcuts</h3>
                    <p class="text-sm text-muted-foreground mt-1">Quick access to common actions</p>
                </div>
                <button
                        onclick={onClose}
                        class="p-2 hover:bg-secondary rounded transition-colors"
                        aria-label="Close keyboard shortcuts"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-auto p-6">
                {#each shortcuts as section}
                    <div class="mb-6 last:mb-0">
                        <h4 class="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            {section.category}
                        </h4>
                        <div class="space-y-2">
                            {#each section.items as shortcut}
                                <div class="flex items-center justify-between py-2">
                                    <span class="text-sm">{shortcut.description}</span>
                                    <div class="flex gap-1">
                                        {#each shortcut.keys as key}
                                            <kbd class="px-2 py-1 text-xs font-mono bg-muted border border-border rounded shadow-sm">
                                                {key}
                                            </kbd>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>

            <!-- Footer -->
            <div class="p-4 border-t border-border flex justify-end">
                <button
                        onclick={onClose}
                        class="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
                >
                    Close
                </button>
            </div>
        </div>
    </div>
{/if}
