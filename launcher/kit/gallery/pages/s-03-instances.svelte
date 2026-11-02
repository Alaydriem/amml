<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "03 Instances", group: "Screens", order: 3, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import InstancesScreen from "../../src/screens/InstancesScreen.svelte";
    import { InstanceFixtures } from "../../src/fixtures/instances";
    import { instanceMenuEn, instancesEn } from "../../src/text/en/instances";

    let instances = $state(InstanceFixtures.instances);

    function launch(id: string) {
        instances = instances.map((instance) =>
            instance.id === id ? { ...instance, running: true, activity: "running since now" } : instance,
        );
    }
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <InstancesScreen {instances} text={instancesEn} menuItems={instanceMenuEn} onlaunch={launch} />
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
