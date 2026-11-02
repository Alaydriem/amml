<script lang="ts">
    import { Template } from "../../text/Template";
    import Button from "../primitives/Button.svelte";
    import Dialog from "../primitives/Dialog.svelte";
    import Note from "../primitives/Note.svelte";
    import RichText from "../primitives/RichText.svelte";
    import type { ConfirmDialogProps } from "./ConfirmDialog";

    let {
        open = false,
        inline = false,
        text,
        values = {},
        path,
        confirmVariant = "primary",
        onconfirm,
        oncancel,
    }: ConfirmDialogProps = $props();
</script>

<Dialog {open} {inline} eyebrow={text.eyebrow} title={Template.fill(text.title, values)} onclose={() => oncancel?.()}>
    <div class="amml-confirm__body"><RichText parts={Template.parts(text.body, values)} /></div>
    {#if path}
        <div class="amml-confirm__path">{path}</div>
    {/if}
    {#if text.note}
        <Note>{Template.fill(text.note, values)}</Note>
    {/if}
    {#snippet actions()}
        <Button variant="ghost" onclick={() => oncancel?.()}>{text.cancel}</Button>
        <Button variant={confirmVariant} onclick={() => onconfirm?.()}>{text.confirm}</Button>
    {/snippet}
</Dialog>

<style>
    .amml-confirm__body :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-confirm__path {
        padding: 9px 12px;
        background: var(--color-rad-void);
        border-radius: var(--radius-rad-md);
        color: var(--color-rad-tx);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
        overflow-wrap: anywhere;
    }
</style>
