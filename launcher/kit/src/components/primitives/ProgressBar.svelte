<script lang="ts">
    import { ProgressBarRules, type ProgressBarProps } from "./ProgressBar";

    let { value, label }: ProgressBarProps = $props();

    const percent = $derived(ProgressBarRules.clamp(value));
</script>

<div
    class="amml-progress"
    role="progressbar"
    aria-label={label}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={Math.round(percent)}
>
    <div class="amml-progress__fill" style:width="{percent}%"></div>
</div>

<style>
    .amml-progress {
        width: 100%;
        height: 6px;
        flex-shrink: 0;
        overflow: clip;
        background: var(--color-rad-line);
        border-radius: var(--radius-rad-pill);
    }

    .amml-progress__fill {
        height: 6px;
        background: var(--color-rad-brand-lift);
        border-radius: var(--radius-rad-pill);
    }
</style>
