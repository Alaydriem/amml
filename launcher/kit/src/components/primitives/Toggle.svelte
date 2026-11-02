<script lang="ts">
    import type { ToggleProps } from "./Toggle";

    let { checked = false, label, size = "md", disabled = false, id, labelledby, onchange }: ToggleProps = $props();
</script>

<button
    {id}
    class="amml-toggle amml-toggle--{size}"
    class:amml-toggle--on={checked}
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={labelledby ? undefined : label}
    aria-labelledby={labelledby}
    {disabled}
    onclick={() => {
        if (!disabled) onchange?.(!checked);
    }}
>
    <span class="amml-toggle__thumb"></span>
</button>

<style>
    .amml-toggle {
        position: relative;
        flex-shrink: 0;
        padding: 0;
        border: 1px solid var(--color-rad-line-2);
        background: var(--color-rad-void);
        cursor: pointer;
    }

    .amml-toggle--md {
        width: 48px;
        height: 28px;
        border-radius: var(--radius-rad-md);
        --amml-thumb: 20px;
        --amml-thumb-radius: var(--radius-rad-sm);
        --amml-thumb-travel: 20px;
    }

    .amml-toggle--sm {
        width: 40px;
        height: 24px;
        border-radius: var(--radius-rad-toggle);
        --amml-thumb: 16px;
        --amml-thumb-radius: var(--radius-rad-box);
        --amml-thumb-travel: 16px;
    }

    .amml-toggle__thumb {
        position: absolute;
        top: 3px;
        left: 3px;
        width: var(--amml-thumb);
        height: var(--amml-thumb);
        border-radius: var(--amml-thumb-radius);
        background: var(--color-rad-faint);
        transition: transform 120ms ease;
    }

    .amml-toggle--on {
        background: var(--color-rad-brand);
        border-color: var(--color-rad-brand-lift);
    }

    .amml-toggle--on .amml-toggle__thumb {
        background: var(--color-rad-on-brand);
        transform: translateX(var(--amml-thumb-travel));
    }

    .amml-toggle:hover:not(:disabled):not(.amml-toggle--on) {
        border-color: var(--color-rad-brand-lift);
    }

    .amml-toggle:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
