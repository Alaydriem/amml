<script lang="ts">
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import { Template } from "../../text/Template";
    import type { StepIndicatorProps } from "./StepIndicator";

    let { total, current, template }: StepIndicatorProps = $props();

    const bars = $derived(Array.from({ length: Math.max(0, total) }, (_, index) => index + 1));
</script>

<div class="amml-steps">
    {#each bars as step (step)}
        <span class="amml-steps__bar" class:amml-steps__bar--on={step <= current} aria-hidden="true"></span>
    {/each}
    <span class="amml-steps__text"><Eyebrow size="md">{Template.fill(template, { current, total })}</Eyebrow></span>
</div>

<style>
    .amml-steps {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .amml-steps__bar {
        width: 28px;
        height: 4px;
        flex-shrink: 0;
        border-radius: var(--radius-rad-2xs);
        background: var(--color-rad-line-2);
    }

    .amml-steps__bar--on {
        background: var(--color-rad-brand-lift);
    }

    .amml-steps__text {
        padding-left: 6px;
    }
</style>
