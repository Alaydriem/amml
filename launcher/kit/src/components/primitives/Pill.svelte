<script lang="ts">
    import type { PillProps } from "./Pill";

    let { tone = "brand", dot = false, tint = false, size = "md", onclick, children }: PillProps = $props();

    const resolved = $derived(tone === "info" ? "brand" : tone);
</script>

{#if onclick}
    <button class="amml-pill amml-pill--{resolved} amml-pill--{size}" class:amml-pill--tint={tint} type="button" onclick={() => onclick()}>
        {#if dot}<span class="amml-pill__dot"></span>{/if}
        {@render children()}
    </button>
{:else}
    <span class="amml-pill amml-pill--{resolved} amml-pill--{size}" class:amml-pill--tint={tint}>
        {#if dot}<span class="amml-pill__dot"></span>{/if}
        {@render children()}
    </span>
{/if}

<style>
    .amml-pill {
        display: inline-flex;
        align-items: center;
        gap: 7px;
        padding: 4px 10px;
        border: 1px solid var(--amml-pill-edge);
        border-radius: var(--radius-rad-pill);
        background: none;
        color: var(--amml-pill-color);
        font-size: var(--text-rad-small);
        line-height: 18px;
        white-space: nowrap;
    }

    button.amml-pill {
        cursor: pointer;
    }

    button.amml-pill:hover {
        background: var(--amml-pill-tint);
    }

    .amml-pill--sm {
        padding: 3px 9px;
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-pill--tint {
        padding: 5px 11px;
        background: var(--amml-pill-tint);
    }

    .amml-pill__dot {
        width: 7px;
        height: 7px;
        flex-shrink: 0;
        border-radius: var(--radius-rad-hair);
        background: var(--amml-pill-color);
    }

    .amml-pill--brand {
        --amml-pill-color: var(--color-rad-brand-lift);
        --amml-pill-edge: var(--color-rad-brand-edge);
        --amml-pill-tint: var(--color-rad-brand-tint);
    }

    .amml-pill--ok {
        --amml-pill-color: var(--color-rad-ok);
        --amml-pill-edge: var(--color-rad-ok-edge);
        --amml-pill-tint: var(--color-rad-ok-tint);
    }

    .amml-pill--warn {
        --amml-pill-color: var(--color-rad-warn);
        --amml-pill-edge: var(--color-rad-warn-edge);
        --amml-pill-tint: var(--color-rad-warn-tint);
    }

    .amml-pill--fault {
        --amml-pill-color: var(--color-rad-fault);
        --amml-pill-edge: var(--color-rad-fault-edge);
        --amml-pill-tint: var(--color-rad-fault-tint);
    }

    .amml-pill--muted {
        --amml-pill-color: var(--color-rad-faint);
        --amml-pill-edge: var(--color-rad-line-2);
        --amml-pill-tint: var(--color-rad-void-2);
    }
</style>
