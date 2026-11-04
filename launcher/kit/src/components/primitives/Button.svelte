<script lang="ts">
    import Icon from "./Icon.svelte";
    import type { ButtonProps } from "./Button";

    let {
        variant = "secondary",
        size = "md",
        tone = "brand",
        icon,
        caps: capsProp = false,
        disabled = false,
        block = false,
        type = "button",
        onclick,
        children,
    }: ButtonProps = $props();

    const fixedCaps = $derived(variant === "launch" || variant === "running");
    const caps = $derived(fixedCaps || capsProp);
</script>

<button
    class="amml-btn amml-btn--{variant} amml-btn--{size}"
    class:amml-btn--tone-brand={variant === "link" && tone === "brand"}
    class:amml-btn--tone-fault={variant === "link" && tone === "fault"}
    class:amml-btn--tone-dim={variant === "link" && tone === "dim"}
    class:amml-btn--block={block}
    class:amml-btn--caps={caps}
    class:amml-btn--fixed={fixedCaps}
    {type}
    {disabled}
    onclick={() => onclick?.()}
>
    {#if variant === "running"}
        <span class="amml-btn__dot"></span>
    {:else if icon}
        <Icon name={icon} size={caps ? 12 : 16} weight={2.2} />
    {/if}
    {@render children()}
</button>

<style>
    .amml-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 9px;
        border: 1px solid transparent;
        border-radius: var(--radius-rad-md);
        background: none;
        color: var(--color-rad-tx);
        font-family: var(--font-rad-sans);
        font-size: var(--text-rad-body);
        line-height: 20px;
        white-space: nowrap;
        cursor: pointer;
        flex-shrink: 0;
    }

    .amml-btn--sm {
        padding: 5px 12px;
        font-size: var(--text-rad-body-sm);
        border-radius: var(--radius-rad-sm);
    }

    .amml-btn--md {
        padding: 10px 16px;
    }

    .amml-btn--lg {
        padding: 11px 18px;
    }

    .amml-btn--block {
        width: 100%;
    }

    .amml-btn--primary {
        background: var(--color-rad-brand);
        color: var(--color-rad-on-brand);
    }

    .amml-btn--primary:hover:not(:disabled) {
        background: var(--color-rad-brand-hover);
    }

    .amml-btn--secondary {
        background: var(--color-rad-void-2);
        border-color: var(--color-rad-line-2);
    }

    .amml-btn--secondary:hover:not(:disabled) {
        background: var(--color-rad-panel);
        border-color: var(--color-rad-brand-lift);
    }

    .amml-btn--ghost {
        color: var(--color-rad-faint);
    }

    .amml-btn--ghost:hover:not(:disabled) {
        color: var(--color-rad-tx);
    }

    .amml-btn--danger {
        background: var(--color-rad-fault-tint);
        color: var(--color-rad-fault);
    }

    .amml-btn--danger:hover:not(:disabled) {
        background: var(--color-rad-fault-chip);
    }

    .amml-btn--warn {
        border-color: var(--color-rad-warn);
        color: var(--color-rad-warn);
    }

    .amml-btn--warn:hover:not(:disabled) {
        background: var(--color-rad-warn-tint);
    }

    .amml-btn.amml-btn--link {
        padding: 0;
        border: 0;
        border-radius: var(--radius-rad-xs);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-btn--link:hover:not(:disabled) {
        text-decoration: underline;
    }

    .amml-btn--tone-brand {
        color: var(--color-rad-brand-lift);
    }

    .amml-btn--tone-fault {
        color: var(--color-rad-fault);
    }

    .amml-btn--tone-dim {
        color: var(--color-rad-dim);
    }

    .amml-btn--caps {
        gap: 8px;
        border-radius: var(--radius-rad-xs);
        font-size: var(--text-rad-body-sm);
        font-weight: var(--font-weight-rad-strong);
        letter-spacing: var(--tracking-rad-caps);
        text-transform: uppercase;
        line-height: 18px;
    }

    .amml-btn--caps.amml-btn--sm {
        padding: 11px 20px;
    }

    .amml-btn--caps.amml-btn--md {
        padding: 12px 22px;
    }

    .amml-btn--caps.amml-btn--lg {
        padding: 14px 26px;
        font-size: var(--text-rad-lead);
        line-height: 22px;
    }

    .amml-btn--caps.amml-btn--ghost {
        padding-inline: 10px;
    }

    .amml-btn.amml-btn--fixed.amml-btn--caps {
        padding: 11px 0;
        color: var(--color-rad-on-brand);
        min-width: 116px;
    }

    .amml-btn--launch {
        background: var(--color-rad-brand);
    }

    .amml-btn--launch:hover:not(:disabled) {
        background: var(--color-rad-brand-hover);
    }

    .amml-btn--running {
        border-color: var(--color-rad-ok);
        cursor: default;
    }

    .amml-btn__dot {
        width: 8px;
        height: 8px;
        border-radius: var(--radius-rad-hair);
        background: var(--color-rad-ok);
    }

    .amml-btn:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
