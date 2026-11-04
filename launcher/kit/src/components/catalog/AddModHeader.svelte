<script lang="ts">
    import BackLink from "../shell/BackLink.svelte";
    import ContextChip from "../primitives/ContextChip.svelte";
    import RichText from "../primitives/RichText.svelte";
    import { AddModHeaderRules, type AddModHeaderProps } from "./AddModHeader";

    let { name, context, text, onback }: AddModHeaderProps = $props();
</script>

<header class="amml-add-header">
    <BackLink label={text.back} onclick={onback} />
    <span class="amml-add-header__rule"></span>
    <h1 class="amml-add-header__title"><RichText parts={AddModHeaderRules.title(name, text)} /></h1>
    <span class="amml-add-header__spacer"></span>
    <ContextChip parts={AddModHeaderRules.parts(context, text)} text={text.chip} />
</header>

<style>
    .amml-add-header {
        display: flex;
        flex-shrink: 0;
        align-items: center;
        gap: 16px;
        padding: 16px 30px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-add-header__rule {
        flex-shrink: 0;
        width: 1px;
        height: 22px;
        background: var(--color-rad-line);
    }

    .amml-add-header__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title);
        font-weight: var(--font-weight-rad-light);
        line-height: 30px;
        letter-spacing: var(--tracking-rad-heading);
    }

    .amml-add-header__title :global(strong) {
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-add-header__spacer {
        flex: 1 1 0;
    }
</style>
