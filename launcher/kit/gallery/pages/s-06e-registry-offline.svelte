<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06e Registry offline", group: "Screens", order: 9.5, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import AddModBrowseScreen from "../../src/screens/AddModBrowseScreen.svelte";
    import { CatalogFixtures } from "../../src/fixtures/catalog";

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
        source="old"
        {showOtherAmml}
        {sort}
        resultLine=""
        groups={[]}
        selectedCards={[]}
        text={en.addModBrowse}
        onshowotheramml={(show) => (showOtherAmml = show)}
        onsort={(id) => (sort = id)}
    />
</AppFrame>
