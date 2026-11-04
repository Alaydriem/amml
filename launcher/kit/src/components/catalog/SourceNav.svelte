<script lang="ts">
    import { Units } from "../../text/Units";
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import Icon from "../primitives/Icon.svelte";
    import Toggle from "../primitives/Toggle.svelte";
    import { SourceNavRules, type SourceNavProps } from "./SourceNav";

    let {
        registries,
        allCount,
        value = SourceNavRules.ALL,
        showOtherAmml = false,
        text,
        onchange,
        onshowotheramml,
    }: SourceNavProps = $props();

    const uid = $props.id();
</script>

<nav class="amml-source-nav" aria-label={text.heading}>
    <div class="amml-source-nav__heading"><Eyebrow size="sm">{text.heading}</Eyebrow></div>
    <button
        class="amml-source-nav__item amml-row"
        class:amml-row--selected={value === SourceNavRules.ALL}
        type="button"
        aria-current={value === SourceNavRules.ALL ? "true" : undefined}
        onclick={() => onchange?.(SourceNavRules.ALL)}
    >
        <span class="amml-source-nav__name">{text.allRegistries}</span>
        {#if allCount !== undefined}<span class="amml-source-nav__count">{Units.en.count(allCount)}</span>{/if}
    </button>
    {#each registries as registry (registry.id)}
        {@const trailing = SourceNavRules.trailing(registry, text)}
        <button
            class="amml-source-nav__item amml-row"
            class:amml-row--selected={value === registry.id}
            class:amml-source-nav__item--down={trailing.fault}
            type="button"
            aria-current={value === registry.id ? "true" : undefined}
            onclick={() => onchange?.(registry.id)}
        >
            <span class="amml-source-nav__name">{registry.name}</span>
            <span class="amml-source-nav__count" class:amml-source-nav__count--fault={trailing.fault}>{trailing.text}</span>
        </button>
    {/each}
    <div class="amml-source-nav__rule"></div>
    <button
        class="amml-source-nav__item amml-source-nav__item--url amml-row"
        class:amml-row--selected={value === SourceNavRules.URL}
        type="button"
        aria-current={value === SourceNavRules.URL ? "true" : undefined}
        onclick={() => onchange?.(SourceNavRules.URL)}
    >
        <Icon name="link" size={15} weight={2} />
        <span class="amml-source-nav__name">{text.fromUrl}</span>
    </button>
    <div class="amml-source-nav__spacer"></div>
    {#if onshowotheramml}
        <div class="amml-source-nav__foot">
            <Toggle
                size="sm"
                label={text.needOtherAmml}
                labelledby="{uid}-other"
                checked={showOtherAmml}
                onchange={(next) => onshowotheramml?.(next)}
            />
            <span class="amml-source-nav__foot-text" id="{uid}-other">{text.needOtherAmml}</span>
        </div>
    {/if}
</nav>

<style>
    .amml-source-nav {
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        gap: 2px;
        width: 230px;
        padding: 16px 12px;
        box-sizing: border-box;
        background: var(--color-rad-nav-ground);
        border-right: 1px solid var(--color-rad-line);
    }

    .amml-source-nav__heading {
        padding: 0 10px 8px;
    }

    .amml-source-nav__item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        padding: 9px 10px;
        border: 0;
        border-radius: var(--radius-rad-md);
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        text-align: left;
    }

    .amml-source-nav__item:not(.amml-row--selected):not(:hover) {
        background: none;
    }

    .amml-source-nav__item.amml-row--selected {
        color: var(--color-rad-tx);
    }

    .amml-source-nav__item--down {
        color: var(--color-rad-faint);
    }

    .amml-source-nav__item--url {
        justify-content: flex-start;
    }

    .amml-source-nav__count {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-row--selected .amml-source-nav__count {
        color: var(--color-rad-dim);
    }

    .amml-source-nav__count--fault,
    .amml-row--selected .amml-source-nav__count--fault {
        color: var(--color-rad-fault);
        font-size: var(--text-rad-label);
    }

    .amml-source-nav__rule {
        height: 1px;
        margin: 10px;
        flex-shrink: 0;
        background: var(--color-rad-line);
    }

    .amml-source-nav__spacer {
        flex: 1 1 0;
    }

    .amml-source-nav__foot {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px;
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-source-nav__foot-text {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 135%;
    }
</style>
