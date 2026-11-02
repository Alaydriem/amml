<script lang="ts">
    import Dialog from "../../../src/components/primitives/Dialog.svelte";
    import Menu from "../../../src/components/primitives/Menu.svelte";

    let {
        outerclose,
        innerclose,
        withInner = false,
        onselect,
    }: { outerclose?: () => void; innerclose?: () => void; withInner?: boolean; onselect?: (id: string) => void } = $props();

    let innerOpen = $state(true);
</script>

<Dialog open title="Outer" onclose={outerclose}>
    <Menu
        label="More"
        items={[
            { id: "open", label: "Open folder" },
            { id: "log", label: "Show log" },
            { id: "remove", label: "Remove", tone: "fault" },
        ]}
        {onselect}
    />
    {#snippet actions()}<button type="button">Outer action</button>{/snippet}
</Dialog>
{#if withInner}
    <Dialog
        open={innerOpen}
        title="Inner"
        onclose={() => {
            innerOpen = false;
            innerclose?.();
        }}
    >
        Inner body
    </Dialog>
{/if}
