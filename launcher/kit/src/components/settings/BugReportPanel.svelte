<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import KeyValueRow from "../primitives/KeyValueRow.svelte";
    import Panel from "../primitives/Panel.svelte";
    import { BugReportPanelRules, type BugReportPanelProps } from "./BugReportPanel";

    let { text, lines, oncopy, onopenlink }: BugReportPanelProps = $props();
</script>

<Panel heading={text.heading}>
    {#snippet aside()}
        <Button variant="link" onclick={() => oncopy?.(BugReportPanelRules.report(lines, text))}>{text.copy}</Button>
    {/snippet}
    <div class="amml-bug__rows">
        {#each lines as line (line.id)}
            {@const href = line.href}
            <KeyValueRow
                label={text.labels[line.id]}
                value={BugReportPanelRules.shown(line, text)}
                tone={href === undefined ? "dim" : "brand"}
                onopen={href === undefined ? undefined : () => onopenlink?.(href)}
            />
        {/each}
    </div>
</Panel>

<style>
    .amml-bug__rows {
        display: flex;
        flex-direction: column;
        padding-inline: 18px;
    }
</style>
