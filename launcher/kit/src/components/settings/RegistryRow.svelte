<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import Tag from "../primitives/Tag.svelte";
    import type { RegistryRowProps } from "./RegistryRow";

    let { id, name, url, status, statusTone = "ok", builtIn = false, text, onremove }: RegistryRowProps = $props();
</script>

<PanelRow>
    {#snippet title()}{name}{/snippet}
    {#snippet detail()}<code class="amml-registry-row__url">{url}</code>{/snippet}
    {#snippet aside()}
        <span class="amml-registry-row__aside">
            <span class="amml-registry-row__status amml-registry-row__status--{statusTone}">{status}</span>
            <span class="amml-registry-row__action">
                {#if builtIn}
                    <Tag bordered={false}>{text.builtIn}</Tag>
                {:else}
                    <Button variant="link" tone="fault" onclick={() => onremove?.(id)}>{text.remove}</Button>
                {/if}
            </span>
        </span>
    {/snippet}
</PanelRow>

<style>
    .amml-registry-row__url {
        display: block;
    }

    .amml-registry-row__aside {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .amml-registry-row__status {
        width: 150px;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        text-align: right;
    }

    .amml-registry-row__status--ok {
        color: var(--color-rad-ok);
    }

    .amml-registry-row__status--fault {
        color: var(--color-rad-fault);
    }

    .amml-registry-row__status--faint {
        color: var(--color-rad-faint);
    }

    .amml-registry-row__action {
        display: flex;
        justify-content: flex-end;
        width: 70px;
    }
</style>
