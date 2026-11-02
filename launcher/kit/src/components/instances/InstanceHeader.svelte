<script lang="ts">
    import BackLink from "../shell/BackLink.svelte";
    import Button from "../primitives/Button.svelte";
    import Tile from "../primitives/Tile.svelte";
    import RunningBadge from "./RunningBadge.svelte";
    import { InstanceHeaderRules, type InstanceHeaderProps } from "./InstanceHeader";

    let { instance, text, onback, ondelete, onopenfolder, onlaunch }: InstanceHeaderProps = $props();
</script>

<div class="amml-instance-header">
    <BackLink label={text.back} onclick={onback} />
    <div class="amml-instance-header__head">
        <Tile size="header" swatch={instance.swatch} />
        <div class="amml-instance-header__text">
            <h1 class="amml-instance-header__name">{instance.name}</h1>
            <span class="amml-instance-header__detail">{InstanceHeaderRules.detail(instance, text)}</span>
        </div>
        <Button variant="danger" onclick={() => ondelete?.(instance.id)}>{text.delete}</Button>
        <Button variant="secondary" onclick={() => onopenfolder?.(instance.id)}>{text.openFolder}</Button>
        {#if instance.running}
            <RunningBadge>{text.running}</RunningBadge>
        {:else}
            <Button variant="primary" caps icon="play" onclick={() => onlaunch?.(instance.id)}>{text.launch}</Button>
        {/if}
    </div>
</div>

<style>
    .amml-instance-header {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .amml-instance-header__head {
        display: flex;
        align-items: center;
        gap: 20px;
        padding-bottom: 22px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-instance-header__text {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .amml-instance-header__name {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title-lg);
        font-weight: var(--font-weight-rad-strong);
        letter-spacing: var(--tracking-rad-heading);
        line-height: 1.2;
    }

    .amml-instance-header__detail {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        letter-spacing: var(--tracking-rad-mono);
        line-height: 16px;
    }
</style>
