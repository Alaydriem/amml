<script lang="ts">
    import { goto } from "$app/navigation";
    import { ConfirmDialog, InstallProgressScreen } from "@amml/launcher-kit";
    import { getContext, onDestroy, onMount } from "svelte";
    import { JobEvents } from "#lib/api/JobEvents.ts";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import AppShell from "#lib/components/AppShell.svelte";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { InstallStore } from "#lib/stores/InstallStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new InstallStore(LauncherApi, JobEvents, (path) => void goto(path), () => shell.showLog());
    const failure = $derived(store.failure ? FaultPresenter.present(store.failure) : null);

    // with no job to show, the player is back at the list
    onMount(async () => {
        await store.start();
        if (store.snapshot === null) await goto("/instances", { replace: true });
    });
    onDestroy(() => store.stopWatching());
</script>

<AppShell {shell} fault={failure}>
    {#if store.snapshot}
        <InstallProgressScreen
            name={store.snapshot.titleName}
            steps={store.steps}
            progress={store.progress}
            stepNumber={store.snapshot.stepNumber}
            stepTotal={store.snapshot.stepTotal}
            text={store.text}
            fault={store.fault?.fault}
            faultText={store.fault?.text}
            restored={store.snapshot.fault?.restored ?? false}
            onback={() => store.back()}
            oncancel={() => store.cancel()}
            onretry={() => store.retry()}
            onshowlog={() => store.showLog()}
        />
    {/if}
    {#if store.asking}
        <ConfirmDialog
            open
            text={store.developerModeText}
            values={store.developerModeValues}
            onconfirm={() => store.allowDeveloperMode()}
            oncancel={() => store.refuseDeveloperMode()}
        />
    {/if}
</AppShell>
