<script lang="ts">
    import Eyebrow from "./Eyebrow.svelte";
    import type { PanelProps } from "./Panel";

    let { heading, roomy = false, aside, children }: PanelProps = $props();

    const headingId = $props.id();
</script>

<section class="amml-panel" aria-labelledby={headingId}>
    <header class="amml-panel__header" class:amml-panel__header--roomy={roomy}>
        <h2 class="amml-panel__heading" id={headingId}><Eyebrow size="md">{heading}</Eyebrow></h2>
        {#if aside}
            <div class="amml-panel__aside">{@render aside()}</div>
        {/if}
    </header>
    {@render children()}
</section>

<style>
    .amml-panel {
        display: flex;
        flex-direction: column;
        overflow: clip;
        background: var(--color-rad-void-2);
        border: 1px solid var(--color-rad-line);
        border-radius: var(--radius-rad-lg);
    }

    .amml-panel__header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 10px 18px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-panel__header--roomy {
        padding-block: 13px;
    }

    .amml-panel__heading {
        margin: 0;
        font-weight: var(--font-weight-rad-regular);
    }

    .amml-panel__aside {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }
</style>
