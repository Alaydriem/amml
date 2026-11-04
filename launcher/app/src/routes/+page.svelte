<script lang="ts">
    import { goto } from "$app/navigation";
    import { getContext, onMount } from "svelte";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    const shell = getContext<ShellStore>(ShellStore.KEY);

    // the first run starts with the checks; later runs start at the instance list
    onMount(async () => {
        await goto(await shell.landing(), { replace: true });
    });
</script>
