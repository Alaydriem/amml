<script lang="ts">
    import Checkbox from "../primitives/Checkbox.svelte";
    import Pill from "../primitives/Pill.svelte";
    import { VersionRowRules, type VersionRowProps } from "./VersionRow";

    let { version, text, selected = false, stop = true, onselect, onkeydown }: VersionRowProps = $props();

    const unsupported = $derived(version.state === "unsupported");
    const pill = $derived(VersionRowRules.pill(version, text));
</script>

<button
    class="amml-version-row amml-row"
    class:amml-row--selected={selected}
    class:amml-version-row--unsupported={unsupported}
    type="button"
    role="radio"
    aria-checked={selected}
    tabindex={stop ? 0 : -1}
    disabled={unsupported}
    onclick={() => onselect?.(version.version)}
    {onkeydown}
>
    <Checkbox decorative variant="marker" checked={selected} />
    <span class="amml-version-row__text">
        <span class="amml-version-row__version">{version.version}</span>
        {#if version.state === "installed"}
            <span class="amml-version-row__note">{text.installedNote}</span>
        {/if}
    </span>
    <span class="amml-version-row__state">
        <Pill tone={pill.tone} dot>{pill.label}</Pill>
    </span>
</button>

<style>
    .amml-version-row {
        display: flex;
        align-items: center;
        gap: 16px;
        width: 100%;
        padding: 13px 14px;
        border: 0;
        border-bottom: 1px solid var(--color-rad-line);
        background: none;
        color: inherit;
        font: inherit;
        text-align: left;
    }

    .amml-version-row--unsupported {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }

    .amml-version-row__text {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .amml-version-row__version {
        color: var(--color-rad-tx);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-lead);
        line-height: 22px;
    }

    .amml-version-row__note {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-version-row__state {
        display: flex;
        flex-shrink: 0;
        justify-content: flex-end;
        width: 170px;
    }
</style>
