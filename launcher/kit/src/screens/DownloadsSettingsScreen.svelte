<script lang="ts">
    import Button from "../components/primitives/Button.svelte";
    import Panel from "../components/primitives/Panel.svelte";
    import DownloadFileRow from "../components/settings/DownloadFileRow.svelte";
    import DownloadUsage from "../components/settings/DownloadUsage.svelte";
    import { Template } from "../text";
    import { DownloadsSettingsScreenRules, type DownloadsSettingsScreenProps } from "./DownloadsSettingsScreen";

    let {
        folder,
        usedBytes,
        files,
        text,
        onremovefile,
        onremoveunfinished,
    }: DownloadsSettingsScreenProps = $props();

    const used = $derived(DownloadsSettingsScreenRules.used(usedBytes));
</script>

<DownloadUsage text={text.usage} path={folder} value={used.value} unit={used.unit} />

<Panel heading={Template.fill(text.filesHeading, { count: files.length })}>
    {#snippet aside()}
        {#if DownloadsSettingsScreenRules.hasUnfinished(files)}
            <Button variant="link" onclick={() => onremoveunfinished?.()}>{text.removeUnfinished}</Button>
        {/if}
    {/snippet}
    {#each files as file (file.id)}
        <DownloadFileRow
            id={file.id}
            name={file.name}
            detail={file.detail}
            sizeText={DownloadsSettingsScreenRules.fileSize(file.size)}
            kept={file.state === "kept"}
            text={text.file}
            onremove={onremovefile}
        />
    {/each}
</Panel>
