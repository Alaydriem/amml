<script lang="ts">
    import SearchField from "../primitives/SearchField.svelte";
    import Select from "../primitives/Select.svelte";
    import type { CatalogToolbarProps } from "./CatalogToolbar";

    let { query = "", placeholder, sort, resultLine, disabled = false, text, onsearch, onsort }: CatalogToolbarProps = $props();
</script>

<div class="amml-toolbar">
    <div class="amml-toolbar__row">
        <div class="amml-toolbar__search">
            <SearchField value={query} label={text.searchLabel} {placeholder} {disabled} oninput={onsearch} />
        </div>
        <Select compact options={text.sortOptions} value={sort} label={text.sortLabel} {disabled} onchange={onsort} />
    </div>
    <div class="amml-toolbar__result">{resultLine}</div>
</div>

<style>
    .amml-toolbar {
        display: flex;
        flex-shrink: 0;
        flex-direction: column;
        gap: 10px;
        padding: 14px 20px 10px;
        background: var(--color-rad-void);
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-toolbar__row {
        display: flex;
        align-items: stretch;
        gap: 10px;
    }

    .amml-toolbar__search {
        display: flex;
        flex: 1 1 0;
        min-width: 0;
    }

    .amml-toolbar__search :global(.amml-search) {
        flex: 1 1 0;
    }

    .amml-toolbar__result {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
