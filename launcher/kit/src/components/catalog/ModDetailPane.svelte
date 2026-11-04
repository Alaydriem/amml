<script lang="ts">
    import KeyValueRow from "../primitives/KeyValueRow.svelte";
    import Skeleton from "../primitives/Skeleton.svelte";
    import Tile from "../primitives/Tile.svelte";
    import { ModDetailPaneRules, type ModDetailPaneProps } from "./ModDetailPane";

    let { detail, pending, text }: ModDetailPaneProps = $props();
</script>

<aside class="amml-detail" aria-busy={pending !== undefined}>
    {#if pending}
        <div class="amml-detail__head">
            <Tile size="detail" swatch={pending.card.swatch} src={pending.card.icon} />
            <div class="amml-detail__heading">
                <h2 class="amml-detail__title">{pending.card.title}</h2>
                <span class="amml-detail__id">
                    {ModDetailPaneRules.subline(pending.card.id, pending.registryName, text)}
                </span>
            </div>
        </div>
        <div class="amml-detail__lines" aria-hidden="true">
            <Skeleton tone="panel" radius="xs" />
            <Skeleton tone="panel" radius="xs" width="82%" />
            <Skeleton tone="panel" radius="xs" width="54%" />
        </div>
        <div class="amml-detail__rows">
            <KeyValueRow
                label={ModDetailPaneRules.versionLabel(pending.api, text)}
                value={pending.card.version}
                tone="strong"
            />
            <div class="amml-detail__pending-row">
                <span>{text.download}</span>
                <Skeleton tone="panel" radius="xs" height={10} width="96px" />
            </div>
            <div class="amml-detail__pending-row">
                <span>{text.source}</span>
                <Skeleton tone="panel" radius="xs" height={10} width="110px" />
            </div>
        </div>
        <p class="amml-detail__loading">{ModDetailPaneRules.loadingNote(pending, text)}</p>
    {:else if detail}
        <div class="amml-detail__head">
            <Tile size="detail" swatch={detail.swatch} src={detail.icon} />
            <div class="amml-detail__heading">
                <h2 class="amml-detail__title">{detail.title}</h2>
                <span class="amml-detail__id">{ModDetailPaneRules.subline(detail.id, detail.registryName, text)}</span>
            </div>
        </div>
        <p class="amml-detail__description">{detail.description ?? detail.summary}</p>
        <div class="amml-detail__rows">
            <KeyValueRow label={ModDetailPaneRules.versionLabel(detail.api, text)} value={detail.version} tone="strong" />
            <KeyValueRow label={text.download} value={ModDetailPaneRules.downloadValue(detail, text)} tone="dim" />
            <KeyValueRow label={text.source} value={detail.sourceLabel} tone="brand" />
        </div>
        <p class="amml-detail__note">{text.note}</p>
    {:else}
        <p class="amml-detail__empty">{text.empty}</p>
    {/if}
</aside>

<style>
    .amml-detail {
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        gap: 14px;
        width: 340px;
        padding: 20px 22px;
        box-sizing: border-box;
        overflow: clip;
        background: var(--color-rad-void-2);
        border-left: 1px solid var(--color-rad-line);
    }

    .amml-detail__head {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .amml-detail__heading {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .amml-detail__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title-md);
        font-weight: var(--font-weight-rad-strong);
        line-height: 24px;
        letter-spacing: var(--tracking-rad-title);
    }

    .amml-detail__id {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        line-height: 16px;
    }

    .amml-detail__description {
        margin: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 160%;
    }

    .amml-detail__lines {
        display: flex;
        flex-direction: column;
        gap: 8px;
        padding-top: 4px;
    }

    .amml-detail__rows {
        display: flex;
        flex-direction: column;
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-detail__pending-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding-block: 8px;
        border-bottom: 1px solid var(--color-rad-line);
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-detail__pending-row:last-child {
        border-bottom: 0;
    }

    .amml-detail__note {
        margin: 0;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-mono);
        line-height: 150%;
    }

    .amml-detail__loading {
        margin: 0;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        line-height: 16px;
    }

    .amml-detail__empty {
        margin: 0;
        padding-top: 24px;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 150%;
    }
</style>
