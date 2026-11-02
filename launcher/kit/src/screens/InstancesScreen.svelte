<script lang="ts">
    import Button from "../components/primitives/Button.svelte";
    import EmptyState from "../components/primitives/EmptyState.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import InstanceList from "../components/instances/InstanceList.svelte";
    import UpdatesBanner from "../components/instances/UpdatesBanner.svelte";
    import PageHeader from "../components/shell/PageHeader.svelte";
    import { Template } from "../text/Template";
    import { InstancesScreenRules, type InstancesScreenProps } from "./InstancesScreen";

    let { instances, text, menuItems, onnew, onreview, onlaunch, onopen, onmenu }: InstancesScreenProps = $props();

    const totals = $derived(InstancesScreenRules.updateTotals(instances));
</script>

<div class="amml-screen">
    <div class="amml-screen__head">
        <PageHeader eyebrow={Template.fill(text.eyebrow, { count: instances.length })}>
            {#snippet title()}<RichText parts={Template.parts(text.title)} />{/snippet}
            {#snippet actions()}
                <Button variant="primary" icon="plus" onclick={onnew}>{text.newInstance}</Button>
            {/snippet}
        </PageHeader>
    </div>
    {#if instances.length === 0}
        <div class="amml-screen__empty">
            <EmptyState title={text.empty.title}>
                {#snippet body()}{text.empty.body}{/snippet}
            </EmptyState>
        </div>
    {:else}
        {#if totals.updates > 0}
            <div class="amml-screen__banner">
                <UpdatesBanner updates={totals.updates} instances={totals.instances} text={text.banner} {onreview} />
            </div>
        {/if}
        <div class="amml-screen__list">
            <InstanceList {instances} text={text.row} {menuItems} label={text.listLabel} {onlaunch} {onopen} {onmenu} />
        </div>
    {/if}
</div>

<style>
    .amml-screen {
        display: flex;
        flex: 1;
        flex-direction: column;
        min-height: 0;
        padding: 34px 40px 0;
    }

    .amml-screen__head {
        padding-bottom: 22px;
    }

    .amml-screen__banner {
        margin-bottom: 18px;
    }

    .amml-screen__empty {
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-screen__list {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }
</style>
