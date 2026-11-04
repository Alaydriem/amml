<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "06 Instance detail", group: "Screens", order: 6, frame: true };
</script>

<script lang="ts">
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";
    import { AppFrame, StatusBar, TitleBar } from "../../src/components/shell";
    import { InstanceHeader } from "../../src/components/instances";
    import InstanceDetailScreen from "../../src/screens/InstanceDetailScreen.svelte";
    import { ModFixtures } from "../../src/fixtures/mods";
    import { InstanceFixtures } from "../../src/fixtures/instances";
    import { instanceDetailEn } from "../../src/text/en/instanceDetail";
    import { instanceHeaderEn } from "../../src/text/en/instances";

    const instance = InstanceFixtures.instances[0]!;
    let mods = $state(ModFixtures.installed);

    function toggle(id: string, enabled: boolean) {
        mods = mods.map((mod) => (mod.id === id ? { ...mod, enabled } : mod));
    }

    function update(id: string) {
        mods = mods.map((mod) => (mod.id === id ? { ...mod, installedVersion: mod.availableVersion ?? mod.installedVersion } : mod));
    }

    function updateAll() {
        mods = mods.map((mod) => ({ ...mod, installedVersion: mod.availableVersion ?? mod.installedVersion }));
    }
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
        />
    {/snippet}
    <InstanceDetailScreen
        {mods}
        text={instanceDetailEn}
        ontoggle={toggle}
        onupdate={update}
        onupdateall={updateAll}
    >
        {#snippet header()}
            <InstanceHeader {instance} text={instanceHeaderEn} />
        {/snippet}
    </InstanceDetailScreen>
    {#snippet statusbar()}
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    {/snippet}
</AppFrame>
