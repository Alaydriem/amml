<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06c Search results", group: "Screens", order: 8, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import AddModBrowseScreen from "../../src/screens/AddModBrowseScreen.svelte";
    import { CatalogFixtures } from "../../src/fixtures/catalog";

    const groups = CatalogFixtures.searchGroups();
    let checked = $state(["@carto/mapper", "@pixl/mapart"]);
    let active = $state("@carto/mapper");
    let query = $state("map");
    let sort = $state("name");
    let showOtherAmml = $state(false);

    const cards = $derived(groups.flatMap((group) => group.rows).filter((card) => checked.includes(card.id)));
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
        source="all"
        {showOtherAmml}
        {query}
        {sort}
        resultLine={CatalogFixtures.searchLine}
        {groups}
        showGroupCount
        {checked}
        selectedCards={cards}
        {active}
        detail={CatalogFixtures.mapper}
        text={en.addModBrowse}
        onshowotheramml={(show) => (showOtherAmml = show)}
        onsearch={(next) => (query = next)}
        onsort={(id) => (sort = id)}
        onchecked={(ids) => (checked = ids)}
        onactivate={(id) => (active = id)}
        onclear={() => (checked = [])}
    />
</AppFrame>
