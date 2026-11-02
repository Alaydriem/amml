<script lang="ts">
    import type { KeyValueRowProps } from "./KeyValueRow";

    let { label, value, tone = "strong", mono = true, onopen }: KeyValueRowProps = $props();
</script>

<div class="amml-kv">
    <span class="amml-kv__label">{label}</span>
    {#if onopen}
        <button class="amml-kv__value amml-kv__value--{tone} amml-kv__button" class:amml-kv__value--mono={mono} type="button" onclick={() => onopen()}>
            {value}
        </button>
    {:else}
        <span class="amml-kv__value amml-kv__value--{tone}" class:amml-kv__value--mono={mono}>{value}</span>
    {/if}
</div>

<style>
    .amml-kv {
        display: flex;
        justify-content: space-between;
        gap: 16px;
        padding-block: 8px;
        border-bottom: 1px solid var(--color-rad-line);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-kv:last-child {
        border-bottom: 0;
    }

    .amml-kv__label {
        color: var(--color-rad-faint);
    }

    .amml-kv__button {
        padding: 0;
        border: 0;
        border-radius: var(--radius-rad-xs);
        background: none;
        font-size: inherit;
        line-height: inherit;
        text-align: right;
        cursor: pointer;
    }

    .amml-kv__button:hover {
        text-decoration: underline;
    }

    .amml-kv__value--mono {
        font-family: var(--font-rad-mono);
    }

    .amml-kv__value--strong {
        color: var(--color-rad-tx);
    }

    .amml-kv__value--dim {
        color: var(--color-rad-dim);
    }

    .amml-kv__value--faint {
        color: var(--color-rad-faint);
    }

    .amml-kv__value--brand {
        color: var(--color-rad-brand-lift);
    }

    .amml-kv__value--ok {
        color: var(--color-rad-ok);
    }

    .amml-kv__value--warn {
        color: var(--color-rad-warn);
    }

    .amml-kv__value--fault {
        color: var(--color-rad-fault);
    }
</style>
