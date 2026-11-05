<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "02c Setup failed", group: "Screens", order: 2.3, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import InstallProgressScreen from "../../src/screens/InstallProgressScreen.svelte";
    import { SharedDataFixtures } from "../../src/fixtures/sharedData";
    import { sharedDataProgressEn } from "../../src/text/en/sharedDataProgress";
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar text={en.shell.titleBar} registryFetched={ShellFixtures.registryFetched} />
    {/snippet}
    <InstallProgressScreen
        name={SharedDataFixtures.name}
        steps={SharedDataFixtures.failedSteps}
        stepNumber={SharedDataFixtures.stepNumber}
        stepTotal={SharedDataFixtures.stepTotal}
        text={sharedDataProgressEn}
        fault={SharedDataFixtures.fault}
        faultText={en.faults[SharedDataFixtures.fault.kind]}
        restored
        onother={() => {}}
        onretry={() => {}}
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
