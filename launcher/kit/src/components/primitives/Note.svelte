<script lang="ts">
    import type { NoteProps } from "./Note";

    let { children }: NoteProps = $props();
</script>

<p class="amml-note">{@render children()}</p>

<style>
    .amml-note {
        margin: 0;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 1.6;
    }

    .amml-note :global(strong) {
        color: var(--color-rad-dim);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-note :global(code) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
    }
</style>
