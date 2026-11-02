<script lang="ts">
    import StatusMark from "../primitives/StatusMark.svelte";
    import { InstallStepRowRules, type InstallStepRowProps } from "./InstallStepRow";

    let { step, text }: InstallStepRowProps = $props();
</script>

<div
    class="amml-step-row amml-step-row--{step.state}"
    class:amml-row--selected={step.state === "active"}
    aria-current={step.state === "active" ? "step" : undefined}
>
    <StatusMark state={InstallStepRowRules.mark(step.state)} label={text[step.state]} />
    <span class="amml-step-row__label">{step.label}</span>
    <span class="amml-step-row__note">{step.note ?? ""}</span>
</div>

<style>
    .amml-step-row {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 11px 4px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-step-row--active,
    .amml-step-row--failed {
        padding-left: 10px;
    }

    .amml-step-row--failed {
        background: var(--color-rad-fault-wash);
        box-shadow: inset 3px 0 0 var(--color-rad-fault);
    }

    .amml-step-row__label {
        flex: 1;
        min-width: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .amml-step-row--active .amml-step-row__label,
    .amml-step-row--failed .amml-step-row__label {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-step-row--waiting .amml-step-row__label {
        color: var(--color-rad-faint);
    }

    .amml-step-row__note {
        flex-shrink: 0;
        width: 90px;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        text-align: right;
    }

    .amml-step-row--active .amml-step-row__note {
        color: var(--color-rad-brand-lift);
    }

    .amml-step-row--failed .amml-step-row__note {
        color: var(--color-rad-fault);
    }
</style>
