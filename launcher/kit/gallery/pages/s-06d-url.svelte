<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06d From a URL", group: "Screens", order: 9, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import AddModUrlScreen from "../../src/screens/AddModUrlScreen.svelte";
    import { CatalogFixtures } from "../../src/fixtures/catalog";

    let url = $state(CatalogFixtures.url);
    let showOtherAmml = $state(false);
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <AddModUrlScreen
        name={CatalogFixtures.instanceName}
        context={CatalogFixtures.context}
        registries={CatalogFixtures.registries}
        allCount={CatalogFixtures.allCount}
        {url}
        {showOtherAmml}
        checks={CatalogFixtures.urlChecks}
        unofficial
        found={CatalogFixtures.found}
        text={en.addModUrl}
        oninput={(next) => (url = next)}
        onshowotheramml={(show) => (showOtherAmml = show)}
    />
</AppFrame>
