<script lang="ts">
    interface Props {
        progress?: number; // 0-100, or undefined for indeterminate
        message?: string;
        show?: boolean;
    }

    let {progress, message = '', show = true}: Props = $props();

    // Indeterminate if progress is undefined or negative
    let isIndeterminate = $derived(progress === undefined || progress < 0);
</script>

{#if show}
    <div class="w-full bg-secondary rounded-full h-2.5 overflow-hidden">
        {#if isIndeterminate}
            <div class="bg-primary h-2.5 w-full animate-pulse"></div>
        {:else}
            <div
                    class="bg-primary h-2.5 transition-all duration-300 ease-out"
                    style:width="{Math.min(100, Math.max(0, progress ?? 0))}%"
            ></div>
        {/if}
    </div>
    {#if message}
        <div class="text-sm text-muted-foreground mt-2 text-center">
            {message}
        </div>
    {/if}
{/if}
