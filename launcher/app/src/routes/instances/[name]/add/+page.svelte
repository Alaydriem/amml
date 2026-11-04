<script lang="ts">
    import { page } from "$app/state";
    import CatalogPage from "#lib/components/CatalogPage.svelte";
</script>

{#key page.params.name}
    <CatalogPage name={page.params.name ?? ""} />
{/key}
