<script lang="ts">
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import Menu from "../primitives/Menu.svelte";
    import Pill from "../primitives/Pill.svelte";
    import StatusDot from "../primitives/StatusDot.svelte";
    import Tag from "../primitives/Tag.svelte";
    import Tile from "../primitives/Tile.svelte";
    import Button from "../primitives/Button.svelte";
    import RunningBadge from "./RunningBadge.svelte";
    import { InstanceRowRules, type InstanceRowProps } from "./InstanceRow";

    let { instance, text, menuItems, emphasis = false, onlaunch, onopen, onmenu }: InstanceRowProps = $props();

    const updateLabel = $derived(InstanceRowRules.updateLabel(instance, text));
</script>

<div class="amml-instance-row">
    <button class="amml-instance-row__open" type="button" onclick={() => onopen?.(instance.id)}>
        <Tile size="rail" swatch={instance.swatch} />
        <span class="amml-instance-row__text">
            <span class="amml-instance-row__name">{instance.name}</span>
            <span class="amml-instance-row__activity">{InstanceRowRules.activityLine(instance, text)}</span>
        </span>
    </button>
    <div class="amml-instance-row__stat amml-instance-row__stat--game">
        <Eyebrow size="sm">{text.game}</Eyebrow>
        <span class="amml-instance-row__value">{instance.gameVersion}</span>
    </div>
    <div class="amml-instance-row__stat amml-instance-row__stat--amml">
        <Eyebrow size="sm">{text.amml}</Eyebrow>
        <span class="amml-instance-row__value">{instance.ammlVersion}</span>
        {#if instance.ammlLocal}
            <Tag>{text.localBuild}</Tag>
        {/if}
    </div>
    <div class="amml-instance-row__stat amml-instance-row__stat--mods">
        <Eyebrow size="sm">{text.mods}</Eyebrow>
        <span class="amml-instance-row__value">{instance.modCount}</span>
    </div>
    <div class="amml-instance-row__update">
        {#if updateLabel}
            <Pill tone="brand" dot tint>{updateLabel}</Pill>
        {:else}
            <StatusDot tone="ok" size="sm">{text.upToDate}</StatusDot>
        {/if}
    </div>
    <div class="amml-instance-row__launch">
        {#if instance.running}
            <RunningBadge block>{text.running}</RunningBadge>
        {:else if emphasis}
            <Button variant="launch" icon="play" block onclick={() => onlaunch?.(instance.id)}>{text.launch}</Button>
        {:else}
            <Button variant="secondary" caps size="sm" icon="play" block onclick={() => onlaunch?.(instance.id)}>
                {text.launch}
            </Button>
        {/if}
    </div>
    <div class="amml-instance-row__menu">
        <Menu items={menuItems} label={text.menuLabel} onselect={(itemId) => onmenu?.(instance.id, itemId)} />
    </div>
</div>

<style>
    .amml-instance-row {
        display: flex;
        align-items: center;
        gap: 20px;
        padding: 16px 4px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-instance-row__open {
        display: flex;
        flex: 1;
        align-items: center;
        gap: 20px;
        min-width: 0;
        padding: 0;
        border: 0;
        background: none;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
    }

    .amml-instance-row__text {
        display: flex;
        flex-direction: column;
        gap: 3px;
        min-width: 0;
    }

    .amml-instance-row__name {
        overflow: hidden;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title-sm);
        font-weight: var(--font-weight-rad-strong);
        letter-spacing: var(--tracking-rad-row);
        line-height: 24px;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .amml-instance-row__activity {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        letter-spacing: var(--tracking-rad-mono);
        line-height: 16px;
    }

    .amml-instance-row__stat {
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        gap: 3px;
    }

    .amml-instance-row__stat--game {
        width: 120px;
    }

    .amml-instance-row__stat--amml {
        width: 110px;
    }

    .amml-instance-row__stat--mods {
        width: 70px;
    }

    .amml-instance-row__value {
        color: var(--color-rad-dim);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-instance-row__update {
        display: flex;
        flex-shrink: 0;
        align-items: center;
        width: 130px;
    }

    .amml-instance-row__launch {
        flex-shrink: 0;
        width: 116px;
    }

    .amml-instance-row__menu {
        display: flex;
        flex-shrink: 0;
        justify-content: center;
        width: 32px;
    }
</style>
