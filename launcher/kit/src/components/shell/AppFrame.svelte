<script lang="ts">
    import type { AppFrameProps } from "./AppFrame";

    let { titlebar, banner, children, statusbar, scroll = false }: AppFrameProps = $props();
</script>

<div class="amml-frame">
    {@render titlebar()}
    {#if banner}
        {@render banner()}
    {/if}
    <main class="amml-frame__body" class:amml-frame__body--scroll={scroll}>
        {@render children()}
    </main>
    {#if statusbar}
        {@render statusbar()}
    {/if}
</div>

<style>
    .amml-frame {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        min-height: 0;
        overflow: clip;
        background: var(--color-rad-void);
        color: var(--color-rad-tx);
        font-family: var(--font-rad-sans);
    }

    .amml-frame__body {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
        overflow: clip;
    }

    .amml-frame__body--scroll {
        overflow-y: auto;
    }
</style>
