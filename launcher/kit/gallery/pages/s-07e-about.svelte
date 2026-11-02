<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "07e About", group: "Screens", order: 14, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, SettingsLayout, SettingsNav, StatusBar, TitleBar } from "../../src/components/shell";
    import { SettingsFixtures } from "../../src/fixtures/settings";
    import { SettingsPages } from "../SettingsPages";
    import AboutSettingsScreen from "../../src/screens/AboutSettingsScreen.svelte";
    import { aboutSettingsEn } from "../../src/text/en/aboutSettings";

    let copied = $state("");

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
            <SettingsNav text={en.shell.settingsNav} items={en.shell.settingsSections} value="about" onchange={SettingsPages.open} />
        {/snippet}
        <AboutSettingsScreen
            version={SettingsFixtures.aboutVersion}
            builtOn={SettingsFixtures.aboutBuiltOn}
            platform={SettingsFixtures.aboutPlatform}
            updateVersion={SettingsFixtures.aboutUpdateVersion}
            author={SettingsFixtures.aboutAuthor}
            links={SettingsFixtures.aboutLinks}
            report={SettingsFixtures.bugReport}
            text={aboutSettingsEn}
            oncopyreport={(text) => (copied = text)}
        />
    </SettingsLayout>
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            message={copied === "" ? undefined : copied.replaceAll(String.fromCharCode(10), " | ")}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
