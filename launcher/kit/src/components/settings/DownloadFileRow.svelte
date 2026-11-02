<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import Tag from "../primitives/Tag.svelte";
    import type { DownloadFileRowProps } from "./DownloadFileRow";

    let { id, name, detail: detailText, sizeText, kept = false, text, onremove }: DownloadFileRowProps = $props();
</script>

<PanelRow size="sm">
    {#snippet title()}<span class="amml-file-row__name">{name}</span>{/snippet}
    {#snippet detail()}<span class="amml-file-row__detail">{detailText}</span>{/snippet}
    {#snippet aside()}
        <span class="amml-file-row__aside">
            <span class="amml-file-row__size">{sizeText}</span>
            <span class="amml-file-row__action">
                {#if kept}
                    <Tag tone="ok" bordered={false}>{text.kept}</Tag>
                {:else}
                    <Button variant="link" tone="fault" onclick={() => onremove?.(id)}>{text.remove}</Button>
                {/if}
            </span>
        </span>
    {/snippet}
</PanelRow>

<style>
    .amml-file-row__name {
        display: block;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-file-row__detail {
        font-size: var(--text-rad-small);
    }

    .amml-file-row__aside {
        display: flex;
        align-items: center;
        gap: 20px;
    }

    .amml-file-row__size {
        width: 90px;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-small);
        line-height: 18px;
        text-align: right;
    }

    .amml-file-row__action {
        display: flex;
        justify-content: flex-end;
        width: 100px;
    }
</style>
