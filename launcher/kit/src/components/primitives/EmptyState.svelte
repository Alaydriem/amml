<script lang="ts">
    import BrandMark from "./BrandMark.svelte";
    import Icon from "./Icon.svelte";
    import type { EmptyStateProps } from "./EmptyState";

    let { icon, title, body, actions }: EmptyStateProps = $props();
</script>

<div class="amml-empty">
    <span class="amml-empty__mark">
        {#if icon}
            <Icon name={icon} size={28} />
        {:else}
            <BrandMark size="md" />
        {/if}
    </span>
    <h2 class="amml-empty__title">{title}</h2>
    {#if body}
        <div class="amml-empty__body">{@render body()}</div>
    {/if}
    {#if actions}
        <div class="amml-empty__actions">{@render actions()}</div>
    {/if}
</div>

<style>
    .amml-empty {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 18px;
        padding: 72px 4px 0;
    }

    .amml-empty__mark {
        display: inline-flex;
        color: var(--color-rad-faint);
    }

    .amml-empty__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title);
        font-weight: var(--font-weight-rad-strong);
        line-height: 30px;
        letter-spacing: var(--tracking-rad-title);
    }

    .amml-empty__body {
        max-width: 560px;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 1.5;
    }

    .amml-empty__body :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-empty__actions {
        display: flex;
        gap: 10px;
    }
</style>
