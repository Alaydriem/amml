<script lang="ts">
    import { BrandMarkRules, type BrandMarkProps } from "./BrandMark";

    let { size = "sm", label }: BrandMarkProps = $props();

    const spec = $derived(BrandMarkRules.SIZES[size]);
</script>

<svg
    class="amml-brand-mark"
    width={spec.width}
    height={spec.height}
    viewBox="0 0 26 30"
    role={label ? "img" : undefined}
    aria-label={label}
    aria-hidden={label ? undefined : "true"}
>
    {#each BrandMarkRules.SEGMENTS as segment (segment.d)}
        <path d={segment.d} fill="none" stroke={segment.color} stroke-width={spec.stroke} />
    {/each}
</svg>

<style>
    .amml-brand-mark {
        display: inline-block;
        flex-shrink: 0;
    }
</style>
