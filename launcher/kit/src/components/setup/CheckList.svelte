<script lang="ts">
    import Panel from "../primitives/Panel.svelte";
    import CheckRow from "./CheckRow.svelte";
    import type { CheckListProps } from "./CheckList";

    let { heading, checks, text, onaction }: CheckListProps = $props();
</script>

<Panel {heading} roomy>
    {#each checks as check (check.id)}
        <CheckRow {check} {text} {onaction} />
    {/each}
</Panel>
