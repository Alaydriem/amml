<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06b Add a mod", group: "Screens", order: 7, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import type { ModDetail, ModDetailPending, PageRef } from "../../src/model";
    import AddModBrowseScreen from "../../src/screens/AddModBrowseScreen.svelte";
    import { CatalogFixtures } from "../../src/fixtures/catalog";

    let groups = $state(CatalogFixtures.browseGroups());
    let loading = $state<PageRef[]>([]);
    let checked = $state(["@kv/chunkborders", "@tobi/clearweather"]);
    let active = $state("@kv/chunkborders");
    let source = $state("community");
    let query = $state("");
    let sort = $state("name");
    let showOtherAmml = $state(false);
    let detail = $state<ModDetail | undefined>(CatalogFixtures.chunkBorders);
    let pending = $state<ModDetailPending | undefined>();

    const cards = $derived(
        [CatalogFixtures.chunkBordersCard, CatalogFixtures.clearWeatherCard].filter((card) => checked.includes(card.id)),
    );

    // Stand-in for the app: the rows arrive after a short delay.
    function loadMore(ref: PageRef) {
        loading = [...loading, ref];
        setTimeout(() => {
            groups = groups.map((group) =>
                group.registryId === ref.registryId ? CatalogFixtures.withPage(group, ref.page) : group,
            );
            loading = loading.filter((entry) => entry.registryId !== ref.registryId || entry.page !== ref.page);
        }, 700);
    }

    // Stand-in for the app: the mod document arrives after a short delay.
    function activate(id: string) {
        active = id;
        detail = undefined;
        pending = undefined;
        if (id !== CatalogFixtures.chunkBordersCard.id) return;
        pending = { ...CatalogFixtures.pendingMapper, card: CatalogFixtures.chunkBordersCard };
        setTimeout(() => {
            if (active !== id) return;
            pending = undefined;
            detail = CatalogFixtures.chunkBorders;
        }, 900);
    }
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
        resultLine={CatalogFixtures.browseLine}
        {groups}
        {checked}
        selectedCards={cards}
        {active}
        {detail}
        {pending}
        {loading}
        text={en.addModBrowse}
        onsource={(id) => (source = id)}
        onshowotheramml={(show) => (showOtherAmml = show)}
        onsearch={(next) => (query = next)}
        onsort={(id) => (sort = id)}
        onchecked={(ids) => (checked = ids)}
        onactivate={activate}
        onloadmore={loadMore}
        onclear={() => (checked = [])}
    />
</AppFrame>
