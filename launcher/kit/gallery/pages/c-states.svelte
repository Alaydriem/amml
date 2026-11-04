<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "09 Interaction states", group: "Screens", order: 20 };
</script>

<script lang="ts">
    import { ModDetailPane } from "../../src/components/catalog";
    import { Button, Checkbox, Eyebrow, Menu } from "../../src/components/primitives";
    import { CatalogFixtures } from "../../src/fixtures/catalog";
    import { en } from "../../src/text/en";
    import type { ButtonProps } from "../../src/components/primitives";

    type Row = { label: string; variant: NonNullable<ButtonProps["variant"]>; text: string };

    const rows: Row[] = [
        { label: "Primary", variant: "primary", text: "New instance" },
        { label: "Secondary", variant: "secondary", text: "Open folder" },
        { label: "Ghost", variant: "ghost", text: "Show log" },
    ];

    const columns = ["rest", "hover", "focus", "disabled"] as const;

    const listStates = [
        { id: "rest", note: "rest", checked: false, selected: false },
        { id: "hover", note: "hover", checked: false, selected: false },
        { id: "selected", note: "selected", checked: true, selected: true },
        { id: "focus", note: "keyboard focus", checked: false, selected: false },
    ];

    let menuHost: HTMLElement | undefined = $state();

    // Opens the real menu once, so the sheet shows its popup.
    $effect(() => {
        menuHost?.querySelector<HTMLButtonElement>(".amml-icon-btn")?.click();
    });
</script>

<section class="sheet">
    <header class="head">
        <Eyebrow size="lg">Launcher v1 · States</Eyebrow>
        <h1 class="title">Hover, focus <strong>and loading</strong></h1>
    </header>

    <div class="block">
        <Eyebrow size="md">Buttons · rest, hover, keyboard focus, disabled</Eyebrow>
        {#each rows as row (row.variant)}
            <div class="button-row">
                <span class="row-name">{row.label}</span>
                {#each columns as column (column)}
                    <span class="state state--{column} state--{row.variant}">
                        <Button variant={row.variant} size="lg" disabled={column === "disabled"}>{row.text}</Button>
                    </span>
                {/each}
            </div>
        {/each}
    </div>

    <div class="panes">
        <div class="block list-block">
            <Eyebrow size="md">List rows</Eyebrow>
            <div class="list">
                {#each listStates as state (state.id)}
                    <div
                        class="list-row amml-row state-row state-row--{state.id}"
                        class:amml-row--selected={state.selected}
                    >
                        <Checkbox decorative checked={state.checked} />
                        <span class="list-name">Coords Plus</span>
                        <span class="list-note">{state.note}</span>
                    </div>
                {/each}
            </div>
        </div>

        <div class="block menu-block">
            <Eyebrow size="md">Row menu, open</Eyebrow>
            <div class="menu-host" bind:this={menuHost}>
                <Menu
                    label="More actions"
                    items={[
                        { id: "open", label: "Open folder" },
                        { id: "log", label: "Show log" },
                        { id: "remove", label: "Remove", tone: "fault", separated: true },
                    ]}
                />
            </div>
        </div>

        <div class="block detail-block">
            <Eyebrow size="md">Detail pane, loading</Eyebrow>
            <div class="detail">
                <ModDetailPane pending={CatalogFixtures.pendingMapper} text={en.addModBrowse.detail} />
            </div>
        </div>
    </div>
</section>

<style>
    .sheet {
        display: flex;
        flex-direction: column;
        gap: 44px;
        width: 1420px;
        max-width: 100%;
        padding: 48px;
        box-sizing: border-box;
        background: var(--color-rad-sheet);
        border-radius: var(--radius-rad-lg);
    }

    .head {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-page);
        font-weight: var(--font-weight-rad-light);
        letter-spacing: var(--tracking-rad-page);
        line-height: 1.14;
    }

    .title strong {
        font-weight: var(--font-weight-rad-strong);
    }

    .block {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .button-row {
        display: flex;
        align-items: center;
        gap: 28px;
    }

    .row-name {
        width: 110px;
        flex-shrink: 0;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 18px;
    }

    .panes {
        display: flex;
        align-items: flex-start;
        gap: 40px;
    }

    .list-block {
        width: 560px;
        flex-shrink: 0;
    }

    .menu-block {
        width: 250px;
        flex-shrink: 0;
    }

    .detail-block {
        width: 340px;
        flex-shrink: 0;
    }

    .list {
        border-top: 1px solid var(--color-rad-line);
    }

    .list-row {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 12px 16px;
        border-bottom: 1px solid var(--color-rad-line);
    }

    .list-name {
        flex: 1;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-body);
        line-height: 20px;
    }

    .list-note {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        line-height: 16px;
    }

    .menu-host {
        display: flex;
        justify-content: flex-end;
        min-height: 190px;
    }

    .detail {
        overflow: hidden;
        border: 1px solid var(--color-rad-line);
        border-radius: var(--radius-rad-md);
    }

    .detail :global(.amml-detail) {
        border-left: 0;
    }

    /*
     * A page cannot hold a real :hover or :focus-visible on many elements at once, so the sheet draws those two
     * looks with the same declarations the components use on :hover and :focus-visible.
     */
    .state--hover.state--primary :global(.amml-btn) {
        background: var(--color-rad-brand-hover);
    }

    .state--hover.state--secondary :global(.amml-btn) {
        background: var(--color-rad-panel);
        border-color: var(--color-rad-brand-lift);
    }

    .state--hover.state--ghost :global(.amml-btn) {
        color: var(--color-rad-tx);
    }

    .state--focus :global(.amml-btn) {
        outline: var(--rad-focus-ring);
        outline-offset: var(--rad-focus-offset);
    }

    .state-row--hover {
        background: var(--color-rad-hover);
    }

    .state-row--focus {
        outline: var(--rad-focus-ring);
        outline-offset: calc(var(--rad-focus-offset) * -1);
        border-radius: var(--radius-rad-xs);
    }
</style>
