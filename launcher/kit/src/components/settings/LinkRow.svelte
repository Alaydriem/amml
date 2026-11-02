<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import type { LinkRowProps } from "./LinkRow";

    let { label, value, href, onopenlink }: LinkRowProps = $props();
</script>

<PanelRow size="sm">
    {#snippet title()}{label}{/snippet}
    {#snippet aside()}
        <span class="amml-link-row__link">
            <Button variant="link" onclick={() => onopenlink?.(href)}>{value}</Button>
        </span>
    {/snippet}
</PanelRow>

<style>
    .amml-link-row__link {
        display: flex;
    }

    .amml-link-row__link :global(.amml-btn.amml-btn--link) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
