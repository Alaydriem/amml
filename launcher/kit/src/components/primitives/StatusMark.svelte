<script lang="ts">
    import Icon from "./Icon.svelte";
    import type { StatusMarkProps } from "./StatusMark";

    let { state, size = 22, label }: StatusMarkProps = $props();
</script>

<span class="amml-status-mark amml-status-mark--{state}" style:width="{size}px" style:height="{size}px">
    <span class="amml-status-mark__glyph" aria-hidden="true">
        {#if state === "ok"}
            <Icon name="check" size={size === 22 ? 13 : 12} weight={3} />
        {:else if state === "warn" || state === "failed"}
            <span class="amml-status-mark__bang">!</span>
        {:else if state === "active"}
            <span class="amml-status-mark__core"></span>
        {/if}
    </span>
    {#if label}
        <span class="amml-visually-hidden">{label}</span>
    {/if}
</span>

<style>
    .amml-status-mark {
        position: relative;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        box-sizing: border-box;
        border-radius: var(--radius-rad-sm);
    }

    .amml-status-mark__glyph {
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }

    .amml-status-mark--ok {
        background: var(--color-rad-ok-chip);
        color: var(--color-rad-ok);
    }

    .amml-status-mark--warn {
        background: var(--color-rad-warn-chip);
        color: var(--color-rad-warn);
    }

    .amml-status-mark--failed {
        border: 1px solid var(--color-rad-fault);
        background: var(--color-rad-fault-chip);
        color: var(--color-rad-fault);
    }

    .amml-status-mark--active {
        border: 1px solid var(--color-rad-brand-lift);
    }

    .amml-status-mark--pending {
        border: 1px solid var(--color-rad-line-2);
    }

    .amml-status-mark__bang {
        font-size: var(--text-rad-small);
        font-weight: var(--font-weight-rad-heavy);
        line-height: 18px;
    }

    .amml-status-mark__core {
        width: 8px;
        height: 8px;
        border-radius: var(--radius-rad-2xs);
        background: var(--color-rad-brand-lift);
    }
</style>
