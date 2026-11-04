<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import type { MoreRowProps } from "./MoreRow";

    let { label, onclick }: MoreRowProps = $props();
</script>

<li class="amml-more">
    <Button variant="link" tone="brand" onclick={() => onclick?.()}>{label}</Button>
</li>

<style>
    .amml-more {
        padding: 11px 26px 11px 20px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-more :global(.amml-btn.amml-btn--link) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
