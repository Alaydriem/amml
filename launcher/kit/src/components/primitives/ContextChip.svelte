<script lang="ts">
    import type { ContextChipProps } from "./ContextChip";

    let { parts, text }: ContextChipProps = $props();
</script>

<div class="amml-chip">
    {#each parts as part, index (part.id)}
        {#if index > 0}
            <span class="amml-chip__separator" aria-hidden="true">{text.separator}</span>
        {/if}
        <span class="amml-chip__label">{part.label}</span>
        <span class="amml-chip__value amml-chip__value--{part.tone ?? 'dim'}">{part.value}</span>
    {/each}
</div>

<style>
    .amml-chip {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 6px 12px;
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-pill);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        white-space: nowrap;
    }

    .amml-chip__label,
    .amml-chip__separator {
        color: var(--color-rad-faint);
    }

    .amml-chip__value--strong {
        color: var(--color-rad-tx);
    }

    .amml-chip__value--dim {
        color: var(--color-rad-dim);
    }

    .amml-chip__value--faint {
        color: var(--color-rad-faint);
    }

    .amml-chip__value--brand {
        color: var(--color-rad-brand-lift);
    }

    .amml-chip__value--ok {
        color: var(--color-rad-ok);
    }

    .amml-chip__value--warn {
        color: var(--color-rad-warn);
    }

    .amml-chip__value--fault {
        color: var(--color-rad-fault);
    }
</style>
