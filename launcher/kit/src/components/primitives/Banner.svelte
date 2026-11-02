<script lang="ts">
    import Icon from "./Icon.svelte";
    import { BannerRules, type BannerProps } from "./Banner";

    let { tone = "fault", icon, children, action }: BannerProps = $props();

    const name = $derived(icon ?? BannerRules.icon(tone));
</script>

<div class="amml-banner amml-banner--{tone}" role={BannerRules.role(tone)}>
    <span class="amml-banner__icon"><Icon name={name} size={20} /></span>
    <div class="amml-banner__text">{@render children()}</div>
    {#if action}
        <div class="amml-banner__action">{@render action()}</div>
    {/if}
</div>

<style>
    .amml-banner {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 14px 16px;
        background: var(--color-rad-void-2);
        border-left: 3px solid var(--amml-tone);
        border-radius: var(--radius-rad-lg);
    }

    .amml-banner--fault {
        --amml-tone: var(--color-rad-fault);
    }

    .amml-banner--warn {
        --amml-tone: var(--color-rad-warn);
    }

    .amml-banner--brand,
    .amml-banner--info {
        --amml-tone: var(--color-rad-brand-lift);
    }

    .amml-banner--ok {
        --amml-tone: var(--color-rad-ok);
    }

    .amml-banner--muted {
        --amml-tone: var(--color-rad-faint);
    }

    .amml-banner__icon {
        display: inline-flex;
        flex-shrink: 0;
        color: var(--amml-tone);
    }

    .amml-banner__text {
        flex: 1;
        min-width: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 1.5;
    }

    .amml-banner__text :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-banner__text :global(code) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
    }

    .amml-banner__action {
        display: inline-flex;
        flex-shrink: 0;
    }
</style>
