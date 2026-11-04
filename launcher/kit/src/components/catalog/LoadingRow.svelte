<script lang="ts">
    import Skeleton from "../primitives/Skeleton.svelte";
    import type { LoadingRowProps } from "./LoadingRow";
    import { ResultPaging } from "./ResultPaging";

    let { note }: LoadingRowProps = $props();
</script>

<div class="amml-loading-row" aria-hidden="true" style:height="{ResultPaging.ROW_HEIGHT}px">
    <span class="amml-loading-row__box"></span>
    <Skeleton width="28px" height={28} tone="soft" radius="seg" />
    <span class="amml-loading-row__lines">
        <Skeleton width="38%" height={12} />
        <Skeleton width="64%" height={10} tone="soft" />
    </span>
    <span class="amml-loading-row__note">{note ?? ""}</span>
</div>

<style>
    .amml-loading-row {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 0 26px 0 20px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-loading-row__box {
        flex-shrink: 0;
        width: 18px;
        height: 18px;
    }

    .amml-loading-row__lines {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 7px;
        min-width: 0;
    }

    .amml-loading-row__note {
        flex-shrink: 0;
        width: 170px;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        line-height: 16px;
        text-align: right;
    }
</style>
