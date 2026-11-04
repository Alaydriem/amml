<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import RichText from "../primitives/RichText.svelte";
    import { SelectionBarRules, type SelectionBarProps } from "./SelectionBar";

    let { selected, text, onclear, oninstall }: SelectionBarProps = $props();

    const empty = $derived(selected.length === 0);
</script>

<div class="amml-selection-bar">
    {#if empty}
        <span class="amml-selection-bar__hint">{text.hint}</span>
        <span class="amml-selection-bar__spacer"></span>
    {:else}
        <span class="amml-selection-bar__summary">
            <RichText parts={SelectionBarRules.summary(selected, text)} />
        </span>
        <Button variant="link" tone="dim" onclick={onclear}>{text.clear}</Button>
        <span class="amml-selection-bar__spacer"></span>
        <span class="amml-selection-bar__size">{SelectionBarRules.totalSize(selected)}</span>
    {/if}
    <Button variant="primary" caps disabled={empty} onclick={oninstall}>
        {SelectionBarRules.installLabel(selected, text)}
    </Button>
</div>

<style>
    .amml-selection-bar {
        display: flex;
        flex-shrink: 0;
        align-items: center;
        gap: 14px;
        height: 64px;
        padding: 0 30px;
        box-sizing: border-box;
        background: var(--color-rad-void-2);
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-selection-bar__summary {
        min-width: 0;
        overflow: hidden;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .amml-selection-bar__summary :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-selection-bar__hint {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-selection-bar__spacer {
        flex: 1 1 0;
    }

    .amml-selection-bar__size {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
