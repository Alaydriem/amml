<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import type { WizardFooterProps } from "./WizardFooter";

    let { text, primaryDisabled = false, onsecondary, onprimary }: WizardFooterProps = $props();
</script>

<footer class="amml-wizard-footer">
    <span class="amml-wizard-footer__note">{text.note ?? ""}</span>
    <div class="amml-wizard-footer__actions">
        {#if text.secondary}
            <Button variant="ghost" caps size="lg" onclick={onsecondary}>{text.secondary}</Button>
        {/if}
        <Button variant="primary" caps size="lg" disabled={primaryDisabled} onclick={onprimary}>{text.primary}</Button>
    </div>
</footer>

<style>
    .amml-wizard-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 24px;
        height: 80px;
        flex-shrink: 0;
        padding-inline: 30px;
        box-sizing: border-box;
        background: var(--color-rad-void);
        border-top: 1px solid var(--color-rad-line);
    }

    .amml-wizard-footer__note {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-wizard-footer__actions {
        display: flex;
        gap: 10px;
    }
</style>
