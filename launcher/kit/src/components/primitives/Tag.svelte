<script lang="ts">
    import type { TagProps } from "./Tag";

    let { tone = "faint", bordered = true, children }: TagProps = $props();
</script>

<span class="amml-tag amml-tag--{tone}" class:amml-tag--bordered={bordered}>{@render children()}</span>

<style>
    .amml-tag {
        display: inline-block;
        color: var(--amml-tag-color);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-caption);
        line-height: 14px;
        letter-spacing: var(--tracking-rad-tag);
        text-transform: uppercase;
    }

    .amml-tag--bordered {
        padding: 3px 6px;
        border: 1px solid var(--amml-tag-edge);
        border-radius: var(--radius-rad-xs);
    }

    .amml-tag--faint {
        --amml-tag-color: var(--color-rad-faint);
        --amml-tag-edge: var(--color-rad-line-2);
    }

    .amml-tag--ok {
        --amml-tag-color: var(--color-rad-ok);
        --amml-tag-edge: var(--color-rad-ok-edge);
    }

    .amml-tag--warn {
        --amml-tag-color: var(--color-rad-warn);
        --amml-tag-edge: var(--color-rad-warn-edge);
    }

    .amml-tag--fault {
        --amml-tag-color: var(--color-rad-fault);
        --amml-tag-edge: var(--color-rad-fault-edge);
    }

    .amml-tag--brand {
        --amml-tag-color: var(--color-rad-brand-lift);
        --amml-tag-edge: var(--color-rad-brand-edge);
    }
</style>
