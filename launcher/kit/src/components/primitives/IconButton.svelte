<script lang="ts">
    import Icon from "./Icon.svelte";
    import type { IconButtonProps } from "./IconButton";

    let { icon, label, iconSize = 18, disabled = false, haspopup, expanded, onclick }: IconButtonProps = $props();
</script>

<button
    class="amml-icon-btn"
    type="button"
    aria-label={label}
    aria-haspopup={haspopup}
    aria-expanded={expanded}
    {disabled}
    onclick={() => onclick?.()}
>
    <Icon name={icon} size={iconSize} />
</button>

<style>
    .amml-icon-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 40px;
        height: 40px;
        flex-shrink: 0;
        padding: 0;
        border: 0;
        border-radius: var(--radius-rad-md);
        background: none;
        color: var(--color-rad-faint);
        cursor: pointer;
    }

    .amml-icon-btn:hover:not(:disabled),
    .amml-icon-btn[aria-expanded="true"] {
        background: var(--color-rad-panel);
        color: var(--color-rad-tx);
    }

    .amml-icon-btn:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
