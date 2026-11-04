<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "07c Shared data", group: "Screens", order: 12, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, SettingsLayout, SettingsNav, StatusBar, TitleBar } from "../../src/components/shell";
    import { SettingsFixtures } from "../../src/fixtures/settings";
    import { SettingsPages } from "../SettingsPages";
    import SharedDataSettingsScreen from "../../src/screens/SharedDataSettingsScreen.svelte";
    import { sharedDataSettingsEn } from "../../src/text/en/sharedDataSettings";

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
            <SettingsNav text={en.shell.settingsNav} items={en.shell.settingsSections} value="shared" onchange={SettingsPages.open} />
        {/snippet}
        <SharedDataSettingsScreen
            stats={SettingsFixtures.sharedStats}
            locations={SettingsFixtures.worldLocations}
            sources={SettingsFixtures.worldSources}
            text={sharedDataSettingsEn}
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
