<script lang="ts">
    import type { EyebrowProps } from "./Eyebrow";

    let { size = "lg", tone = "faint", id, children }: EyebrowProps = $props();
</script>

<span {id} class="amml-eyebrow amml-eyebrow--{size} amml-eyebrow--{tone}">{@render children()}</span>

<style>
    .amml-eyebrow {
        display: block;
        font-family: var(--font-rad-mono);
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
    }

    .amml-eyebrow--sm {
        font-size: var(--text-rad-label);
        line-height: 16px;
    }

    .amml-eyebrow--md {
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-eyebrow--lg {
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-eyebrow--faint {
        color: var(--color-rad-faint);
    }

    .amml-eyebrow--warn {
        color: var(--color-rad-warn);
    }
</style>
