<script lang="ts">
    import BackLink from "./BackLink.svelte";
    import type { SettingsNavProps } from "./SettingsNav";

    let { text, items, value, onback, onchange }: SettingsNavProps = $props();
</script>

<nav class="amml-settings-nav" aria-label={text.label}>
    <div class="amml-settings-nav__back"><BackLink label={text.back} onclick={onback} /></div>
    {#each items as item (item.id)}
        <button
            class="amml-settings-nav__item amml-row"
            class:amml-row--selected={item.id === value}
            type="button"
            aria-current={item.id === value ? "page" : undefined}
            onclick={() => onchange?.(item.id)}
        >
            {item.label}
        </button>
    {/each}
</nav>

<style>
    .amml-settings-nav {
        display: flex;
        flex-direction: column;
        gap: 4px;
        width: 240px;
        flex-shrink: 0;
        padding: 26px 16px;
        box-sizing: border-box;
        background: var(--color-rad-nav-ground);
        border-right: 1px solid var(--color-rad-line);
    }

    .amml-settings-nav__back {
        padding: 0 12px 18px;
    }

    .amml-settings-nav__item {
        padding: 10px 12px;
        border: 0;
        border-radius: var(--radius-rad-md);
        background: none;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 20px;
        text-align: left;
    }

    /* the selected tab keeps its rule straight along the left edge: only the right corners are round */
    .amml-settings-nav__item.amml-row--selected {
        border-top-left-radius: 0;
        border-bottom-left-radius: 0;
    }

    .amml-settings-nav__item:hover,
    .amml-settings-nav__item.amml-row--selected {
        color: var(--color-rad-tx);
    }
</style>
