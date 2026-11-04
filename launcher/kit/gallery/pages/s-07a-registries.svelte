<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "07a Registries", group: "Screens", order: 10, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, SettingsLayout, SettingsNav, StatusBar, TitleBar } from "../../src/components/shell";
    import { SettingsFixtures } from "../../src/fixtures/settings";
    import { SettingsPages } from "../SettingsPages";
    import RegistriesSettingsScreen from "../../src/screens/RegistriesSettingsScreen.svelte";
    import { registriesSettingsEn } from "../../src/text/en/registriesSettings";

    let registries = $state(SettingsFixtures.registries);

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
            <SettingsNav text={en.shell.settingsNav} items={en.shell.settingsSections} value="registries" onchange={SettingsPages.open} />
        {/snippet}
        <RegistriesSettingsScreen
            {registries}
            text={registriesSettingsEn}
            onremoveregistry={(id) => (registries = registries.filter((registry) => registry.id !== id))}
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
