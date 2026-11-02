<script lang="ts">
    import Button from "../components/primitives/Button.svelte";
    import Note from "../components/primitives/Note.svelte";
    import Panel from "../components/primitives/Panel.svelte";
    import PanelRow from "../components/primitives/PanelRow.svelte";
    import Toggle from "../components/primitives/Toggle.svelte";
    import InstalledVersionRow from "../components/settings/InstalledVersionRow.svelte";
    import { Units } from "../text/Units";
    import { GameVersionsSettingsScreenRules, type GameVersionsSettingsScreenProps } from "./GameVersionsSettingsScreen";

    let {
        showPreview,
        versionSource,
        releaseCount,
        previewCount,
        fetched,
        versions,
        versionsFolder,
        text,
        onpreviewchange,
        onrefreshversions,
        onremoveversion,
    }: GameVersionsSettingsScreenProps = $props();

    const installed = $derived(GameVersionsSettingsScreenRules.installed(versions));
</script>

<Panel heading={text.buildsHeading}>
    <PanelRow>
        {#snippet title()}{text.previewTitle}{/snippet}
        {#snippet detail()}{text.previewNote}{/snippet}
        {#snippet aside()}
            <Toggle checked={showPreview} label={text.previewToggleLabel} onchange={(on) => onpreviewchange?.(on)} />
        {/snippet}
    </PanelRow>
    <PanelRow>
        {#snippet title()}{text.listTitle}{/snippet}
        {#snippet detail()}
            <code class="amml-versions__list">{GameVersionsSettingsScreenRules.listDetail({ versionSource, releaseCount, previewCount }, text)}</code>
        {/snippet}
        {#snippet aside()}
            <span class="amml-versions__aside">
                <span class="amml-versions__fetched">{fetched}</span>
                <Button variant="secondary" onclick={() => onrefreshversions?.()}>{text.refresh}</Button>
            </span>
        {/snippet}
    </PanelRow>
</Panel>

<Panel heading={GameVersionsSettingsScreenRules.installedHeading(installed, text)}>
    {#snippet aside()}<span class="amml-versions__folder">{versionsFolder}</span>{/snippet}
    {#each installed as version (version.version)}
        <InstalledVersionRow
            version={version.version}
            instanceCount={version.instanceCount}
            sizeText={Units.en.size(version.size ?? 0, { trim: true })}
            text={text.row}
            onremove={onremoveversion}
        />
    {/each}
</Panel>
<Note>{text.note}</Note>

<style>
    .amml-versions__list {
        display: block;
    }

    .amml-versions__aside {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .amml-versions__fetched {
        color: var(--color-rad-ok);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-versions__folder {
        display: block;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
