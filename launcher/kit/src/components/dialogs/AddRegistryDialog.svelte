<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import Callout from "../primitives/Callout.svelte";
    import Dialog from "../primitives/Dialog.svelte";
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import TextField from "../primitives/TextField.svelte";
    import { AddRegistryDialogRules, type AddRegistryDialogProps } from "./AddRegistryDialog";

    let { open = false, inline = false, text, value, problem, oninput, onadd, oncancel }: AddRegistryDialogProps = $props();

    const captionId = $props.id();
    const allowed = $derived(AddRegistryDialogRules.canAdd(value, problem));
    const shown = $derived(problem ?? (AddRegistryDialogRules.isInvalid(value) ? text.invalid : undefined));

    function add() {
        if (allowed) onadd?.(value.trim());
    }
</script>

<Dialog {open} {inline} title={text.title} size="lg" onclose={() => oncancel?.()}>
    <div class="amml-add-registry__field">
        <Eyebrow size="sm" id={captionId}>{text.fieldLabel}</Eyebrow>
        <TextField {value} mono label={text.fieldLabel} labelledby={captionId} placeholder={text.placeholder} {oninput} onsubmit={add} />
    </div>
    {#if shown}
        <Callout tone="warn" ground="base">{shown}</Callout>
    {/if}
    {#snippet actions()}
        <Button variant="ghost" onclick={() => oncancel?.()}>{text.cancel}</Button>
        <Button variant="primary" disabled={!allowed} onclick={add}>{text.add}</Button>
    {/snippet}
</Dialog>

<style>
    .amml-add-registry__field {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
</style>
