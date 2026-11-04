<script lang="ts">
    import { tick } from "svelte";
    import Icon from "./Icon.svelte";
    import type { SearchFieldProps } from "./SearchField";

    let { value = "", label, placeholder, disabled = false, oninput, onsubmit }: SearchFieldProps = $props();

    let input: HTMLInputElement | undefined = $state();

    async function typed(event: Event & { currentTarget: HTMLInputElement }) {
        oninput?.(event.currentTarget.value);
        // The field shows the prop: if the parent keeps the old text, the typed text is taken back.
        await tick();
        if (input && input.value !== value) input.value = value;
    }
</script>

<div class="amml-search" class:amml-search--disabled={disabled}>
    <Icon name="search" size={15} weight={2.2} />
    <input
        bind:this={input}
        class="amml-search__input"
        type="search"
        aria-label={label}
        {placeholder}
        {disabled}
        {value}
        spellcheck="false"
        oninput={typed}
        onkeydown={(event) => {
            if (event.key === "Enter") onsubmit?.(event.currentTarget.value);
        }}
    />
</div>

<style>
    .amml-search {
        display: flex;
        align-items: center;
        gap: 10px;
        padding-inline: 12px;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-md);
        color: var(--color-rad-faint);
    }

    .amml-search:has(:focus-visible) {
        border-color: var(--color-rad-brand-lift);
        outline: var(--rad-focus-ring);
        outline-offset: var(--rad-focus-offset);
    }

    .amml-search__input {
        flex: 1;
        min-width: 0;
        padding-block: 9px;
        border: 0;
        background: none;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        appearance: none;
    }

    .amml-search__input:focus-visible {
        outline: none;
    }

    .amml-search__input::placeholder {
        color: var(--color-rad-faint);
    }

    .amml-search--disabled {
        opacity: var(--opacity-rad-disabled);
    }
</style>
