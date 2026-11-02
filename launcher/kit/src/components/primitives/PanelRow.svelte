<script lang="ts">
    import type { PanelRowProps } from "./PanelRow";

    let { size = "md", leading, title, detail, aside }: PanelRowProps = $props();
</script>

<div class="amml-panel-row amml-panel-row--{size}">
    {#if leading}
        <div class="amml-panel-row__leading">{@render leading()}</div>
    {/if}
    <div class="amml-panel-row__body">
        <div class="amml-panel-row__title">{@render title()}</div>
        {#if detail}
            <div class="amml-panel-row__detail">{@render detail()}</div>
        {/if}
    </div>
    {#if aside}
        <div class="amml-panel-row__aside">{@render aside()}</div>
    {/if}
</div>

<style>
    .amml-panel-row {
        display: flex;
        align-items: center;
        gap: 22px;
        padding: 14px 18px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-panel-row:last-child {
        border-bottom: 0;
    }

    .amml-panel-row--sm {
        padding-block: 11px;
    }

    .amml-panel-row__leading {
        display: inline-flex;
        flex-shrink: 0;
    }

    .amml-panel-row__body {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 3px;
        min-width: 0;
    }

    .amml-panel-row__title {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-lead);
        line-height: 22px;
    }

    .amml-panel-row--sm .amml-panel-row__title {
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-panel-row__detail {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-panel-row__detail :global(code) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-panel-row__aside {
        display: inline-flex;
        flex-shrink: 0;
        align-items: center;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
    }
</style>
