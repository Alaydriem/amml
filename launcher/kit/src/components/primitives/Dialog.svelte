<script lang="ts">
    import Eyebrow from "./Eyebrow.svelte";
    import { DialogFocus, type DialogProps } from "./Dialog";
    import { LayerStack } from "./LayerStack";

    let { open = false, title, eyebrow, size = "md", inline = false, onclose, children, actions }: DialogProps = $props();

    let card: HTMLElement | undefined = $state();
    const titleId = $props.id();
    const visible = $derived(inline || open);

    $effect(() => {
        if (inline || !open) return;
        const token = LayerStack.open(onkeydown);
        return () => LayerStack.close(token);
    });

    $effect(() => {
        if (inline || !open || !card) return;
        const previous = document.activeElement;
        const first = DialogFocus.within(card)[0];
        (first ?? card).focus();
        return () => {
            if (previous instanceof HTMLElement) previous.focus();
        };
    });

    function onkeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            event.preventDefault();
            onclose?.();
            return;
        }
        if (event.key === "Tab" && card && !inline) {
            const next = DialogFocus.wrap(card, document.activeElement, event.shiftKey);
            if (next) {
                event.preventDefault();
                next.focus();
            }
        }
    }
</script>

{#if visible}
    {#snippet cardBody()}
        <div
            bind:this={card}
            class="amml-dialog amml-dialog--{size}"
            role="dialog"
            aria-modal={inline ? undefined : "true"}
            aria-labelledby={titleId}
            tabindex="-1"
            onkeydown={inline ? onkeydown : undefined}
        >
            {#if eyebrow}
                <Eyebrow size="sm" tone="warn">{eyebrow}</Eyebrow>
            {/if}
            <h2 class="amml-dialog__title" id={titleId}>{title}</h2>
            <div class="amml-dialog__body">{@render children()}</div>
            {#if actions}
                <div class="amml-dialog__actions">{@render actions()}</div>
            {/if}
        </div>
    {/snippet}

    {#if inline}
        {@render cardBody()}
    {:else}
        <div class="amml-dialog-scrim" role="presentation" onpointerdown={(event) => event.target === event.currentTarget && onclose?.()}>
            {@render cardBody()}
        </div>
    {/if}
{/if}

<style>
    .amml-dialog-scrim {
        position: fixed;
        inset: 0;
        z-index: 100;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--color-rad-scrim);
    }

    .amml-dialog {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 24px;
        background: var(--color-rad-void-2);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-xl);
        box-shadow: var(--rad-shadow-dialog);
        box-sizing: border-box;
    }

    .amml-dialog--md {
        width: 392px;
    }

    .amml-dialog--lg {
        width: 448px;
        gap: 14px;
    }

    .amml-dialog__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-title-md);
        font-weight: var(--font-weight-rad-strong);
        line-height: 24px;
        letter-spacing: var(--tracking-rad-title);
    }

    .amml-dialog__body {
        display: flex;
        flex-direction: column;
        gap: 12px;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 1.6;
    }

    .amml-dialog--lg .amml-dialog__body {
        gap: 14px;
    }

    .amml-dialog__body :global(strong) {
        color: var(--color-rad-tx);
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-dialog__actions {
        display: flex;
        justify-content: flex-end;
        gap: 9px;
        padding-top: 10px;
    }
</style>
