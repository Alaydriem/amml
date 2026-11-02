<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import StatusMark from "../primitives/StatusMark.svelte";
    import { CheckRowRules, type CheckRowProps } from "./CheckRow";

    let { check, text, onaction }: CheckRowProps = $props();
</script>

<div class="amml-check-row">
    <StatusMark state={CheckRowRules.mark(check.severity)} label={text[check.severity]} />
    <div class="amml-check-row__text">
        <span class="amml-check-row__title" class:amml-check-row__title--muted={check.severity === "muted"}>
            {check.title}
        </span>
        {#if check.detail}
            <span class="amml-check-row__detail">{check.detail}</span>
        {/if}
    </div>
    <div class="amml-check-row__value">
        {#if check.actionLabel}
            <Button
                variant="secondary"
                size="sm"
                icon={check.elevated ? "shield" : undefined}
                onclick={() => onaction?.(check.id)}
            >
                {check.actionLabel}
            </Button>
        {:else if check.value}
            <span
                class="amml-check-row__text-value"
                class:amml-check-row__text-value--ok={check.severity === "ok"}
                class:amml-check-row__text-value--mono={CheckRowRules.isMonoValue(check.value)}
            >
                {check.value}
            </span>
        {/if}
    </div>
</div>

<style>
    .amml-check-row {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 15px 18px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-check-row:last-child {
        border-bottom: 0;
    }

    .amml-check-row__text {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .amml-check-row__title {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-lead);
        line-height: 22px;
    }

    .amml-check-row__title--muted {
        color: var(--color-rad-dim);
    }

    .amml-check-row__detail {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }

    .amml-check-row__value {
        display: flex;
        flex-shrink: 0;
        justify-content: flex-end;
        width: 170px;
    }

    .amml-check-row__text-value {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-check-row__text-value--ok {
        color: var(--color-rad-ok);
    }

    .amml-check-row__text-value--mono {
        font-family: var(--font-rad-mono);
    }
</style>
