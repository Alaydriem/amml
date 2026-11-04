<script lang="ts">
    import { tick } from "svelte";
    import Icon from "./Icon.svelte";
    import type { SelectProps } from "./Select";

    let { options, value, label, compact = false, disabled = false, onchange }: SelectProps = $props();

    let control: HTMLSelectElement | undefined = $state();

    async function picked(event: Event & { currentTarget: HTMLSelectElement }) {
        onchange?.(event.currentTarget.value);
        // The control shows the prop: if the parent keeps the old value, the pick is taken back.
        await tick();
        if (control && value !== undefined && control.value !== value) control.value = value;
    }
</script>

<div class="amml-select" class:amml-select--compact={compact} class:amml-select--disabled={disabled}>
    <select bind:this={control} class="amml-select__control" aria-label={label} {disabled} {value} onchange={picked}>
        {#each options as option (option.id)}
            <option value={option.id}>{option.label}</option>
        {/each}
    </select>
    <span class="amml-select__chevron"><Icon name="chevron-down" size={14} weight={2.2} /></span>
</div>

<style>
    .amml-select {
        position: relative;
        display: inline-flex;
        min-width: 150px;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-md);
        color: var(--color-rad-faint);
    }

    .amml-select--compact {
        min-width: 0;
    }

    .amml-select:has(:focus-visible) {
        border-color: var(--color-rad-brand-lift);
        outline: var(--rad-focus-ring);
        outline-offset: var(--rad-focus-offset);
    }

    .amml-select__control {
        flex: 1;
        padding: 9px 36px 9px 12px;
        border: 0;
        background: none;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body);
        line-height: 20px;
        appearance: none;
        cursor: pointer;
    }

    .amml-select__control:focus-visible {
        outline: none;
    }

    .amml-select__control option {
        background: var(--color-rad-void-2);
        color: var(--color-rad-tx);
    }

    .amml-select__chevron {
        position: absolute;
        top: 50%;
        right: 12px;
        display: inline-flex;
        transform: translateY(-50%);
        pointer-events: none;
    }

    .amml-select--disabled {
        opacity: var(--opacity-rad-disabled);
    }
</style>
