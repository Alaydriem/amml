<script lang="ts">
    import InstanceRow from "./InstanceRow.svelte";
    import type { InstanceListProps } from "./InstanceList";

    let { instances, text, menuItems, label, onlaunch, onopen, onmenu }: InstanceListProps = $props();
</script>

<ul class="amml-instance-list" aria-label={label}>
    {#each instances as instance, index (instance.id)}
        <li>
            <InstanceRow {instance} {text} {menuItems} emphasis={index === 0} {onlaunch} {onopen} {onmenu} />
        </li>
    {/each}
</ul>

<style>
    .amml-instance-list {
        margin: 0;
        padding: 0;
        list-style: none;
        border-top: 1px solid var(--color-rad-line);
    }
</style>
