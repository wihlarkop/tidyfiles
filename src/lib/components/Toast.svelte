<script lang="ts">
    import {toastStore, type Toast} from '$lib/stores/toast.svelte';

    let toasts = $derived(toastStore.all);

    function getToastStyles(type: Toast['type']) {
        switch (type) {
            case 'success':
                return 'bg-green-600 text-white border-green-700';
            case 'error':
                return 'bg-red-600 text-white border-red-700';
            case 'warning':
                return 'bg-yellow-600 text-white border-yellow-700';
            case 'info':
            default:
                return 'bg-blue-600 text-white border-blue-700';
        }
    }

    function getIcon(type: Toast['type']) {
        switch (type) {
            case 'success':
                return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>`;
            case 'error':
                return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>`;
            case 'warning':
                return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>`;
            case 'info':
            default:
                return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>`;
        }
    }
</script>

<!-- Toast Container -->
<div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
    {#each toasts as toast (toast.id)}
        <div
                class="pointer-events-auto animate-in slide-in-from-right duration-300
                       flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg border
                       min-w-[300px] max-w-[500px] {getToastStyles(toast.type)}"
        >
            <!-- Icon -->
            <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                {@html getIcon(toast.type)}
            </svg>

            <!-- Message -->
            <div class="flex-1 text-sm font-medium">
                {toast.message}
            </div>

            <!-- Close Button -->
            <button
                    onclick={() => toastStore.remove(toast.id)}
                    class="flex-shrink-0 hover:opacity-75 transition-opacity"
                    aria-label="Close notification"
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
            </button>
        </div>
    {/each}
</div>

<style>
    @keyframes slide-in-from-right {
        from {
            transform: translateX(100%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    .animate-in {
        animation: slide-in-from-right 0.3s ease-out;
    }
</style>
