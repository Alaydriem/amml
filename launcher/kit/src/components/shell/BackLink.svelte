<script lang="ts">
    import Icon from "../primitives/Icon.svelte";
    import type { BackLinkProps } from "./BackLink";

    let { label, onclick }: BackLinkProps = $props();
</script>

<button class="amml-back" type="button" onclick={() => onclick?.()}>
    <Icon name="chevron-left" size={14} weight={2.2} />
    {label}
</button>

<style>
    .amml-back {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 0;
        border: 0;
        background: none;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        cursor: pointer;
    }

    .amml-back:hover {
        color: var(--color-rad-tx);
    }
</style>
