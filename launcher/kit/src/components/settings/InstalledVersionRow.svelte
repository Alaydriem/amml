<script lang="ts">
    import { Template } from "../../text/Template";
    import Button from "../primitives/Button.svelte";
    import PanelRow from "../primitives/PanelRow.svelte";
    import { InstalledVersionRowRules, type InstalledVersionRowProps } from "./InstalledVersionRow";

    let { version, instanceCount, sizeText, text, onremove }: InstalledVersionRowProps = $props();

    const removable = $derived(InstalledVersionRowRules.canRemove(instanceCount));
</script>

<PanelRow size="sm">
    {#snippet leading()}<span class="amml-installed-version__version">{version}</span>{/snippet}
    {#snippet title()}
        <span class="amml-installed-version__instances" class:amml-installed-version__instances--none={removable}>
            {instanceCount === 0 ? text.noInstances : Template.plural(text.instances, instanceCount)}
        </span>
    {/snippet}
    {#snippet aside()}
        <span class="amml-installed-version__aside">
            <span class="amml-installed-version__size">{sizeText}</span>
            <span class="amml-installed-version__action">
                <Button variant="link" tone="fault" disabled={!removable} onclick={() => removable && onremove?.(version)}>
                    {text.remove}
                </Button>
            </span>
        </span>
    {/snippet}
</PanelRow>

<style>
    .amml-installed-version__version {
        width: 120px;
        color: var(--color-rad-tx);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-installed-version__instances {
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-installed-version__instances--none {
        color: var(--color-rad-faint);
    }

    .amml-installed-version__aside {
        display: flex;
        align-items: center;
        gap: 20px;
    }

    .amml-installed-version__size {
        width: 80px;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-small);
        line-height: 18px;
        text-align: right;
    }

    .amml-installed-version__action {
        display: flex;
        justify-content: flex-end;
        width: 90px;
    }
</style>
