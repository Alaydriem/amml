<script lang="ts">
    import { page } from "$app/state";
    import InstanceDetailPage from "#lib/components/InstanceDetailPage.svelte";
</script>

<!-- a new name builds a new page: the store belongs to one instance -->
{#key page.params.name}
    <InstanceDetailPage name={page.params.name ?? ""} />
{/key}
