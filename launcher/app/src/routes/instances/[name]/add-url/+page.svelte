<script lang="ts">
    import { page } from "$app/state";
    import UrlInstallPage from "#lib/components/UrlInstallPage.svelte";
</script>

{#key page.params.name}
    <UrlInstallPage name={page.params.name ?? ""} />
{/key}
