<script lang="ts">
    import { goto } from "$app/navigation";
    import { AddModUrlScreen } from "@amml/launcher-kit";
    import { getContext, onDestroy, onMount, untrack } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import { RegistryEvents } from "#lib/api/RegistryEvents.ts";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";
    import { UrlInstallStore } from "#lib/stores/UrlInstallStore.svelte.ts";
    import AppShell from "./AppShell.svelte";

    let { name }: { name: string } = $props();

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new UrlInstallStore(
        untrack(() => name), LauncherApi, (path) => void goto(path), RegistryEvents);
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);

    onMount(() => store.load());
    onDestroy(() => store.stopWatching());
</script>

<AppShell {shell} {fault} onfault={() => store.fault && actions.run(store.fault, () => store.load())}>
    <AddModUrlScreen
        {name}
        context={store.context}
        registries={store.sources}
        allCount={store.allCount}
        url={store.url}
        checks={store.checks}
        unofficial={store.unofficial}
        checking={store.checking}
        found={store.found}
        text={store.text}
        onback={() => store.back()}
        onsource={(id) => store.chooseSource(id)}
        oninput={(url) => store.setUrl(url)}
        oncheck={(url) => store.inspect(url)}
        oninstall={() => store.install()}
    />
</AppShell>
