<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import type { WorldSourceRowProps } from "./WorldSourceRow";

    let { id, name, detail: detailText, text, oncopy }: WorldSourceRowProps = $props();
</script>

<PanelRow size="sm">
    {#snippet title()}{name}{/snippet}
    {#snippet detail()}<code class="amml-world-source__detail">{detailText}</code>{/snippet}
    {#snippet aside()}<Button variant="secondary" onclick={() => oncopy?.(id)}>{text.copy}</Button>{/snippet}
</PanelRow>

<style>
    .amml-world-source__detail {
        display: block;
    }
</style>
