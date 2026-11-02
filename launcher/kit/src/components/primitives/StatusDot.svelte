<script lang="ts">
    import type { StatusDotProps } from "./StatusDot";

    let { tone = "ok", size = "md", caps = false, children }: StatusDotProps = $props();
</script>

<span class="amml-status-dot amml-status-dot--{size} amml-status-dot--{tone}" class:amml-status-dot--caps={caps}>
    <span class="amml-status-dot__dot"></span>
    <span class="amml-status-dot__text">{@render children()}</span>
</span>

<style>
    .amml-status-dot {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-status-dot--sm {
        gap: 7px;
        font-size: var(--text-rad-small);
    }

    .amml-status-dot--caps {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        letter-spacing: var(--tracking-rad-caps);
        text-transform: uppercase;
    }

    .amml-status-dot__dot {
        width: 8px;
        height: 8px;
        flex-shrink: 0;
        border-radius: var(--radius-rad-hair);
        background: var(--amml-dot);
    }

    .amml-status-dot--sm .amml-status-dot__dot {
        width: 7px;
        height: 7px;
    }

    .amml-status-dot--ok {
        --amml-dot: var(--color-rad-ok);
    }

    .amml-status-dot--warn {
        --amml-dot: var(--color-rad-warn);
    }

    .amml-status-dot--fault {
        --amml-dot: var(--color-rad-fault);
    }

    .amml-status-dot--brand,
    .amml-status-dot--info {
        --amml-dot: var(--color-rad-brand-lift);
    }

    .amml-status-dot--muted {
        --amml-dot: var(--color-rad-faint);
    }
</style>
