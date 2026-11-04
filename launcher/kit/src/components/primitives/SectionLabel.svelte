<script lang="ts">
    import Eyebrow from "./Eyebrow.svelte";
    import type { SectionLabelProps } from "./SectionLabel";

    let { sticky = false, children }: SectionLabelProps = $props();
</script>

<div class="amml-section-label" class:amml-section-label--sticky={sticky}>
    <Eyebrow size="sm">{@render children()}</Eyebrow>
</div>

<style>
    .amml-section-label {
        padding: 6px 26px 6px 20px;
        background: var(--color-rad-void-2);
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-section-label--sticky {
        position: sticky;
        top: 0;
        z-index: 1;
    }
</style>
