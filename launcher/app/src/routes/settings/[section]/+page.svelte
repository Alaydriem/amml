<script lang="ts">
    import { page } from "$app/state";
    import SettingsPage from "#lib/components/SettingsPage.svelte";
</script>

<SettingsPage section={page.params.section} />
