<script lang="ts">
    interface Props {
        class?: string;
        variant?: 'text' | 'circular' | 'rectangular';
        width?: string;
        height?: string;
        count?: number;
    }

    let {
        class: className = '',
        variant = 'rectangular',
        width = '100%',
        height = variant === 'text' ? '1rem' : '3rem',
        count = 1
    }: Props = $props();

    function getVariantClasses() {
        switch (variant) {
            case 'text':
                return 'rounded';
            case 'circular':
                return 'rounded-full';
            case 'rectangular':
            default:
                return 'rounded-md';
        }
    }
</script>

{#each Array(count) as _, i}
    <div
        class="animate-pulse bg-muted {getVariantClasses()} {className}"
        style="width: {width}; height: {height};"
        role="status"
        aria-label="Loading..."
    ></div>
{/each}

<style>
    @keyframes pulse {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }

    .animate-pulse {
        animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    }
</style>
