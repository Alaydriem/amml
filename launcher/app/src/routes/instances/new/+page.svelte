<script lang="ts">
    import { goto } from "$app/navigation";
    import { NewInstanceScreen } from "@amml/launcher-kit";
    import { getContext, onMount } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import AppShell from "#lib/components/AppShell.svelte";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { NewInstanceStore } from "#lib/stores/NewInstanceStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new NewInstanceStore(LauncherApi, (path) => void goto(path));
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);

    onMount(() => store.load());
</script>

<AppShell {shell} {fault} onfault={() => store.fault && actions.run(store.fault, () => store.load())}>
    <NewInstanceScreen
        versions={store.gameVersions}
        takenNames={store.kitTakenNames}
        summary={store.summary}
        text={store.text}
        channel={store.channel}
        showPreview={store.previewOn}
        problem={store.problemText}
        selectedVersion={store.selectedVersion}
        name={store.name}
        dataMode={store.dataMode}
        onchannelchange={(channel) => store.setChannel(channel)}
        onversionchange={(version) => store.setVersion(version)}
        onnamechange={(name) => store.setName(name)}
        ondatamodechange={(mode) => store.setDataMode(mode)}
        onback={() => store.cancel()}
        oncancel={() => store.cancel()}
        oncreate={(submit) => store.create(submit)}
    />
</AppShell>
