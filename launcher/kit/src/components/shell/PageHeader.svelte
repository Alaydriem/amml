<script lang="ts">
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import type { PageHeaderProps } from "./PageHeader";

    let { eyebrow, size = "page", title, actions }: PageHeaderProps = $props();
</script>

<div class="amml-page-header">
    <div class="amml-page-header__text">
        {#if eyebrow}
            <Eyebrow>{eyebrow}</Eyebrow>
        {/if}
        <h1 class="amml-page-header__title amml-page-header__title--{size}">{@render title()}</h1>
    </div>
    {#if actions}
        <div class="amml-page-header__actions">{@render actions()}</div>
    {/if}
</div>

<style>
    .amml-page-header {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 16px;
    }

    .amml-page-header__text {
        display: flex;
        flex-direction: column;
        gap: 10px;
        min-width: 0;
    }

    .amml-page-header__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-light);
    }

    .amml-page-header__title--page {
        font-size: var(--text-rad-page);
        line-height: 1.14;
        letter-spacing: var(--tracking-rad-page);
    }

    .amml-page-header__title--section {
        font-size: var(--text-rad-title);
        line-height: 30px;
        letter-spacing: var(--tracking-rad-heading);
    }

    .amml-page-header__title :global(strong) {
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-page-header__actions {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-shrink: 0;
    }
</style>
