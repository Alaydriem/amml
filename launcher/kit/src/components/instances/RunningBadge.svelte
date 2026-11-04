<script lang="ts">
    import type { RunningBadgeProps } from "./RunningBadge";

    let { block = false, children }: RunningBadgeProps = $props();
</script>

<span class="amml-running" class:amml-running--block={block} role="status">
    <span class="amml-running__dot" aria-hidden="true"></span>
    {@render children()}
</span>

<style>
    .amml-running {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
        gap: 8px;
        min-width: 116px;
        padding: 11px 0;
        border: 1px solid var(--color-rad-ok);
        border-radius: var(--radius-rad-xs);
        color: var(--color-rad-on-brand);
        font-size: var(--text-rad-body-sm);
        font-weight: var(--font-weight-rad-strong);
        letter-spacing: var(--tracking-rad-caps);
        line-height: 18px;
        text-transform: uppercase;
        white-space: nowrap;
    }

    .amml-running--block {
        display: flex;
        width: 100%;
    }

    .amml-running__dot {
        width: 8px;
        height: 8px;
        border-radius: var(--radius-rad-hair);
        background: var(--color-rad-ok);
    }
</style>
