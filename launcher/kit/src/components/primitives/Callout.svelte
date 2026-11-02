<script lang="ts">
    import { CalloutRules, type CalloutProps } from "./Callout";

    let { tone = "brand", ground, children }: CalloutProps = $props();

    const resolved = $derived(CalloutRules.ground(tone, ground));
</script>

<div class="amml-callout amml-callout--{tone} amml-callout--{resolved}">
    {@render children()}
</div>

<style>
    .amml-callout {
        padding: 14px 16px;
        border-left: 3px solid var(--amml-rule);
        border-radius: var(--radius-rad-card);
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 1.6;
    }

    .amml-callout :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-callout--brand {
        --amml-rule: var(--color-rad-brand);
    }

    .amml-callout--warn {
        --amml-rule: var(--color-rad-warn);
    }

    .amml-callout--ok {
        --amml-rule: var(--color-rad-ok);
    }

    .amml-callout--tint {
        background: var(--color-rad-brand-callout);
        border-radius: 0 var(--radius-rad-toggle) var(--radius-rad-toggle) 0;
    }

    .amml-callout--raised {
        background: var(--color-rad-void-2);
    }

    .amml-callout--base {
        background: var(--color-rad-void);
    }
</style>
