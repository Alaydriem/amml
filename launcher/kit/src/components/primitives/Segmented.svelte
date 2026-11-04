<script lang="ts">
    import { RovingFocus } from "./RovingFocus";
    import type { SegmentedProps } from "./Segmented";

    let { options, value, label, disabled = false, onchange }: SegmentedProps = $props();

    let buttons: HTMLButtonElement[] = $state([]);

    const selected = $derived(options.findIndex((option) => option.id === value));
    const stop = $derived(RovingFocus.stop(selected));

    function select(index: number) {
        if (disabled) return;
        const option = options[index];
        if (!option || option.id === value) return;
        onchange?.(option.id);
    }

    function onkeydown(event: KeyboardEvent, index: number) {
        if (disabled) return;
        const target = RovingFocus.target(event.key, index, options.length);
        if (target === null) return;
        event.preventDefault();
        select(target);
        buttons[target]?.focus();
    }
</script>

<div class="amml-segmented" role="radiogroup" aria-label={label}>
    {#each options as option, index (option.id)}
        <button
            bind:this={buttons[index]}
            class="amml-segmented__option"
            class:amml-segmented__option--on={option.id === value}
            type="button"
            role="radio"
            aria-checked={option.id === value}
            tabindex={index === stop ? 0 : -1}
            {disabled}
            onclick={() => select(index)}
            onkeydown={(event) => onkeydown(event, index)}
        >
            {option.label}
        </button>
    {/each}
</div>

<style>
    .amml-segmented {
        display: inline-flex;
        padding: 3px;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-lg);
    }

    .amml-segmented__option {
        padding: 6px 14px;
        border: 0;
        border-radius: var(--radius-rad-seg);
        background: none;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        cursor: pointer;
    }

    .amml-segmented__option:hover:not(:disabled):not(.amml-segmented__option--on) {
        color: var(--color-rad-tx);
    }

    .amml-segmented__option--on {
        background: var(--color-rad-brand);
        color: var(--color-rad-on-brand);
    }

    .amml-segmented__option:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
