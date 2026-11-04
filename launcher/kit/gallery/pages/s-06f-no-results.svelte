<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06f No results", group: "Screens", order: 9.6, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import AddModBrowseScreen from "../../src/screens/AddModBrowseScreen.svelte";
    import { CatalogFixtures } from "../../src/fixtures/catalog";

    let source = $state("community");
    let query = $state(CatalogFixtures.noResultsQuery);
    let sort = $state("name");
    let showOtherAmml = $state(false);
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <AddModBrowseScreen
        name={CatalogFixtures.instanceName}
        context={CatalogFixtures.context}
        registries={CatalogFixtures.registries}
        allCount={CatalogFixtures.allCount}
        {source}
        {showOtherAmml}
        {query}
        {sort}
        resultLine={CatalogFixtures.noResultsLine}
        groups={[]}
        selectedCards={[]}
        otherMatches={CatalogFixtures.noResultsOtherMatches}
        text={en.addModBrowse}
        onsource={(id) => (source = id)}
        onshowotheramml={(show) => (showOtherAmml = show)}
        onsearch={(next) => (query = next)}
        onsort={(id) => (sort = id)}
        onsearchall={() => (source = "all")}
        onclearsearch={() => (query = "")}
    />
</AppFrame>
