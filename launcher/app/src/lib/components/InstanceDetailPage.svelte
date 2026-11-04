<script lang="ts">
    import { goto } from "$app/navigation";
    import { ConfirmDialog, InstanceDetailScreen, InstanceHeader } from "@amml/launcher-kit";
    import { getContext, onMount, untrack } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { InstanceDetailStore } from "#lib/stores/InstanceDetailStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";
    import AppShell from "./AppShell.svelte";

    let { name }: { name: string } = $props();

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new InstanceDetailStore(
        untrack(() => name),
        LauncherApi,
        (path) => void goto(path),
        () => shell.registries,
    );
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);
    const shown = $derived(store.header());

    onMount(() => store.load());
</script>

<AppShell {shell} {fault} onfault={() => store.fault && actions.run(store.fault, () => store.load())}>
    {#if shown}
        <InstanceDetailScreen
            mods={store.installed()}
            text={store.text}
            onupdateall={() => store.updateAll()}
            onadd={() => store.add()}
            ontoggle={(id, enabled) => store.toggle(id, enabled)}
            onmenu={(id, item) => store.menuAction(id, item)}
            onupdate={(id) => store.update(id)}
        >
            {#snippet header()}
                <InstanceHeader
                    instance={shown}
                    text={store.headerText}
                    onback={() => store.back()}
                    ondelete={() => store.askDelete()}
                    onopenfolder={() => store.openFolder()}
                    onlaunch={() => store.launch()}
                />
            {/snippet}
        </InstanceDetailScreen>
    {/if}
    {#if store.pendingRemove}
        <ConfirmDialog
            open
            text={store.removeText}
            values={{ title: store.pendingRemove.title ?? store.pendingRemove.id, id: store.pendingRemove.id }}
            confirmVariant="danger"
            onconfirm={() => store.confirmRemove()}
            oncancel={() => store.cancelRemove()}
        />
    {/if}
    {#if store.pendingDelete && store.instance}
        <ConfirmDialog
            open
            text={store.deleteText}
            values={{ name: store.instance.name }}
            path={store.instance.folder}
            confirmVariant="danger"
            onconfirm={() => store.confirmDelete()}
            oncancel={() => store.cancelDelete()}
        />
    {/if}
</AppShell>
