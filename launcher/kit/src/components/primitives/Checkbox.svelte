<script lang="ts">
    import Icon from "./Icon.svelte";
    import type { CheckboxProps } from "./Checkbox";

    let { checked = false, label, variant = "check", decorative = false, disabled = false, onchange }: CheckboxProps = $props();
</script>

{#snippet face()}
    {#if checked && variant === "check"}
        <Icon name="check" size={12} weight={3} />
    {:else if variant === "marker"}
        <span class="amml-checkbox__marker"></span>
    {/if}
{/snippet}

{#if decorative}
    <span class="amml-checkbox amml-checkbox--{variant}" class:amml-checkbox--on={checked} aria-hidden="true">
        {@render face()}
    </span>
{:else}
    <button
        class="amml-checkbox amml-checkbox--{variant}"
        class:amml-checkbox--on={checked}
        type="button"
        role="checkbox"
        aria-checked={checked}
        aria-label={label}
        {disabled}
        onclick={() => {
            if (!disabled) onchange?.(!checked);
        }}
    >
        {@render face()}
    </button>
{/if}

<style>
    .amml-checkbox {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 18px;
        height: 18px;
        flex-shrink: 0;
        padding: 0;
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-box);
        background: none;
        color: var(--color-rad-on-brand);
    }

    button.amml-checkbox {
        cursor: pointer;
    }

    .amml-checkbox--check.amml-checkbox--on {
        background: var(--color-rad-brand);
        border-color: var(--color-rad-brand-lift);
    }

    .amml-checkbox--marker.amml-checkbox--on {
        border-color: var(--color-rad-brand-lift);
    }

    .amml-checkbox__marker {
        width: 10px;
        height: 10px;
        border-radius: var(--radius-rad-2xs);
        background: var(--color-rad-brand-lift);
        opacity: 0;
    }

    .amml-checkbox--on .amml-checkbox__marker {
        opacity: 1;
    }

    button.amml-checkbox:hover:not(:disabled) {
        border-color: var(--color-rad-brand-lift);
    }

    .amml-checkbox:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
