<script lang="ts">
    import { goto } from "$app/navigation";
    import { InstallProgressScreen } from "@amml/launcher-kit";
    import { getContext, onDestroy, onMount } from "svelte";
    import { JobEvents } from "#lib/api/JobEvents.ts";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import AppShell from "#lib/components/AppShell.svelte";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { SetupProgressStore } from "#lib/stores/SetupProgressStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new SetupProgressStore(LauncherApi, JobEvents, (path) => void goto(path), () => shell.showLog());
    const failure = $derived(store.failure ? FaultPresenter.present(store.failure) : null);

    onMount(() => store.start());
    onDestroy(() => store.stopWatching());
</script>

<AppShell {shell} steps={{ total: 3, current: 2 }} fault={failure}>
    {#if store.snapshot}
        <InstallProgressScreen
            name={store.sourceName}
            steps={store.steps}
            progress={store.progress}
            stepNumber={store.snapshot.stepNumber}
            stepTotal={store.snapshot.stepTotal}
            text={store.text}
            fault={store.fault?.fault}
            faultText={store.fault?.text}
            restored={true}
            onother={() => store.chooseAnother()}
            onretry={() => store.retry()}
            onshowlog={() => store.showLog()}
        />
    {/if}
</AppShell>
