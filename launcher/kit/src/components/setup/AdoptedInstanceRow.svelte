<script lang="ts">
    import Tile from "../primitives/Tile.svelte";
    import { AdoptedInstanceRowRules, type AdoptedInstanceRowProps } from "./AdoptedInstanceRow";

    let { instance, text }: AdoptedInstanceRowProps = $props();
</script>

<div class="amml-adopted-row">
    <Tile size="dot" swatch={instance.swatch} />
    <span class="amml-adopted-row__name">{instance.name}</span>
    <span class="amml-adopted-row__meta">{AdoptedInstanceRowRules.meta(instance, text)}</span>
</div>

<style>
    .amml-adopted-row {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 12px 18px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-adopted-row:last-child {
        border-bottom: 0;
    }

    .amml-adopted-row__name {
        flex: 1;
        min-width: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-adopted-row__meta {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
