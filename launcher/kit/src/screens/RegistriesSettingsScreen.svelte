<script lang="ts">
    import Button from "../components/primitives/Button.svelte";
    import Note from "../components/primitives/Note.svelte";
    import Panel from "../components/primitives/Panel.svelte";
    import RegistryRow from "../components/settings/RegistryRow.svelte";
    import { RegistriesSettingsScreenRules, type RegistriesSettingsScreenProps } from "./RegistriesSettingsScreen";

    let { registries, text, onaddregistry, onremoveregistry }: RegistriesSettingsScreenProps = $props();
</script>

<Panel heading={text.heading}>
    {#snippet aside()}
        <Button variant="link" onclick={() => onaddregistry?.()}>{text.add}</Button>
    {/snippet}
    {#each registries as registry (registry.id)}
        {@const status = RegistriesSettingsScreenRules.status(registry, text)}
        <RegistryRow
            id={registry.id}
            name={registry.name}
            url={registry.url}
            status={status.text}
            statusTone={status.tone}
            builtIn={registry.builtIn}
            text={text.row}
            onremove={onremoveregistry}
        />
    {/each}
</Panel>
<Note>{text.note}</Note>
