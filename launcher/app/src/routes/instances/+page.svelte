<script lang="ts">
    import { goto } from "$app/navigation";
    import { ConfirmDialog, InstancesScreen } from "@amml/launcher-kit";
    import { getContext, onDestroy, onMount } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import AppShell from "#lib/components/AppShell.svelte";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { InstancesStore } from "#lib/stores/InstancesStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new InstancesStore(LauncherApi, (path) => void goto(path));
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    // a launch the launcher refused comes first; the cut-off install it finished at start comes after
    const shown = $derived(store.fault ?? shell.startupFault);
    const fault = $derived(shown ? FaultPresenter.present(shown) : null);

    onMount(async () => {
        await store.load();
        await shell.loadStartupFault();
    });
    onDestroy(() => store.dispose());
</script>

<AppShell {shell} {fault} onfault={() => shown && actions.run(shown, () => store.load())}>
    <InstancesScreen
        instances={store.instances()}
        text={store.text}
        menuItems={store.menu}
        onnew={() => store.newInstance()}
        onreview={() => store.review()}
        onlaunch={(id) => store.launch(id)}
        onopen={(id) => store.open(id)}
        onmenu={(id, item) => store.menuAction(id, item)}
    />
    {#if store.pendingDelete}
        <ConfirmDialog
            open
            text={store.deleteText}
            values={{ name: store.pendingDelete.name }}
            path={store.pendingDelete.folder}
            confirmVariant="danger"
            onconfirm={() => store.confirmDelete()}
            oncancel={() => store.cancelDelete()}
        />
    {/if}
</AppShell>
