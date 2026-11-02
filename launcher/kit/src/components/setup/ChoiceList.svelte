<script lang="ts">
    import Panel from "../primitives/Panel.svelte";
    import { RovingFocus } from "../primitives/RovingFocus";
    import ChoiceRow from "./ChoiceRow.svelte";
    import type { ChoiceListProps } from "./ChoiceList";

    let { heading, choices, value, onchange }: ChoiceListProps = $props();

    let group: HTMLElement | undefined = $state();

    const selected = $derived(choices.findIndex((choice) => choice.id === value));
    const stop = $derived(RovingFocus.stop(selected));

    function onkeydown(event: KeyboardEvent, index: number) {
        const target = RovingFocus.target(event.key, index, choices.length, { axis: "vertical" });
        if (target === null) return;
        event.preventDefault();
        const choice = choices[target];
        if (choice) onchange?.(choice.id);
        group?.querySelectorAll<HTMLElement>('[role="radio"]')[target]?.focus();
    }
</script>

<Panel {heading} roomy>
    <div bind:this={group} role="radiogroup" aria-label={heading}>
        {#each choices as choice, index (choice.id)}
            <ChoiceRow
                id={choice.id}
                title={choice.title}
                detail={choice.detail}
                meta={choice.meta}
                selected={index === selected}
                stop={index === stop}
                onselect={(id) => onchange?.(id)}
                onkeydown={(event) => onkeydown(event, index)}
            />
        {/each}
    </div>
</Panel>
