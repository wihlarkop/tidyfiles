<script lang="ts">
    import Skeleton from './Skeleton.svelte';

    interface Props {
        rows?: number;
        columns?: number;
    }

    let { rows = 5, columns = 3 }: Props = $props();
</script>

<div class="border border-border rounded-lg overflow-hidden">
    <table class="w-full">
        <thead class="bg-muted">
            <tr>
                {#each Array(columns) as _, i}
                    <th class="px-4 py-2 text-left">
                        <Skeleton height="1rem" width={i === 0 ? '60%' : '80%'} />
                    </th>
                {/each}
            </tr>
        </thead>
        <tbody>
            {#each Array(rows) as _, rowIndex}
                <tr class="border-t border-border">
                    {#each Array(columns) as _, colIndex}
                        <td class="px-4 py-2">
                            <Skeleton
                                height="0.875rem"
                                width={colIndex === 0 ? '70%' : (colIndex === columns - 1 ? '40%' : '85%')}
                            />
                        </td>
                    {/each}
                </tr>
            {/each}
        </tbody>
    </table>
</div>
