<script lang="ts">
    import { Template } from "../../text/Template";
    import Button from "../primitives/Button.svelte";
    import Callout from "../primitives/Callout.svelte";
    import Dialog from "../primitives/Dialog.svelte";
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import RichText from "../primitives/RichText.svelte";
    import TextField from "../primitives/TextField.svelte";
    import { AddModDialogRules, type AddModDialogProps } from "./AddModDialog";

    let {
        open = false,
        inline = false,
        text,
        values = {},
        value,
        knownIds,
        suggestions,
        oninput,
        onadd,
        oninstallurl,
        oncancel,
    }: AddModDialogProps = $props();

    const captionId = $props.id();
    const unofficial = $derived(AddModDialogRules.isUnofficial(value, knownIds));

    function install() {
        if (unofficial) oninstallurl?.(value.trim());
    }
</script>

<Dialog {open} {inline} title={Template.fill(text.title, values)} size="lg" onclose={() => oncancel?.()}>
    <div class="amml-add-mod__field">
        <Eyebrow size="sm" id={captionId}>{text.fieldLabel}</Eyebrow>
        <TextField {value} mono label={text.fieldLabel} labelledby={captionId} {oninput} onsubmit={install} />
    </div>
    {#if unofficial}
        <Callout tone="warn" ground="base"><RichText parts={Template.parts(text.unofficial)} /></Callout>
    {/if}
    {#if suggestions.length > 0}
        <div class="amml-add-mod__suggestions">
            <Eyebrow size="sm">{text.suggestionsLabel}</Eyebrow>
            {#each suggestions as suggestion (suggestion.id)}
                <div class="amml-add-mod__suggestion">
                    <div class="amml-add-mod__title">
                        {suggestion.title} <span class="amml-add-mod__id">{suggestion.id}</span>
                    </div>
                    <Button variant="link" onclick={() => onadd?.(suggestion.id)}>{text.add}</Button>
                </div>
            {/each}
        </div>
    {/if}
    {#snippet actions()}
        <Button variant="ghost" onclick={() => oncancel?.()}>{text.cancel}</Button>
        <Button variant="warn" disabled={!unofficial} onclick={install}>{text.install}</Button>
    {/snippet}
</Dialog>

<style>
    .amml-add-mod__field {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .amml-add-mod__suggestions {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .amml-add-mod__suggestion {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 9px 12px;
        background: var(--color-rad-void);
        border-radius: var(--radius-rad-md);
    }

    .amml-add-mod__title {
        flex: 1;
        min-width: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-add-mod__id {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
