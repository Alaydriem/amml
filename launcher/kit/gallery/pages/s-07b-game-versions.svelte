<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "07b Game versions", group: "Screens", order: 11, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, SettingsLayout, SettingsNav, StatusBar, TitleBar } from "../../src/components/shell";
    import { SettingsFixtures } from "../../src/fixtures/settings";
    import { SettingsPages } from "../SettingsPages";
    import GameVersionsSettingsScreen from "../../src/screens/GameVersionsSettingsScreen.svelte";
    import { gameVersionsSettingsEn } from "../../src/text/en/gameVersionsSettings";

    let showPreview = $state(false);
    let versions = $state(SettingsFixtures.versions);

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
            <SettingsNav text={en.shell.settingsNav} items={en.shell.settingsSections} value="versions" onchange={SettingsPages.open} />
        {/snippet}
        <GameVersionsSettingsScreen
            {showPreview}
            versionSource={SettingsFixtures.versionSource}
            releaseCount={SettingsFixtures.releaseCount}
            previewCount={SettingsFixtures.previewCount}
            fetched={SettingsFixtures.versionsFetched}
            {versions}
            versionsFolder={SettingsFixtures.versionsFolder}
            text={gameVersionsSettingsEn}
            onpreviewchange={(on) => (showPreview = on)}
            onremoveversion={(version) => (versions = versions.filter((item) => item.version !== version))}
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
