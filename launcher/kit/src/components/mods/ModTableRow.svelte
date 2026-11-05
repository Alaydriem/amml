<script lang="ts">
    import { Template } from "../../text/Template";
    import Tag from "../primitives/Tag.svelte";
    import Menu from "../primitives/Menu.svelte";
    import Pill from "../primitives/Pill.svelte";
    import Toggle from "../primitives/Toggle.svelte";
    import { ModTableRowRules, type ModTableRowProps } from "./ModTableRow";

    let { mod, text, menuItems, ontoggle, onmenu, onupdate }: ModTableRowProps = $props();

    const controls = $derived(ModTableRowRules.controls(mod));
    const state = $derived(ModTableRowRules.state(mod));
    const newer = $derived(ModTableRowRules.hasUpdate(mod));
    const slots = $derived({ title: mod.title });
</script>

<div class="amml-mod-row" class:amml-mod-row--off={!controls.coreTag && !mod.enabled} role="row">
    <div class="amml-mod-row__on" role="cell">
        {#if controls.coreTag}
            <Tag>{text.coreTag}</Tag>
        {:else}
            <Toggle
                checked={mod.enabled}
                label={Template.fill(text.toggleLabel, slots)}
                onchange={(enabled) => ontoggle?.(mod.id, enabled)}
            />
        {/if}
    </div>
    <div class="amml-mod-row__name" role="cell">
        <span class="amml-mod-row__title">{mod.title}</span>
        <span class="amml-mod-row__id">{ModTableRowRules.subline(mod, text)}</span>
    </div>
    <div class="amml-mod-row__version amml-mod-row__version--installed" role="cell">
        {mod.installedVersion}
        {#if mod.localBuild}
            <Tag>{text.localBuild}</Tag>
        {/if}
    </div>
    <div class="amml-mod-row__version" class:amml-mod-row__version--newer={newer} role="cell">
        {mod.availableVersion ?? text.noVersion}
    </div>
    <div class="amml-mod-row__state" role="cell">
        {#if state === "update"}
            <Pill onclick={() => onupdate?.(mod.id)}>{#snippet children()}{text.update}{/snippet}</Pill>
        {:else if state === "unofficial"}
            <Pill tone="warn" tint>{#snippet children()}{text.unofficial}{/snippet}</Pill>
        {:else}
            <span class="amml-mod-row__current">{text.upToDate}</span>
        {/if}
    </div>
    <div class="amml-mod-row__menu" role="cell">
        {#if controls.menu}
            <Menu items={menuItems} label={Template.fill(text.menuLabel, slots)} onselect={(itemId) => onmenu?.(mod.id, itemId)} />
        {/if}
    </div>
</div>

<style>
    .amml-mod-row {
        display: flex;
        align-items: center;
        gap: 20px;
        padding: 12px 4px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-mod-row__on {
        display: flex;
        align-items: center;
        flex-shrink: 0;
        width: 48px;
    }

    .amml-mod-row__name {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .amml-mod-row__title {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-lead);
        font-weight: var(--font-weight-rad-strong);
        line-height: 22px;
    }

    .amml-mod-row--off .amml-mod-row__title {
        color: var(--color-rad-faint);
    }

    .amml-mod-row__id {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-mod-row__version {
        flex-shrink: 0;
        width: 110px;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-mod-row__version--installed {
        color: var(--color-rad-dim);
    }

    .amml-mod-row__version--newer {
        color: var(--color-rad-brand-lift);
    }

    .amml-mod-row__state {
        display: flex;
        flex-shrink: 0;
        width: 130px;
    }

    .amml-mod-row__current {
        padding-block: 7px;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-mod-row__menu {
        display: flex;
        flex-shrink: 0;
        justify-content: center;
        width: 32px;
    }
</style>
