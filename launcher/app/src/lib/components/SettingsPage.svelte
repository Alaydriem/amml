<script lang="ts">
    import { goto } from "$app/navigation";
    import {
        AboutSettingsScreen,
        AddRegistryDialog,
        ConfirmDialog,
        DownloadsSettingsScreen,
        GameVersionsSettingsScreen,
        RegistriesSettingsScreen,
        SettingsLayout,
        SettingsNav,
        SharedDataSettingsScreen,
    } from "@amml/launcher-kit";
    import { getContext, onDestroy, untrack } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import { RegistryEvents } from "#lib/api/RegistryEvents.ts";
    import { FaultPresenter } from "#lib/present/FaultPresenter.ts";
    import { FaultActions } from "#lib/stores/FaultActions.ts";
    import { SettingsStore } from "#lib/stores/SettingsStore.svelte.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";
    import AppShell from "./AppShell.svelte";

    let { section }: { section: string | undefined } = $props();

    const shell = getContext<ShellStore>(ShellStore.KEY);
    const store = new SettingsStore(
        LauncherApi,
        (path) => void goto(path),
        (text) => navigator.clipboard.writeText(text),
        (text) => shell.say(text),
        RegistryEvents,
    );
    const actions = new FaultActions(shell, LauncherApi, (path) => void goto(path), (text) => navigator.clipboard.writeText(text));
    const fault = $derived(store.fault ? FaultPresenter.present(store.fault) : null);
    const stats = $derived(store.stats());

    // the page follows the address
    $effect(() => {
        const id = section;
        untrack(() => void store.open(id));
    });

    onDestroy(() => store.stopWatching());
</script>

<AppShell {shell} {fault} onfault={() => store.fault && actions.run(store.fault, () => store.open(section))}>
    <SettingsLayout>
        {#snippet nav()}
            <SettingsNav
                text={store.texts.shell.settingsNav}
                items={store.items}
                value={store.section}
                onback={() => store.back()}
                onchange={(id) => store.pick(id)}
            />
        {/snippet}
        {#if store.section === "registries"}
            <RegistriesSettingsScreen
                registries={store.sources()}
                text={store.texts.registries}
                onaddregistry={() => store.openAdd()}
                onremoveregistry={(id) => store.removeRegistry(id)}
            />
        {:else if store.section === "versions"}
            <GameVersionsSettingsScreen
                showPreview={store.settings?.preview ?? false}
                versionSource={SettingsStore.VERSION_SOURCE}
                releaseCount={store.releaseCount}
                previewCount={store.previewCount}
                fetched={store.versionsFetched()}
                versions={store.gameVersions()}
                versionsFolder={store.versionsFolder()}
                text={store.texts.versions}
                onpreviewchange={(on) => store.setPreview(on)}
                onrefreshversions={() => store.refreshVersions()}
                onremoveversion={(version) => store.askRemoveVersion(version)}
            />
        {:else if store.section === "shared" && stats}
            <SharedDataSettingsScreen
                {stats}
                locations={store.locations()}
                sources={store.worldSources()}
                text={store.texts.shared}
                onopenfolder={() => store.openSharedFolder()}
                oncopyworlds={(id) => store.copyWorlds(id)}
            />
        {:else if store.section === "downloads" && store.downloads}
            <DownloadsSettingsScreen
                folder={store.downloads.folder}
                usedBytes={store.downloads.used}
                files={store.downloadFiles()}
                text={store.texts.downloads}
                onremovefile={(id) => store.removeFile(id)}
                onremoveunfinished={() => store.removeUnfinished()}
            />
        {:else if store.section === "about" && store.about}
            <AboutSettingsScreen
                version={store.about.launcherVersion}
                builtOn={store.about.builtOn}
                platform={store.platform()}
                author={SettingsStore.AUTHOR}
                links={SettingsStore.LINKS}
                report={store.report()}
                text={store.texts.about}
                onopenlink={(href) => store.openLink(href)}
                oncopyreport={(report) => store.copyReport(report)}
            />
        {/if}
    </SettingsLayout>
    <AddRegistryDialog
        open={store.addOpen}
        text={store.texts.addRegistry}
        value={store.addValue}
        problem={store.addProblem}
        oninput={(value) => store.setAddValue(value)}
        onadd={(url) => store.add(url)}
        oncancel={() => store.closeAdd()}
    />
    {#if store.pendingVersion}
        <ConfirmDialog
            open
            text={store.texts.removeVersion}
            values={{ version: store.pendingVersion }}
            confirmVariant="danger"
            onconfirm={() => store.confirmRemoveVersion()}
            oncancel={() => store.cancelRemoveVersion()}
        />
    {/if}
</AppShell>
