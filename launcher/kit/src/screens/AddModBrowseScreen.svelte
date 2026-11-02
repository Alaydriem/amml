<script lang="ts">
    import AddModHeader from "../components/catalog/AddModHeader.svelte";
    import CatalogToolbar from "../components/catalog/CatalogToolbar.svelte";
    import ModDetailPane from "../components/catalog/ModDetailPane.svelte";
    import ModResultList from "../components/catalog/ModResultList.svelte";
    import SelectionBar from "../components/catalog/SelectionBar.svelte";
    import SourceNav from "../components/catalog/SourceNav.svelte";
    import { SourceNavRules } from "../components/catalog/SourceNav";
    import Button from "../components/primitives/Button.svelte";
    import EmptyState from "../components/primitives/EmptyState.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import { AddModBrowseScreenRules, type AddModBrowseScreenProps } from "./AddModBrowseScreen";

    let {
        name,
        context,
        registries,
        allCount,
        source = SourceNavRules.ALL,
        showOtherAmml = false,
        query = "",
        sort,
        resultLine,
        groups,
        checked = [],
        selectedCards,
        active,
        detail,
        pending,
        loading = [],
        generation = 0,
        otherMatches = 0,
        showGroupCount = false,
        text,
        observerFactory,
        onback,
        onsource,
        onshowotheramml,
        onsearch,
        onsort,
        onchecked,
        onloadmore,
        onshowmore,
        onactivate,
        onclear,
        oninstall,
        onretrysource,
        onopensettings,
        onsearchall,
        onclearsearch,
    }: AddModBrowseScreenProps = $props();

    const registry = $derived(AddModBrowseScreenRules.registry(source, registries));
    const view = $derived(AddModBrowseScreenRules.view(source, registries, groups, query));
</script>

<div class="amml-add-browse">
    <AddModHeader {name} {context} text={text.header} {onback} />
    <div class="amml-add-browse__body">
        <SourceNav
            {registries}
            {allCount}
            value={source}
            {showOtherAmml}
            text={text.nav}
            onchange={onsource}
            {onshowotheramml}
        />
        <div class="amml-add-browse__center">
            <CatalogToolbar
                {query}
                placeholder={AddModBrowseScreenRules.placeholder(source, registries, text)}
                {sort}
                resultLine={AddModBrowseScreenRules.line(view, registry, resultLine, text)}
                disabled={view === "offline"}
                text={text.toolbar}
                {onsearch}
                {onsort}
            />
            {#if view === "offline" && registry}
                <div class="amml-add-browse__state amml-add-browse__state--offline">
                    <EmptyState icon="alert-circle" title={AddModBrowseScreenRules.offlineTitle(registry, text)}>
                        {#snippet body()}<RichText parts={AddModBrowseScreenRules.offlineBody(registry, text)} />{/snippet}
                        {#snippet actions()}
                            <Button onclick={() => onretrysource?.(registry.id)}>{text.offline.retry}</Button>
                            <Button variant="ghost" onclick={onopensettings}>{text.offline.settings}</Button>
                        {/snippet}
                    </EmptyState>
                </div>
            {:else if view === "empty"}
                <div class="amml-add-browse__state">
                    <EmptyState icon="search" title={AddModBrowseScreenRules.noResultsTitle(registry, query, text)}>
                        {#snippet body()}{AddModBrowseScreenRules.noResultsBody(otherMatches, text)}{/snippet}
                        {#snippet actions()}
                            {#if AddModBrowseScreenRules.canSearchAll(source, otherMatches)}
                                <Button onclick={onsearchall}>{text.noResults.searchAll}</Button>
                            {/if}
                            <Button variant="ghost" onclick={onclearsearch}>{text.noResults.clear}</Button>
                        {/snippet}
                    </EmptyState>
                </div>
            {:else}
                <ModResultList
                    {groups}
                    {checked}
                    {active}
                    {loading}
                    {generation}
                    {showGroupCount}
                    text={text.list}
                    {observerFactory}
                    {onloadmore}
                    {onshowmore}
                    {onactivate}
                    onchange={onchecked}
                />
            {/if}
        </div>
        <ModDetailPane {detail} {pending} text={text.detail} />
    </div>
    <SelectionBar selected={selectedCards} text={text.bar} {onclear} {oninstall} />
</div>

<style>
    .amml-add-browse {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
    }

    .amml-add-browse__body {
        display: flex;
        flex: 1 1 0;
        min-height: 0;
    }

    .amml-add-browse__center {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
        min-width: 0;
    }

    /* The compact empty state of sheets 06e and 06f: smaller title and body, a closer top edge. */
    .amml-add-browse__state {
        flex: 1 1 0;
        min-height: 0;
        padding: 0 36px;
        overflow-y: auto;
    }

    .amml-add-browse__state :global(.amml-empty) {
        gap: 14px;
        padding-top: 56px;
    }

    .amml-add-browse__state :global(.amml-empty__title) {
        font-size: var(--text-rad-title-sm);
        line-height: 24px;
        letter-spacing: normal;
    }

    .amml-add-browse__state :global(.amml-empty__body) {
        max-width: 520px;
        font-size: var(--text-rad-body-sm);
        line-height: 160%;
    }

    .amml-add-browse__state--offline :global(.amml-empty__mark) {
        color: var(--color-rad-fault);
    }

    .amml-add-browse__state--offline :global(.amml-empty__body strong) {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        font-weight: var(--font-weight-rad-regular);
    }
</style>
