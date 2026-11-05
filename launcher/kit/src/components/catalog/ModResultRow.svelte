<script lang="ts">
    import { Template } from "../../text/Template";
    import Checkbox from "../primitives/Checkbox.svelte";
    import Pill from "../primitives/Pill.svelte";
    import Tile from "../primitives/Tile.svelte";
    import { ModResultRowRules, type ModResultRowProps } from "./ModResultRow";
    import { ResultPaging } from "./ResultPaging";

    let { card, checked = false, active = false, anchor = false, text, onchange, onactivate }: ModResultRowProps = $props();

    const selectable = $derived(ModResultRowRules.selectable(card.state));
</script>

<li
    class="amml-result amml-row"
    class:amml-row--selected={active}
    class:amml-result--blocked={card.blockedNote !== undefined}
    style:height="{ResultPaging.ROW_HEIGHT}px"
    data-anchor={anchor ? "" : undefined}
>
    <span class="amml-result__check">
        <Checkbox
            {checked}
            disabled={!selectable}
            label={Template.fill(text.select, { title: card.title })}
            onchange={(next) => onchange?.(card.id, next)}
        />
    </span>
    <Tile size="row" swatch={card.swatch} src={card.icon} />
    <button
        class="amml-result__main"
        type="button"
        aria-current={active ? "true" : undefined}
        onclick={() => onactivate?.(card.id)}
    >
        <span class="amml-result__line">
            <span class="amml-result__title">{card.title}</span>
            <span class="amml-result__id">{card.id}</span>
        </span>
        {#if card.blockedNote !== undefined}
            <span class="amml-result__summary amml-result__note">{card.blockedNote}</span>
        {:else}
            <span class="amml-result__summary">
                {card.summary}
                {#if card.laterNote !== undefined}
                    <span class="amml-result__note">{card.laterNote}</span>
                {/if}
            </span>
        {/if}
    </button>
    <span class="amml-result__version">{card.version}</span>
    <span class="amml-result__state">
        {#if card.state === "installed"}
            <span class="amml-result__installed">{text.installed}</span>
        {:else if card.state === "update"}
            <Pill size="sm">{#snippet children()}{text.update}{/snippet}</Pill>
        {/if}
    </span>
</li>

<style>
    .amml-result {
        position: relative;
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 0 26px 0 20px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-result:has(.amml-result__main:focus-visible) {
        outline: var(--rad-focus-ring);
        outline-offset: calc(var(--rad-focus-offset) * -1);
    }

    .amml-result__check {
        position: relative;
        z-index: 1;
        display: inline-flex;
        flex-shrink: 0;
    }

    .amml-result__main {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 1px;
        min-width: 0;
        padding: 0;
        border: 0;
        background: none;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }

    .amml-result__main:focus-visible {
        outline: none;
    }

    /* The button covers the whole row, so a press anywhere on it opens the mod. */
    .amml-result__main::after {
        content: "";
        position: absolute;
        inset: 0;
    }

    .amml-result__line,
    .amml-result__summary {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .amml-result__title {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body-sm);
        font-weight: var(--font-weight-rad-strong);
        line-height: 18px;
    }

    .amml-result__id {
        margin-left: 4px;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        line-height: 16px;
    }

    .amml-result__summary {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-result__note {
        color: var(--color-rad-dim);
    }

    .amml-result--blocked .amml-result__title,
    .amml-result--blocked .amml-result__id,
    .amml-result--blocked .amml-result__version {
        opacity: var(--opacity-rad-disabled);
    }

    .amml-result__version {
        flex-shrink: 0;
        width: 72px;
        color: var(--color-rad-dim);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        text-align: right;
    }

    .amml-result__state {
        display: flex;
        flex-shrink: 0;
        justify-content: flex-end;
        width: 84px;
    }

    .amml-result__installed {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
