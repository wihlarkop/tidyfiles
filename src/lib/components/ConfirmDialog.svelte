<script lang="ts">
    interface Props {
        show: boolean;
        title: string;
        message: string;
        confirmText?: string;
        cancelText?: string;
        variant?: 'destructive' | 'primary' | 'warning';
        onConfirm: () => void;
        onCancel: () => void;
    }

    let {
        show = false,
        title,
        message,
        confirmText = 'Confirm',
        cancelText = 'Cancel',
        variant = 'primary',
        onConfirm,
        onCancel
    }: Props = $props();

    let dialogElement = $state<HTMLDivElement | null>(null);
    let cancelButton = $state<HTMLButtonElement | null>(null);
    let confirmButton = $state<HTMLButtonElement | null>(null);

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            onCancel();
            return;
        }

        // Tab focus trapping
        if (e.key === 'Tab' && dialogElement) {
            const focusableElements = dialogElement.querySelectorAll(
                'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
            );
            const firstElement = focusableElements[0] as HTMLElement;
            const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

            if (e.shiftKey && document.activeElement === firstElement) {
                lastElement?.focus();
                e.preventDefault();
            } else if (!e.shiftKey && document.activeElement === lastElement) {
                firstElement?.focus();
                e.preventDefault();
            }
        }
    }

    // Autofocus cancel button when the dialog opens
    $effect(() => {
        if (show && cancelButton) {
            setTimeout(() => cancelButton?.focus(), 100);
        }
    });

    // Icon and color schemes based on variant
    const variantConfig = {
        destructive: {
            bgColor: 'bg-destructive/10',
            iconColor: 'text-destructive',
            buttonClass: 'bg-destructive text-destructive-foreground',
            icon: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
        },
        primary: {
            bgColor: 'bg-primary/10',
            iconColor: 'text-primary',
            buttonClass: 'bg-primary text-primary-foreground',
            icon: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
        },
        warning: {
            bgColor: 'bg-blue-600/10',
            iconColor: 'text-blue-600',
            buttonClass: 'bg-primary text-primary-foreground',
            icon: 'M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6'
        }
    };

    const config = $derived(variantConfig[variant]);
</script>

{#if show}
    <div
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
        onclick={onCancel}
        onkeydown={handleKeydown}
        role="button"
        tabindex="-1"
    >
        <div
            bind:this={dialogElement}
            class="bg-background border border-border rounded-lg shadow-lg max-w-md w-full m-4"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="dialog-title"
            aria-describedby="dialog-description"
            tabindex="-1"
        >
            <div class="p-6 space-y-4">
                <div class="flex items-start gap-3">
                    <div class="w-10 h-10 rounded-full {config.bgColor} flex items-center justify-center flex-shrink-0">
                        <svg class="w-6 h-6 {config.iconColor}" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={config.icon}/>
                        </svg>
                    </div>
                    <div class="flex-1">
                        <h3 id="dialog-title" class="text-lg font-semibold mb-2">{title}</h3>
                        <div id="dialog-description" class="text-sm text-muted-foreground">
                            {@html message}
                        </div>
                    </div>
                </div>

                <div class="flex gap-2 justify-end pt-2">
                    <button
                        bind:this={cancelButton}
                        onclick={onCancel}
                        class="px-4 py-2 border border-border rounded hover:bg-secondary"
                        aria-label="Cancel action"
                    >
                        {cancelText}
                    </button>
                    <button
                        bind:this={confirmButton}
                        onclick={onConfirm}
                        class="px-4 py-2 {config.buttonClass} rounded hover:opacity-90"
                        aria-label="Confirm action"
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
