<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "04 New instance", group: "Screens", order: 4, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import NewInstanceScreen from "../../src/screens/NewInstanceScreen.svelte";
    import { InstanceFixtures } from "../../src/fixtures/instances";
    import { newInstanceEn } from "../../src/text/en/newInstance";
    import type { Channel, DataMode } from "../../src/model";

    let channel = $state<Channel>("release");
    let selectedVersion = $state<string | undefined>(InstanceFixtures.selectedVersion);
    let name = $state(InstanceFixtures.draftName);
    let dataMode = $state<DataMode>("shared");

    const takenNames = InstanceFixtures.instances.map((instance) => instance.name);
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <NewInstanceScreen
        versions={InstanceFixtures.versions}
        {takenNames}
        summary={InstanceFixtures.summary}
        text={newInstanceEn}
        {channel}
        {selectedVersion}
        {name}
        {dataMode}
        onchannelchange={(next) => (channel = next)}
        onversionchange={(next) => (selectedVersion = next)}
        onnamechange={(next) => (name = next)}
        ondatamodechange={(next) => (dataMode = next)}
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
