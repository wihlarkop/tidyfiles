<script lang="ts">
    interface Props {
        icon?: string;
        title: string;
        description: string;
        actionText?: string;
        onAction?: () => void;
        actionDisabled?: boolean;
        children?: any;
    }

    let {
        icon = '📋',
        title,
        description,
        actionText,
        onAction,
        actionDisabled = false,
        children
    }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 px-6 text-center">
    <!-- Icon with subtle animation -->
    <div class="text-6xl mb-4 animate-fade-in">
        {icon}
    </div>

    <!-- Title -->
    <h3 class="text-xl font-semibold mb-2 text-foreground">
        {title}
    </h3>

    <!-- Description -->
    <p class="text-sm text-muted-foreground max-w-md mb-6">
        {description}
    </p>

    <!-- Action Button or Custom Content -->
    {#if children}
        {@render children()}
    {:else if actionText && onAction}
        <button
            onclick={onAction}
            disabled={actionDisabled}
            class="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
        >
            {actionText}
        </button>
    {/if}
</div>

<style>
    @keyframes fade-in {
        from {
            opacity: 0;
            transform: scale(0.9);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }

    .animate-fade-in {
        animation: fade-in 0.5s ease-out;
    }
</style>
