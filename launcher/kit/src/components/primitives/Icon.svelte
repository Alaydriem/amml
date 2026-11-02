<script lang="ts">
    import { ICONS, type IconProps, type IconShape } from "./Icon";

    let { name, size = 18, weight = 2, label }: IconProps = $props();

    const shape: IconShape = $derived(ICONS[name]);
</script>

<svg
    class="amml-icon"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    role={label ? "img" : undefined}
    aria-label={label}
    aria-hidden={label ? undefined : "true"}
>
    {#each shape.strokes ?? [] as d (d)}
        <path {d} fill="none" stroke="currentColor" stroke-width={weight} stroke-linecap="round" stroke-linejoin="round" />
    {/each}
    {#each shape.circles ?? [] as [cx, cy, r] (`${cx},${cy},${r}`)}
        <circle {cx} {cy} {r} fill="none" stroke="currentColor" stroke-width={weight} />
    {/each}
    {#each shape.fills ?? [] as d (d)}
        <path {d} fill="currentColor" />
    {/each}
    {#each shape.dots ?? [] as [cx, cy, r] (`${cx},${cy},${r}`)}
        <circle {cx} {cy} {r} fill="currentColor" />
    {/each}
</svg>

<style>
    .amml-icon {
        display: inline-block;
        flex-shrink: 0;
    }
</style>
