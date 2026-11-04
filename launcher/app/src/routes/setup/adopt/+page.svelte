<script lang="ts">
    import { goto } from "$app/navigation";
    import { FirstRunAdoptScreen } from "@amml/launcher-kit";
    import { getContext, onMount } from "svelte";
    import { JobEvents } from "#lib/api/JobEvents.ts";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import AppShell from "#lib/components/AppShell.svelte";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { SetupStore } from "#lib/stores/SetupStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new SetupStore(LauncherApi, JobEvents, (path) => void goto(path));
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);

    onMount(() => store.loadAdopt());
</script>

<AppShell
    {shell}
    steps={{ total: 3, current: 2 }}
    {fault}
    onfault={() => store.fault && actions.run(store.fault, () => store.adopt())}
>
    <FirstRunAdoptScreen
        instances={store.adopted}
        choices={store.choices}
        selected={store.selected}
        text={store.adoptText}
        onselect={(id) => store.select(id)}
        onnext={() => store.adopt()}
        onback={() => store.back()}
    />
</AppShell>
