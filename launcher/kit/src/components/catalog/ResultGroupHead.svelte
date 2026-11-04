<script lang="ts">
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import type { ResultGroupHeadProps } from "./ResultGroupHead";

    let { id, heading }: ResultGroupHeadProps = $props();
</script>

<h2 class="amml-group-head" {id} data-head>
    <Eyebrow size="sm">{heading}</Eyebrow>
</h2>

<style>
    .amml-group-head {
        position: sticky;
        top: 0;
        z-index: 1;
        margin: 0;
        padding: 6px 26px 6px 20px;
        background: var(--color-rad-void-2);
        border-bottom: 1px solid var(--color-rad-line);
        font-weight: var(--font-weight-rad-regular);
    }
</style>
