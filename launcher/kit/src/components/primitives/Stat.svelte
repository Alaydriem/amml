<script lang="ts">
    import Eyebrow from "./Eyebrow.svelte";
    import type { StatProps } from "./Stat";

    let { label, value, unit }: StatProps = $props();
</script>

<div class="amml-stat">
    <Eyebrow size="sm">{label}</Eyebrow>
    <div class="amml-stat__value">
        {value}{#if unit}<span class="amml-stat__unit"> {unit}</span>{/if}
    </div>
</div>

<style>
    .amml-stat {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 3px;
    }

    .amml-stat__value {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title);
        font-weight: var(--font-weight-rad-light);
        line-height: 30px;
    }

    .amml-stat__unit {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }
</style>
