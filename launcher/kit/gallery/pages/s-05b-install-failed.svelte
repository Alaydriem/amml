<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "05b Install failed", group: "Screens", order: 5.5, frame: true };
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
        <TitleBar text={en.shell.titleBar} registryFetched={ShellFixtures.registryFetched} />
    {/snippet}
    <InstallProgressScreen
        name={InstallFixtures.name}
        steps={InstallFixtures.failedSteps}
        stepNumber={InstallFixtures.stepNumber}
        stepTotal={InstallFixtures.stepTotal}
        text={installProgressEn}
        fault={InstallFixtures.fault}
        faultText={en.faults[InstallFixtures.fault.kind]}
        restored
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
