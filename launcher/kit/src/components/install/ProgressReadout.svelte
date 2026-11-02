<script lang="ts">
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import ProgressBar from "../primitives/ProgressBar.svelte";
    import { Units } from "../../text/Units";
    import { ProgressReadoutRules, type ProgressReadoutProps } from "./ProgressReadout";

    let { eyebrow, progress, text }: ProgressReadoutProps = $props();

    const percent = $derived(Units.percent(progress.done, progress.total));
</script>

<div class="amml-readout">
    <Eyebrow size="lg">{eyebrow}</Eyebrow>
    <div class="amml-readout__figures">
        <span class="amml-readout__percent">{percent}<span class="amml-readout__sign">%</span></span>
        <span class="amml-readout__line">{ProgressReadoutRules.line(progress, text)}</span>
    </div>
    <ProgressBar value={percent} label={text.progressLabel} />
    {#if progress.detail}
        <span class="amml-readout__detail">{progress.detail}</span>
    {/if}
</div>

<style>
    .amml-readout {
        display: flex;
        flex-direction: column;
        gap: 18px;
        min-width: 0;
    }

    .amml-readout__figures {
        display: flex;
        align-items: baseline;
        gap: 14px;
    }

    .amml-readout__percent {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-readout);
        font-weight: var(--font-weight-rad-light);
        letter-spacing: var(--tracking-rad-display);
        line-height: 1;
    }

    .amml-readout__sign {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-percent);
    }

    .amml-readout__line {
        color: var(--color-rad-dim);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-readout__detail {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
