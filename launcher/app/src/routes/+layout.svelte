<script lang="ts">
    import "@amml/launcher-kit/css/tokens.css";
    import "@amml/launcher-kit/css/base.css";
    import { onDestroy, onMount, setContext, type Snippet } from "svelte";
    import { LauncherApi } from "#lib/api/LauncherApi.ts";
    import { RegistryEvents } from "#lib/api/RegistryEvents.ts";
    import { I18n } from "#lib/i18n/index.ts";
    import { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    let { children }: { children: Snippet } = $props();

    const shell = new ShellStore(LauncherApi, RegistryEvents);
    setContext(ShellStore.KEY, shell);

    // The pack of the window's language. No pack means English, so a failure here changes nothing the player sees.
    onMount(async () => {
        try {
            I18n.adopt(await LauncherApi.i18nLoad(navigator.language));
        } catch {
            I18n.adopt(null);
        }
        await shell.start();
    });

    onDestroy(() => shell.dispose());
</script>

{@render children()}
