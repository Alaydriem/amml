<script lang="ts">
    import Panel from "../primitives/Panel.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import ProgressBar from "../primitives/ProgressBar.svelte";
    import Select from "../primitives/Select.svelte";
    import { DownloadUsageRules, type DownloadUsageProps } from "./DownloadUsage";

    let { text, path, usedText, percent, limitOptions, limit, onlimitchange }: DownloadUsageProps = $props();
</script>

<Panel heading={text.heading}>
    {#snippet aside()}<span class="amml-usage__path">{path}</span>{/snippet}
    <div class="amml-usage__meter">
        <div class="amml-usage__figures">
            <div class="amml-usage__used">
                {usedText} <span class="amml-usage__of">{DownloadUsageRules.ofText(limit, limitOptions, text)}</span>
            </div>
            <div class="amml-usage__note">{text.note}</div>
        </div>
        <ProgressBar value={percent} label={text.barLabel} />
    </div>
    <PanelRow>
        {#snippet title()}{text.limitTitle}{/snippet}
        {#snippet detail()}{text.limitNote}{/snippet}
        {#snippet aside()}
            <Select options={limitOptions} value={limit} label={text.limitLabel} onchange={(id) => onlimitchange?.(id)} />
        {/snippet}
    </PanelRow>
</Panel>

<style>
    .amml-usage__path {
        display: block;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-usage__meter {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding: 16px 18px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-usage__figures {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 16px;
    }

    .amml-usage__used {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title);
        font-weight: var(--font-weight-rad-light);
        line-height: 30px;
    }

    .amml-usage__of {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-usage__note {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }
</style>
