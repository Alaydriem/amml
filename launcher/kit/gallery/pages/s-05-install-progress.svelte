<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "05 Install progress", group: "Screens", order: 5, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import InstallProgressScreen from "../../src/screens/InstallProgressScreen.svelte";
    import { InstallFixtures } from "../../src/fixtures/install";
    import { installProgressEn } from "../../src/text/en/installProgress";
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <InstallProgressScreen
        name={InstallFixtures.name}
        steps={InstallFixtures.steps}
        progress={InstallFixtures.progress}
        stepNumber={InstallFixtures.stepNumber}
        stepTotal={InstallFixtures.stepTotal}
        text={installProgressEn}
    />
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
