<script lang="ts">
    import PanelRow from "../primitives/PanelRow.svelte";
    import Tile from "../primitives/Tile.svelte";
    import type { WorldLocationRowProps } from "./WorldLocationRow";

    let { location, text }: WorldLocationRowProps = $props();
</script>

<PanelRow size="sm">
    {#snippet leading()}<span class="amml-world-location__dot"><Tile size="dot" swatch={location.swatch} /></span>{/snippet}
    {#snippet title()}
        {location.name}
        {#if location.mode === "own" && location.folder}
            <code class="amml-world-location__folder">{location.folder}</code>
        {/if}
    {/snippet}
    {#snippet aside()}
        <span class="amml-world-location__mode amml-world-location__mode--{location.mode}">
            {location.mode === "shared" ? text.shared : text.own}
        </span>
    {/snippet}
</PanelRow>

<style>
    .amml-world-location__folder {
        margin-left: 4px;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-world-location__dot {
        display: flex;
        margin-right: -6px;
    }

    .amml-world-location__mode {
        display: block;
        width: 120px;
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        text-align: right;
    }

    .amml-world-location__mode--shared {
        color: var(--color-rad-brand-lift);
    }

    .amml-world-location__mode--own {
        color: var(--color-rad-dim);
    }
</style>
