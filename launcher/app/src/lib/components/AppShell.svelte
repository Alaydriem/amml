<script lang="ts">
    import { goto } from "$app/navigation";
    import { AppFrame, FaultBanner, StatusBar, TitleBar } from "@amml/launcher-kit";
    import type { Snippet } from "svelte";
    import type { PresentedFault } from "#lib/present/FaultPresenter.ts";
    import type { FaultKindView } from "#lib/bindings/FaultKindView.ts";
    import type { ShellStore } from "#lib/stores/ShellStore.svelte.ts";

    let {
        shell,
        steps,
        fault,
        onfault,
        children,
    }: {
        shell: ShellStore;
        steps?: { total: number; current: number };
        fault?: PresentedFault | null;
        onfault?: (kind: FaultKindView) => void;
        children: Snippet;
    } = $props();
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar
            text={shell.texts.titleBar}
            {steps}
            registryFetched={shell.registryFetched()}
            onrefresh={() => void shell.refresh()}
            onsettings={() => void goto("/settings/registries")}
        />
    {/snippet}
    {#snippet banner()}
        {#if fault}
            <FaultBanner fault={fault.fault} text={fault.text} onaction={() => onfault?.(fault.kind)} />
        {/if}
    {/snippet}
    {@render children()}
    {#snippet statusbar()}
        <StatusBar
            text={shell.texts.statusBar}
            version={shell.launcherVersion}
            dataPath={shell.dataPath}
            message={shell.message}
            storeVersion={shell.storeVersion}
        />
    {/snippet}
</AppFrame>
