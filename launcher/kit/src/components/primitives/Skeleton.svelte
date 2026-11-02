<script lang="ts">
    import type { SkeletonProps } from "./Skeleton";

    let { width = "100%", height = 12, tone = "strong", radius = "sm" }: SkeletonProps = $props();
</script>

<span
    class="amml-skeleton amml-skeleton--{tone} amml-skeleton--{radius}"
    aria-hidden="true"
    style:width
    style:height="{height}px"
></span>

<style>
    .amml-skeleton {
        display: block;
        flex-shrink: 0;
    }

    .amml-skeleton--xs {
        border-radius: var(--radius-rad-xs);
    }

    .amml-skeleton--sm {
        border-radius: var(--radius-rad-sm);
    }

    .amml-skeleton--seg {
        border-radius: var(--radius-rad-seg);
    }

    .amml-skeleton--strong {
        background: var(--color-rad-line-2);
    }

    .amml-skeleton--soft {
        background: var(--color-rad-line);
    }

    .amml-skeleton--panel {
        background: var(--color-rad-panel);
    }
</style>
