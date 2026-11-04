<script lang="ts">
    import Note from "../components/primitives/Note.svelte";
    import Panel from "../components/primitives/Panel.svelte";
    import SharedFolderPanel from "../components/settings/SharedFolderPanel.svelte";
    import WorldLocationRow from "../components/settings/WorldLocationRow.svelte";
    import WorldSourceRow from "../components/settings/WorldSourceRow.svelte";
    import { SharedDataSettingsScreenRules, type SharedDataSettingsScreenProps } from "./SharedDataSettingsScreen";

    let { stats, locations, sources, text, onopenfolder, oncopyworlds }: SharedDataSettingsScreenProps = $props();
</script>

<SharedFolderPanel
    text={text.folder}
    path={stats.folder}
    stats={SharedDataSettingsScreenRules.stats(stats)}
    onopen={onopenfolder}
/>

<Panel heading={text.locationsHeading}>
    {#each locations as location (location.instanceId)}
        <WorldLocationRow {location} text={text.location} />
    {/each}
</Panel>

<Panel heading={text.copyInHeading}>
    {#each sources as source (source.id)}
        <WorldSourceRow
            id={source.id}
            name={source.name}
            detail={SharedDataSettingsScreenRules.sourceDetail(source, text)}
            text={text.source}
            oncopy={oncopyworlds}
        />
    {/each}
</Panel>
<Note>{text.note}</Note>
