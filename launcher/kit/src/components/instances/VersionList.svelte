<script lang="ts">
    import { RovingFocus } from "../primitives/RovingFocus";
    import VersionRow from "./VersionRow.svelte";
    import { VersionListRules, type VersionListProps } from "./VersionList";

    let { versions, text, label, value, onchange }: VersionListProps = $props();

    let group: HTMLElement | undefined = $state();

    const selected = $derived(versions.findIndex((version) => version.version === value));
    const stop = $derived(VersionListRules.stop(versions, selected));

    function onkeydown(event: KeyboardEvent, index: number) {
        const target = RovingFocus.target(event.key, index, versions.length, {
            axis: "vertical",
            disabled: (at) => VersionListRules.disabled(versions[at]),
        });
        if (target === null) return;
        event.preventDefault();
        const version = versions[target];
        if (version) onchange?.(version.version);
        group?.querySelectorAll<HTMLElement>('[role="radio"]')[target]?.focus();
    }
</script>

<div bind:this={group} class="amml-version-list" role="radiogroup" aria-label={label}>
    {#each versions as version, index (version.version)}
        <VersionRow
            {version}
            {text}
            selected={index === selected}
            stop={index === stop}
            onselect={(next) => onchange?.(next)}
            onkeydown={(event) => onkeydown(event, index)}
        />
    {/each}
</div>

<style>
    .amml-version-list {
        display: flex;
        flex-direction: column;
        border-top: 1px solid var(--color-rad-line);
    }
</style>
