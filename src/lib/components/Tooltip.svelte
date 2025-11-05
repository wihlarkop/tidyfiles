<script lang="ts">
    interface Props {
        text: string;
        position?: 'top' | 'bottom' | 'left' | 'right';
        delay?: number;
        children: any;
    }

    let { text, position = 'top', delay = 500, children }: Props = $props();

    let showTooltip = $state(false);
    let timeoutId: number | null = null;
    let tooltipElement = $state<HTMLDivElement | null>(null);
    let wrapperElement = $state<HTMLDivElement | null>(null);
    let adjustedPosition = $state(position);

    function handleMouseEnter() {
        timeoutId = window.setTimeout(() => {
            showTooltip = true;
        }, delay);
    }

    function handleMouseLeave() {
        if (timeoutId) {
            clearTimeout(timeoutId);
            timeoutId = null;
        }
        showTooltip = false;
    }

    // Smart positioning: adjust if the tooltip goes off-screen
    $effect(() => {
        if (showTooltip && tooltipElement && wrapperElement) {
            const tooltipRect = tooltipElement.getBoundingClientRect();
            const wrapperRect = wrapperElement.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const viewportHeight = window.innerHeight;

            let newPosition = position;

            // Check if the tooltip goes off-screen and adjust
            switch (position) {
                case 'top':
                    if (tooltipRect.top < 0) {
                        newPosition = 'bottom'; // Flip to bottom if top is cut off
                    }
                    break;
                case 'bottom':
                    if (tooltipRect.bottom > viewportHeight) {
                        newPosition = 'top'; // Flip to top if bottom is cut off
                    }
                    break;
                case 'left':
                    if (tooltipRect.left < 0) {
                        newPosition = 'right'; // Flip to right if left is cut off
                    }
                    break;
                case 'right':
                    if (tooltipRect.right > viewportWidth) {
                        newPosition = 'left'; // Flip to left if right is cut off
                    }
                    break;
            }

            // Check horizontal overflow for top/bottom positions
            if (newPosition === 'top' || newPosition === 'bottom') {
                if (tooltipRect.left < 0 || tooltipRect.right > viewportWidth) {
                    // If horizontal overflow, try the left / right position instead
                    const spaceOnRight = viewportWidth - wrapperRect.right;
                    const spaceOnLeft = wrapperRect.left;
                    newPosition = spaceOnRight > spaceOnLeft ? 'right' : 'left';
                }
            }

            adjustedPosition = newPosition;
        } else {
            adjustedPosition = position;
        }
    });

    function getPositionClasses() {
        switch (adjustedPosition) {
            case 'top':
                return 'bottom-full left-1/2 -translate-x-1/2 mb-2';
            case 'bottom':
                return 'top-full left-1/2 -translate-x-1/2 mt-2';
            case 'left':
                return 'right-full top-1/2 -translate-y-1/2 mr-2';
            case 'right':
                return 'left-full top-1/2 -translate-y-1/2 ml-2';
            default:
                return 'bottom-full left-1/2 -translate-x-1/2 mb-2';
        }
    }
</script>

<div
    class="relative inline-block"
    bind:this={wrapperElement}
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    role="group"
>
    {@render children()}

    {#if showTooltip && text}
        <div
            bind:this={tooltipElement}
            class="absolute z-50 pointer-events-none {getPositionClasses()}"
            role="tooltip"
        >
            <div class="bg-gray-900 dark:bg-gray-700 text-white text-xs rounded px-3 py-2 shadow-lg max-w-xs whitespace-normal">
                {text}
            </div>
        </div>
    {/if}
</div>

<style>
    /* Ensure tooltip appears above most elements */
    div[role="tooltip"] {
        z-index: 9999;
    }
</style>
