<script lang="ts">
    import Checkbox from "../primitives/Checkbox.svelte";
    import type { ChoiceRowProps } from "./ChoiceRow";

    let { id, title, detail, meta, selected = false, stop = true, onselect, onkeydown }: ChoiceRowProps = $props();
</script>

<button
    class="amml-choice-row amml-row"
    class:amml-row--selected={selected}
    type="button"
    role="radio"
    aria-checked={selected}
    tabindex={stop ? 0 : -1}
    onclick={() => onselect?.(id)}
    {onkeydown}
>
    <Checkbox decorative variant="marker" checked={selected} />
    <span class="amml-choice-row__text">
        <span class="amml-choice-row__title">{title}</span>
        {#if detail}
            <span class="amml-choice-row__detail">{detail}</span>
        {/if}
    </span>
    <span class="amml-choice-row__meta">{meta ?? ""}</span>
</button>

<style>
    .amml-choice-row {
        display: flex;
        align-items: center;
        gap: 14px;
        width: 100%;
        padding: 12px 18px;
        border: 0;
        border-bottom: 1px solid var(--color-rad-line);
        background: none;
        color: inherit;
        font: inherit;
        text-align: left;
    }

    .amml-choice-row:last-child {
        border-bottom: 0;
    }

    .amml-choice-row__text {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .amml-choice-row__title {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-choice-row__detail {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-choice-row__meta {
        flex-shrink: 0;
        width: 150px;
        color: var(--color-rad-dim);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        text-align: right;
    }
</style>
