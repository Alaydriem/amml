<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "08 Dialogs and errors", group: "Screens", order: 15, frame: false };
</script>

<script lang="ts">
    import { AddModDialog, ConfirmDialog, FaultBanner } from "../../src/components/dialogs";
    import { DialogFixtures } from "../../src/fixtures/dialogs";
    import { dialogsEn } from "../../src/text/en/dialogs";
    import { faultsEn } from "../../src/text/en/faults";

    let input = $state(DialogFixtures.addModInput);
    let last = $state("");
</script>

<section class="sheet">
    <header class="sheet__head">
        <div class="sheet__eyebrow">Launcher v1 · overlays</div>
        <h2 class="sheet__title">Dialogs and <strong>named error states</strong></h2>
    </header>

    <div class="sheet__dialogs">
        <ConfirmDialog
            inline
            text={dialogsEn.developerMode}
            values={{ file: DialogFixtures.developerModeFile, version: DialogFixtures.developerModeVersion }}
            onconfirm={() => (last = "developer mode: confirm")}
            oncancel={() => (last = "developer mode: cancel")}
        />

        <ConfirmDialog
            inline
            text={dialogsEn.deleteInstance}
            values={{ name: DialogFixtures.deleteName }}
            path={DialogFixtures.deleteFolder}
            confirmVariant="danger"
            onconfirm={() => (last = "delete: confirm")}
            oncancel={() => (last = "delete: cancel")}
        />

        <AddModDialog
            inline
            text={dialogsEn.addMod}
            values={{ name: DialogFixtures.addModInstance }}
            value={input}
            knownIds={DialogFixtures.knownIds}
            suggestions={DialogFixtures.suggestions}
            oninput={(next) => (input = next)}
            onadd={(id) => (last = `add: ${id}`)}
            oninstallurl={(url) => (last = `install unofficial: ${url}`)}
            oncancel={() => (last = "add mod: cancel")}
        />
    </div>

    <div class="sheet__banners">
        <div class="sheet__eyebrow">Error banners · each is its own state</div>
        {#each DialogFixtures.faultOrder as kind (kind)}
            <FaultBanner fault={DialogFixtures.faults[kind]} text={faultsEn[kind]} onaction={(action) => (last = `banner: ${action}`)} />
        {/each}
    </div>

    <p class="sheet__last">Last action: {last}</p>
</section>

<style>
    .sheet {
        display: flex;
        flex-direction: column;
        gap: 40px;
        padding: 48px;
        background: #140c26;
        width: 1420px;
        max-width: 100%;
        box-sizing: border-box;
    }

    .sheet__head {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .sheet__eyebrow {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-small);
        line-height: 18px;
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
    }

    .sheet__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-page);
        font-weight: 300;
        line-height: 42px;
        letter-spacing: -0.024em;
    }

    .sheet__title strong {
        font-weight: 600;
    }

    .sheet__dialogs {
        display: flex;
        align-items: flex-start;
        gap: 32px;
    }

    .sheet__banners {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .sheet__banners > .sheet__eyebrow {
        padding-bottom: 4px;
    }

    .sheet__last {
        margin: 0;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
    }
</style>
