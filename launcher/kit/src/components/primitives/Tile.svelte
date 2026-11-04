<script lang="ts">
    import Icon from "./Icon.svelte";
    import { TileRules, type TileProps } from "./Tile";

    let { size = "rail", swatch = 3, icon, src, label }: TileProps = $props();

    const spec = $derived(TileRules.SIZES[size]);
    const color = $derived(TileRules.swatchColor(swatch));
</script>

{#snippet face()}
    {#if src}
        <img class="amml-tile__image" {src} alt={label ?? ""} width={spec.inner * 1.5} height={spec.inner * 1.5} />
    {:else if icon}
        <Icon name={icon} size={spec.inner} {label} />
    {:else}
        <span
            class="amml-tile__swatch"
            style:width="{spec.inner}px"
            style:height="{spec.inner}px"
            style:background={color}
        ></span>
    {/if}
{/snippet}

{#if size === "dot"}
    <span class="amml-tile amml-tile--dot">{@render face()}</span>
{:else}
    <span class="amml-tile amml-tile--frame amml-tile--{size}" style:width="{spec.outer}px" style:height="{spec.outer}px">
        {@render face()}
    </span>
{/if}

<style>
    .amml-tile {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        color: var(--color-rad-dim);
    }

    .amml-tile--frame {
        background: var(--color-rad-void-2);
        border: 1px solid var(--color-rad-line);
        box-sizing: border-box;
    }

    .amml-tile--row {
        border-radius: var(--radius-rad-seg);
    }

    .amml-tile--detail {
        background: var(--color-rad-void);
        border-radius: var(--radius-rad-card);
    }

    .amml-tile--rail,
    .amml-tile--header {
        border-radius: var(--radius-rad-rail);
    }

    .amml-tile__swatch {
        flex-shrink: 0;
    }

    .amml-tile--row .amml-tile__swatch {
        border-radius: var(--radius-rad-2xs);
    }

    .amml-tile--detail .amml-tile__swatch,
    .amml-tile--dot .amml-tile__swatch {
        border-radius: var(--radius-rad-swatch);
    }

    .amml-tile--rail .amml-tile__swatch,
    .amml-tile--header .amml-tile__swatch {
        border-radius: var(--radius-rad-xs);
    }

    .amml-tile__image {
        object-fit: contain;
    }
</style>
