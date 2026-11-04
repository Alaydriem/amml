<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import Callout from "../primitives/Callout.svelte";
    import Eyebrow from "../primitives/Eyebrow.svelte";
    import Segmented from "../primitives/Segmented.svelte";
    import TextField from "../primitives/TextField.svelte";
    import { Template } from "../../text/Template";
    import type { DataMode } from "../../model";
    import { NewInstanceFormRules, type NewInstanceFormProps } from "./NewInstanceForm";

    let {
        name,
        dataMode,
        takenNames,
        folder,
        canSubmit = true,
        problem,
        text,
        summary,
        onnamechange,
        ondatamodechange,
        oncreate,
        oncancel,
    }: NewInstanceFormProps = $props();

    const uid = $props.id();
    const allowed = $derived(canSubmit && !problem && NewInstanceFormRules.canCreate(name, takenNames));

    function submit() {
        if (!allowed) return;
        oncreate?.({ name: name.trim(), dataMode });
    }
</script>

<form
    class="amml-new-instance"
    onsubmit={(event) => {
        event.preventDefault();
        submit();
    }}
>
    <div class="amml-new-instance__group">
        <Eyebrow size="md" id="{uid}-name">{text.nameLabel}</Eyebrow>
        <TextField value={name} labelledby="{uid}-name" label={text.nameLabel} mono oninput={onnamechange} />
        <span class="amml-new-instance__hint">{Template.fill(text.folderHint, { folder })}</span>
        {#if problem}
            <span class="amml-new-instance__problem" role="alert">{problem}</span>
        {/if}
    </div>
    <div class="amml-new-instance__group">
        <Eyebrow size="md">{text.worldsLabel}</Eyebrow>
        <div class="amml-new-instance__segmented">
            <Segmented
                label={text.worldsLabel}
                options={[
                    { id: "shared", label: text.shared },
                    { id: "own", label: text.own },
                ]}
                value={dataMode}
                onchange={(id) => ondatamodechange?.(id as DataMode)}
            />
        </div>
    </div>
    <Callout tone="brand">{@render summary()}</Callout>
    <div class="amml-new-instance__actions">
        <Button variant="ghost" caps onclick={oncancel}>{text.cancel}</Button>
        <Button variant="primary" caps type="submit" disabled={!allowed}>{text.create}</Button>
    </div>
</form>

<style>
    .amml-new-instance {
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        gap: 18px;
        box-sizing: border-box;
        width: 400px;
        padding: 22px;
        background: var(--color-rad-void-2);
        border: 1px solid var(--color-rad-line);
        border-radius: var(--radius-rad-lg);
    }

    .amml-new-instance__group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .amml-new-instance__problem {
        color: var(--color-rad-warn);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-new-instance__hint {
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .amml-new-instance__segmented {
        align-self: flex-start;
    }

    .amml-new-instance__actions {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        padding-top: 4px;
    }
</style>
