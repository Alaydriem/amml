<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "03b Instances / None yet", group: "Screens", order: 3.5, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import InstancesScreen from "../../src/screens/InstancesScreen.svelte";
    import { instanceMenuEn, instancesEn } from "../../src/text/en/instances";
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar text={en.shell.titleBar} registryFetched={ShellFixtures.registryFetched} />
    {/snippet}
    <InstancesScreen instances={[]} text={instancesEn} menuItems={instanceMenuEn} />
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
