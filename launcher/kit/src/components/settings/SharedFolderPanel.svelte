<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import Panel from "../primitives/Panel.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import Stat from "../primitives/Stat.svelte";
    import type { SharedFolderPanelProps } from "./SharedFolderPanel";

    let { text, path, stats, onopen }: SharedFolderPanelProps = $props();
</script>

<Panel heading={text.heading}>
    <PanelRow>
        {#snippet title()}<span class="amml-shared__path">{path}</span>{/snippet}
        {#snippet detail()}{text.note}{/snippet}
        {#snippet aside()}<Button variant="secondary" onclick={() => onopen?.()}>{text.open}</Button>{/snippet}
    </PanelRow>
    <div class="amml-shared__stats">
        {#each stats as stat (stat.id)}
            <!-- Stat trims the space before its unit, so the unit carries it. -->
            <Stat label={text.stats[stat.id]} value={stat.value} unit={stat.unit === undefined ? undefined : ` ${stat.unit}`} />
        {/each}
    </div>
</Panel>

<style>
    .amml-shared__path {
        display: block;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-shared__stats {
        display: flex;
        padding: 14px 18px;
    }
</style>
