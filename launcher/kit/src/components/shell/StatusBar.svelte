<script lang="ts">
    import StatusDot from "../primitives/StatusDot.svelte";
    import { Template } from "../../text/Template";
    import type { StatusBarProps } from "./StatusBar";

    let { text, version, dataPath, message, storeVersion, storeTone = "ok" }: StatusBarProps = $props();
</script>

<footer class="amml-statusbar">
    <span class="amml-statusbar__text">{message ?? Template.fill(text.left, { version, dataPath })}</span>
    {#if storeVersion !== undefined}
        <StatusDot tone={storeTone}>{Template.fill(text.store, { version: storeVersion })}</StatusDot>
    {/if}
</footer>

<style>
    .amml-statusbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: var(--rad-statusbar-height);
        flex-shrink: 0;
        padding-inline: 30px;
        box-sizing: border-box;
        background: var(--color-rad-void);
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-statusbar__text {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        letter-spacing: var(--tracking-rad-status);
    }
</style>
