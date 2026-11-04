<script lang="ts">
    import Panel from "../primitives/Panel.svelte";
    import type { DownloadUsageProps } from "./DownloadUsage";

    let { text, path, value, unit }: DownloadUsageProps = $props();
</script>

<Panel heading={text.heading}>
    {#snippet aside()}<span class="amml-usage__path">{path}</span>{/snippet}
    <div class="amml-usage__total">
        {value} <span class="amml-usage__unit">{unit}</span>
    </div>
</Panel>

<style>
    .amml-usage__path {
        display: block;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-usage__total {
        padding: 16px 18px;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title);
        font-weight: var(--font-weight-rad-light);
        line-height: 30px;
    }

    .amml-usage__unit {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }
</style>
