<script lang="ts">
    import { Template } from "../../text/Template";
    import BrandMark from "../primitives/BrandMark.svelte";
    import Button from "../primitives/Button.svelte";
    import RichText from "../primitives/RichText.svelte";
    import type { AboutHeaderProps } from "./AboutHeader";

    let { text, build, updateVersion, onupdate }: AboutHeaderProps = $props();
</script>

<header class="amml-about-header">
    <BrandMark size="lg" label={text.markLabel} />
    <div class="amml-about-header__body">
        <h1 class="amml-about-header__title"><RichText parts={Template.parts(text.title)} /></h1>
        <div class="amml-about-header__build">{build}</div>
    </div>
    {#if updateVersion !== undefined}
        <div class="amml-about-header__update">
            <div class="amml-about-header__notice">{Template.fill(text.updateNotice, { version: updateVersion })}</div>
            <Button variant="primary" onclick={() => onupdate?.()}>{text.update}</Button>
        </div>
    {/if}
</header>

<style>
    .amml-about-header {
        display: flex;
        align-items: center;
        gap: 24px;
        padding: 6px 0 4px;
    }

    .amml-about-header__body {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .amml-about-header__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-page);
        font-weight: var(--font-weight-rad-light);
        line-height: 1.14;
        letter-spacing: var(--tracking-rad-page);
    }

    .amml-about-header__title :global(strong) {
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-about-header__build {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-about-header__update {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 8px;
    }

    .amml-about-header__notice {
        color: var(--color-rad-brand-lift);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }
</style>
