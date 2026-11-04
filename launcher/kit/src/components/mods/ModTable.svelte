<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import { ModTableRules, type ModTableProps } from "./ModTable";
    import ModTableRow from "./ModTableRow.svelte";

    let { mods, text, menuItems, onupdateall, onadd, ontoggle, onmenu, onupdate }: ModTableProps = $props();

    const updates = $derived(ModTableRules.updateCount(mods));
</script>

<section class="amml-mod-table">
    <div class="amml-mod-table__bar">
        <Eyebrow>{text.sectionLabel}</Eyebrow>
        <div class="amml-mod-table__actions">
            <Button icon="arrow-up" disabled={updates === 0} onclick={onupdateall}>{text.updateAll}</Button>
            <Button variant="primary" icon="plus" onclick={onadd}>{text.addMod}</Button>
        </div>
    </div>
    <div class="amml-mod-table__grid" role="table" aria-label={text.tableLabel}>
        <div role="rowgroup">
            <div class="amml-mod-table__head" role="row">
                <div class="amml-mod-table__col amml-mod-table__col--on" role="columnheader">
                    <Eyebrow size="sm">{text.columns.on}</Eyebrow>
                </div>
                <div class="amml-mod-table__col amml-mod-table__col--name" role="columnheader">
                    <Eyebrow size="sm">{text.columns.name}</Eyebrow>
                </div>
                <div class="amml-mod-table__col amml-mod-table__col--version" role="columnheader">
                    <Eyebrow size="sm">{text.columns.installed}</Eyebrow>
                </div>
                <div class="amml-mod-table__col amml-mod-table__col--version" role="columnheader">
                    <Eyebrow size="sm">{text.columns.available}</Eyebrow>
                </div>
                <div class="amml-mod-table__col amml-mod-table__col--state" role="columnheader">
                    <span class="amml-visually-hidden">{text.columns.state}</span>
                </div>
                <div class="amml-mod-table__col amml-mod-table__col--menu" role="columnheader">
                    <span class="amml-visually-hidden">{text.columns.actions}</span>
                </div>
            </div>
        </div>
        <div class="amml-mod-table__rows" role="rowgroup">
            {#each mods as mod (mod.id)}
                <ModTableRow {mod} text={text.row} {menuItems} {ontoggle} {onmenu} {onupdate} />
            {/each}
        </div>
    </div>
</section>

<style>
    .amml-mod-table {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
    }

    .amml-mod-table__bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 20px 0 12px;
    }

    .amml-mod-table__actions {
        display: flex;
        gap: 10px;
    }

    .amml-mod-table__grid {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
    }

    .amml-mod-table__head {
        display: flex;
        align-items: center;
        gap: 20px;
        padding: 8px 4px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-mod-table__col {
        position: relative;
        flex-shrink: 0;
    }

    .amml-mod-table__col--on {
        width: 48px;
    }

    .amml-mod-table__col--name {
        flex: 1 1 0;
    }

    .amml-mod-table__col--version {
        width: 110px;
    }

    .amml-mod-table__col--state {
        width: 130px;
    }

    .amml-mod-table__col--menu {
        width: 32px;
    }

    .amml-mod-table__rows {
        flex: 1 1 0;
        min-height: 0;
        overflow-y: auto;
    }
</style>
