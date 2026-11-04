<script lang="ts">
    import { goto } from "$app/navigation";
    import { AddModBrowseScreen } from "@amml/launcher-kit";
    import { getContext, onMount, untrack } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { CatalogStore } from "#lib/stores/CatalogStore.svelte.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";
    import AppShell from "./AppShell.svelte";

    let { name }: { name: string } = $props();

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new CatalogStore(
        untrack(() => name), LauncherApi, (path) => void goto(path));
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);

    onMount(() => store.load());
</script>

<AppShell {shell} {fault} onfault={() => store.fault && actions.run(store.fault, () => store.load())}>
    <AddModBrowseScreen
        {name}
        context={store.context}
        registries={store.sources}
        allCount={store.allCount}
        source={store.source}
        query={store.query}
        sort={store.sort}
        resultLine={store.resultLine()}
        groups={store.groups()}
        checked={store.checked}
        selectedCards={store.selected}
        active={store.active}
        detail={store.detail}
        pending={store.pending}
        loading={store.loading}
        generation={store.generation}
        otherMatches={store.otherMatches()}
        showGroupCount={store.searching}
        text={store.text}
        onback={() => store.back()}
        onsource={(id) => store.chooseSource(id)}
        onsearch={(query) => store.setQuery(query)}
        onsort={(id) => store.setSort(id)}
        onchecked={(ids) => store.setChecked(ids)}
        onloadmore={(page) => store.loadPage(page)}
        onshowmore={(id) => store.showMore(id)}
        onactivate={(id) => store.activate(id)}
        onclear={() => store.clearSelection()}
        oninstall={() => store.install()}
        onretrysource={(id) => store.retrySource(id)}
        onopensettings={() => store.openSettings()}
        onsearchall={() => store.searchEverywhere()}
        onclearsearch={() => store.clearSearch()}
    />
</AppShell>
