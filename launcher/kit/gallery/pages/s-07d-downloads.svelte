<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "07d Downloads", group: "Screens", order: 13, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, SettingsLayout, SettingsNav, StatusBar, TitleBar } from "../../src/components/shell";
    import { SettingsFixtures } from "../../src/fixtures/settings";
    import { SettingsPages } from "../SettingsPages";
    import DownloadsSettingsScreen from "../../src/screens/DownloadsSettingsScreen.svelte";
    import { downloadsSettingsEn } from "../../src/text/en/downloadsSettings";

    let files = $state(SettingsFixtures.downloadFiles);
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <SettingsLayout>
        {#snippet nav()}
            <SettingsNav text={en.shell.settingsNav} items={en.shell.settingsSections} value="downloads" onchange={SettingsPages.open} />
        {/snippet}
        <DownloadsSettingsScreen
            folder={SettingsFixtures.downloadsFolder}
            usedBytes={SettingsFixtures.downloadsUsedBytes}
            {files}
            text={downloadsSettingsEn}
            onremovefile={(id) => (files = files.filter((file) => file.id !== id))}
            onremoveunfinished={() => (files = files.filter((file) => file.state !== "unfinished"))}
        />
    </SettingsLayout>
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
