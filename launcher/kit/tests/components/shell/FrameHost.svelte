<script lang="ts">
    import AppFrame from "../../../src/components/shell/AppFrame.svelte";
</script>

<AppFrame>
    {#snippet titlebar()}<div data-testid="title">title</div>{/snippet}
    {#snippet banner()}<div data-testid="banner">banner</div>{/snippet}
    <div data-testid="page">page</div>
</AppFrame>
