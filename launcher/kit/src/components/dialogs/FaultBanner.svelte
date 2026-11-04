<script lang="ts">
    import Banner from "../primitives/Banner.svelte";
    import Button from "../primitives/Button.svelte";
    import type { FaultBannerProps } from "./FaultBanner";

    let { fault, text, onaction }: FaultBannerProps = $props();
</script>

{#snippet button()}
    <Button variant={fault.primaryAction ? "primary" : "secondary"} onclick={() => onaction?.(fault.kind)}>
        {text.action}
    </Button>
{/snippet}

<Banner tone={fault.severity} action={onaction ? button : undefined}>
    <strong>{text.title}</strong>
    {text.body}
    {#if fault.code}
        <code class="amml-fault__code">{fault.code}</code>
    {/if}
</Banner>

<style>
    .amml-fault__code.amml-fault__code {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-mono);
    }
</style>
