<script lang="ts">
    import { Units } from "../text/Units";
    import NewInstanceForm from "../components/instances/NewInstanceForm.svelte";
    import VersionList from "../components/instances/VersionList.svelte";
    import Eyebrow from "../components/primitives/Eyebrow.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import Segmented from "../components/primitives/Segmented.svelte";
    import BackLink from "../components/shell/BackLink.svelte";
    import PageHeader from "../components/shell/PageHeader.svelte";
    import { Template } from "../text/Template";
    import type { Channel } from "../model";
    import { NewInstanceScreenRules, type NewInstanceScreenProps } from "./NewInstanceScreen";

    let {
        versions,
        takenNames,
        summary: contents,
        text,
        channel,
        selectedVersion,
        name,
        dataMode,
        onchannelchange,
        onversionchange,
        onnamechange,
        ondatamodechange,
        onback,
        oncancel,
        oncreate,
    }: NewInstanceScreenProps = $props();

    const visible = $derived(NewInstanceScreenRules.visible(versions, channel));
    const selectable = $derived(NewInstanceScreenRules.selectable(versions, channel, selectedVersion));
    const folder = $derived(Template.fill(text.folderPath, { name: name.trim() }));
    const installs = $derived(
        Template.parts(text.installs, {
            amml: Template.fill(text.amml, { version: contents.ammlVersion }),
            mods: Template.plural(text.mods, contents.defaultMods),
        }),
    );
    const disk = $derived(Template.fill(text.disk, { size: Units.en.size(contents.diskBytes, { digits: 1 }) }));
</script>

<div class="amml-screen">
    <div class="amml-screen__back">
        <BackLink label={text.back} onclick={onback} />
    </div>
    <div class="amml-screen__title">
        <PageHeader>
            {#snippet title()}<RichText parts={Template.parts(text.title)} />{/snippet}
        </PageHeader>
    </div>
    <div class="amml-screen__columns">
        <div class="amml-screen__versions">
            <div class="amml-screen__versions-head">
                <Eyebrow size="lg">{text.versionLabel}</Eyebrow>
                <Segmented
                    label={text.versionLabel}
                    options={[
                        { id: "release", label: text.release },
                        { id: "preview", label: text.preview },
                    ]}
                    value={channel}
                    onchange={(id) => onchannelchange?.(id as Channel)}
                />
            </div>
            <div class="amml-screen__scroll">
                <VersionList
                    versions={visible}
                    text={text.versionRow}
                    label={text.versionsLabel}
                    value={selectedVersion}
                    onchange={onversionchange}
                />
            </div>
        </div>
        <div class="amml-screen__form">
            <NewInstanceForm
                {name}
                {dataMode}
                {takenNames}
                {folder}
                canSubmit={selectable}
                text={text.form}
                {onnamechange}
                {ondatamodechange}
                {oncancel}
                oncreate={(request) => {
                    if (selectedVersion !== undefined) oncreate?.({ ...request, version: selectedVersion });
                }}
            >
                {#snippet summary()}
                    <p class="amml-screen__summary"><RichText parts={installs} /></p>
                    <p class="amml-screen__disk">{disk}</p>
                {/snippet}
            </NewInstanceForm>
        </div>
    </div>
</div>

<style>
    .amml-screen {
        display: flex;
        flex: 1;
        flex-direction: column;
        min-height: 0;
        padding: 22px 40px 0;
    }

    .amml-screen__back {
        padding-bottom: 14px;
    }

    .amml-screen__title {
        padding-bottom: 22px;
    }

    .amml-screen__columns {
        display: flex;
        flex: 1;
        gap: 32px;
        min-height: 0;
    }

    .amml-screen__versions {
        display: flex;
        flex: 1;
        flex-direction: column;
        min-width: 0;
    }

    .amml-screen__versions-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-bottom: 12px;
    }

    .amml-screen__scroll {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }

    .amml-screen__form {
        align-self: flex-start;
        flex-shrink: 0;
    }

    .amml-screen__summary,
    .amml-screen__disk {
        margin: 0;
    }

    .amml-screen__summary + .amml-screen__disk {
        margin-top: 6px;
        color: var(--color-rad-faint);
    }
</style>
