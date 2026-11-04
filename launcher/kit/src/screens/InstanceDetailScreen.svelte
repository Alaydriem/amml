<script lang="ts">
    import ModTable from "../components/mods/ModTable.svelte";
    import { InstanceDetailScreenRules, type InstanceDetailScreenProps } from "./InstanceDetailScreen";

    let { mods, text, header, onupdateall, onadd, ontoggle, onmenu, onupdate }: InstanceDetailScreenProps = $props();

    const tableText = $derived(InstanceDetailScreenRules.tableText(text, mods));
</script>

<div class="amml-instance-detail">
    {@render header()}
    <ModTable {mods} text={tableText} menuItems={text.menu} {onupdateall} {onadd} {ontoggle} {onmenu} {onupdate} />
</div>

<style>
    .amml-instance-detail {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-height: 0;
        padding: 22px 40px 0;
    }
</style>
