<script lang="ts">
    import { tick } from "svelte";
    import type { TextFieldProps } from "./TextField";

    let {
        value = "",
        label,
        placeholder,
        mono = false,
        disabled = false,
        readonly = false,
        id,
        labelledby,
        oninput,
        onsubmit,
    }: TextFieldProps = $props();

    let input: HTMLInputElement | undefined = $state();

    async function typed(event: Event & { currentTarget: HTMLInputElement }) {
        oninput?.(event.currentTarget.value);
        // The field shows the prop: if the parent keeps the old text, the typed text is taken back.
        await tick();
        if (input && input.value !== value) input.value = value;
    }
</script>

<div class="amml-field" class:amml-field--mono={mono} class:amml-field--disabled={disabled}>
    <input
        bind:this={input}
        {id}
        class="amml-field__input"
        type="text"
        aria-label={labelledby ? undefined : label}
        aria-labelledby={labelledby}
        {placeholder}
        {disabled}
        {readonly}
        {value}
        spellcheck="false"
        oninput={typed}
        onkeydown={(event) => {
            if (event.key === "Enter") onsubmit?.(event.currentTarget.value);
        }}
    />
</div>

<style>
    .amml-field {
        display: flex;
        align-items: center;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-md);
    }

    .amml-field:has(:focus-visible) {
        border-color: var(--color-rad-brand-lift);
        outline: var(--rad-focus-ring);
        outline-offset: var(--rad-focus-offset);
    }

    .amml-field__input {
        flex: 1;
        min-width: 0;
        padding: 10px 12px;
        border: 0;
        background: none;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-field__input:focus-visible {
        outline: none;
    }

    .amml-field--mono .amml-field__input {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-field__input::placeholder {
        color: var(--color-rad-faint);
    }

    .amml-field--disabled {
        opacity: var(--opacity-rad-disabled);
    }
</style>
