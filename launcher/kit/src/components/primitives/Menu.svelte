<script lang="ts">
    import { tick } from "svelte";
    import IconButton from "./IconButton.svelte";
    import { LayerStack } from "./LayerStack";
    import { MenuRules, type MenuProps } from "./Menu";

    let { items, label, onselect }: MenuProps = $props();

    let open = $state(false);
    let root: HTMLElement | undefined = $state();
    let entries: HTMLButtonElement[] = $state([]);

    $effect(() => {
        if (!open) return;
        const token = LayerStack.open(onkeydown);
        const onpointerdown = (event: PointerEvent) => {
            if (MenuRules.isOutside(root, event.target)) hide(false);
        };
        window.addEventListener("pointerdown", onpointerdown);
        return () => {
            LayerStack.close(token);
            window.removeEventListener("pointerdown", onpointerdown);
        };
    });

    async function show() {
        open = true;
        await tick();
        entries[MenuRules.first(items)]?.focus();
    }

    function hide(restoreFocus: boolean) {
        open = false;
        if (restoreFocus) root?.querySelector<HTMLButtonElement>(".amml-icon-btn")?.focus();
    }

    function choose(id: string) {
        hide(true);
        onselect?.(id);
    }

    function onkeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            event.preventDefault();
            hide(true);
            return;
        }
        if (event.key === "Tab") {
            hide(true);
            return;
        }
        const current = entries.findIndex((entry) => entry === document.activeElement);
        const target = MenuRules.target(items, event.key, current);
        if (target === null) return;
        event.preventDefault();
        entries[target]?.focus();
    }

    function onfocusout(event: FocusEvent) {
        if (open && event.relatedTarget instanceof Node && !root?.contains(event.relatedTarget)) hide(false);
    }
</script>

<div class="amml-menu" bind:this={root} {onfocusout}>
    <IconButton icon="more" {label} haspopup="menu" expanded={open} onclick={() => (open ? hide(false) : show())} />
    {#if open}
        <ul class="amml-menu__list" role="menu" aria-label={label}>
            {#each items as item, index (item.id)}
                {#if item.separated && index > 0}
                    <li class="amml-menu__divider" role="separator"></li>
                {/if}
                <li role="none">
                    <button
                        bind:this={entries[index]}
                        class="amml-menu__item"
                        class:amml-menu__item--fault={item.tone === "fault"}
                        type="button"
                        role="menuitem"
                        disabled={item.disabled}
                        onclick={() => choose(item.id)}
                    >
                        {item.label}
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<style>
    /* The parent may stretch the menu; the popup must still open under the button, so the menu never grows. */
    .amml-menu {
        position: relative;
        display: inline-flex;
        height: max-content;
    }

    .amml-menu__list {
        position: absolute;
        top: calc(100% + 4px);
        right: 0;
        z-index: 50;
        min-width: 180px;
        margin: 0;
        padding: 6px;
        list-style: none;
        background: var(--color-rad-panel);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-md);
        box-shadow: var(--rad-shadow-menu);
    }

    .amml-menu__divider {
        height: 1px;
        margin-block: 6px;
        background: var(--color-rad-line);
    }

    .amml-menu__item {
        display: block;
        width: 100%;
        padding: 9px 12px;
        border: 0;
        border-radius: var(--radius-rad-sm);
        background: none;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
        text-align: left;
        cursor: pointer;
    }

    .amml-menu__item:focus-visible {
        outline-offset: calc(var(--rad-focus-offset) * -1);
    }

    .amml-menu__item:hover:not(:disabled),
    .amml-menu__item:focus-visible {
        background: var(--color-rad-menu-hover);
        color: var(--color-rad-tx);
    }

    .amml-menu__item--fault,
    .amml-menu__item--fault:hover:not(:disabled) {
        color: var(--color-rad-fault);
    }

    .amml-menu__item:disabled {
        opacity: var(--opacity-rad-disabled);
        cursor: not-allowed;
    }
</style>
