<script lang="ts">
    import AddModHeader from "../components/catalog/AddModHeader.svelte";
    import SourceNav from "../components/catalog/SourceNav.svelte";
    import { SourceNavRules } from "../components/catalog/SourceNav";
    import UrlInstallPane from "../components/catalog/UrlInstallPane.svelte";
    import Button from "../components/primitives/Button.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import { AddModUrlScreenRules, type AddModUrlScreenProps } from "./AddModUrlScreen";

    let {
        name,
        context,
        registries,
        allCount,
        showOtherAmml = false,
        url = "",
        checks,
        unofficial = false,
        checking = false,
        found,
        text,
        onback,
        onsource,
        onshowotheramml,
        oninput,
        oncheck,
        oninstall,
    }: AddModUrlScreenProps = $props();
</script>

<div class="amml-add-url">
    <AddModHeader {name} {context} text={text.header} {onback} />
    <div class="amml-add-url__body">
        <SourceNav
            {registries}
            {allCount}
            value={SourceNavRules.URL}
            {showOtherAmml}
            text={text.nav}
            onchange={onsource}
            {onshowotheramml}
        />
        <UrlInstallPane {url} {checks} {unofficial} {checking} text={text.pane} {oninput} {oncheck} />
    </div>
    {#if found}
        <div class="amml-add-url__bar">
            <span class="amml-add-url__found"><RichText parts={AddModUrlScreenRules.found(found, text)} /></span>
            <Button variant="warn" caps onclick={oninstall}>{text.installUnofficial}</Button>
        </div>
    {/if}
</div>

<style>
    .amml-add-url {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
    }

    .amml-add-url__body {
        display: flex;
        flex: 1 1 0;
        min-height: 0;
    }

    .amml-add-url__bar {
        display: flex;
        flex-shrink: 0;
        align-items: center;
        justify-content: space-between;
        gap: 14px;
        height: 64px;
        padding: 0 30px;
        box-sizing: border-box;
        background: var(--color-rad-void-2);
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-add-url__found {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-add-url__found :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }
</style>
